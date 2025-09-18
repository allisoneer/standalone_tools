use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use vim_mini::traits::TextOps;
use vim_mini::types::{Position, Range};

pub struct MockBuffer {
    rope: Rope,
}

impl MockBuffer {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    fn line_str(&self, line: u32) -> String {
        if line as usize >= self.rope.len_lines() {
            return String::new();
        }
        let line_ref = self.rope.line(line as usize);
        let mut s = line_ref.to_string();
        // Remove trailing newline if present
        if s.ends_with('\n') {
            s.pop();
        }
        s
    }

    fn grapheme_count(&self, s: &str) -> u32 {
        s.graphemes(true).count() as u32
    }

    fn graphemes_at_col(&self, line: u32, _col: u32) -> Vec<String> {
        let s = self.line_str(line);
        s.graphemes(true).map(|g| g.to_string()).collect()
    }

    fn is_word_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    fn is_blank_line(&self, line: u32) -> bool {
        self.line_str(line).trim().is_empty()
    }
}

impl TextOps for MockBuffer {
    fn line_count(&self) -> u32 {
        self.rope.len_lines() as u32
    }

    fn line_len(&self, line: u32) -> u32 {
        self.grapheme_count(&self.line_str(line))
    }

    fn line_start(&self, line: u32) -> Position {
        Position { line, col: 0 }
    }

    fn line_end(&self, line: u32) -> Position {
        let len = self.line_len(line);
        // Position at last character, not past it
        let col = if len > 0 { len - 1 } else { 0 };
        Position { line, col }
    }

    fn move_left(&self, pos: Position, count: u32) -> Position {
        let col = pos.col.saturating_sub(count);
        Position {
            line: pos.line,
            col,
        }
    }

    fn move_right(&self, pos: Position, count: u32) -> Position {
        let max = self.line_len(pos.line);
        // Allow moving to one past last character (for append mode)
        let col = (pos.col + count).min(max);
        Position {
            line: pos.line,
            col,
        }
    }

    fn move_up(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position {
        let line = pos.line.saturating_sub(count);
        let target_col = preferred_col.unwrap_or(pos.col);
        let max_col = self.line_len(line);
        // Don't go past line end
        let col = if max_col > 0 {
            target_col.min(max_col - 1)
        } else {
            0
        };
        Position { line, col }
    }

    fn move_down(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position {
        let line = (pos.line + count).min(self.line_count().saturating_sub(1));
        let target_col = preferred_col.unwrap_or(pos.col);
        let max_col = self.line_len(line);
        // Don't go past line end
        let col = if max_col > 0 {
            target_col.min(max_col - 1)
        } else {
            0
        };
        Position { line, col }
    }

    fn next_word_start(&self, pos: Position, count: u32) -> Position {
        let mut current_pos = pos;
        let mut words_found = 0;

        while words_found < count {
            let found_word;
            let mut in_word = false;

            // Check if we're currently in a word
            let graphemes = self.graphemes_at_col(current_pos.line, 0);
            if let Some(grapheme) = graphemes.get(current_pos.col as usize)
                && let Some(first_char) = grapheme.chars().next()
            {
                in_word = Self::is_word_char(first_char);
            }

            // Scan forward
            loop {
                let graphemes = self.graphemes_at_col(current_pos.line, 0);
                let col = current_pos.col as usize;

                // Move past current position
                if col + 1 < graphemes.len() {
                    current_pos.col += 1;
                    if let Some(ch) = graphemes[current_pos.col as usize].chars().next() {
                        let is_word = Self::is_word_char(ch);
                        if !in_word && is_word {
                            found_word = true;
                            break;
                        }
                        in_word = is_word;
                    }
                } else {
                    // Move to next line
                    if current_pos.line + 1 < self.line_count() {
                        current_pos.line += 1;
                        current_pos.col = 0;
                        let graphemes = self.graphemes_at_col(current_pos.line, 0);
                        if let Some(grapheme) = graphemes.first()
                            && let Some(ch) = grapheme.chars().next()
                        {
                            if Self::is_word_char(ch) {
                                found_word = true;
                                break;
                            }
                            in_word = Self::is_word_char(ch);
                        }
                    } else {
                        // End of buffer
                        return self.clamp(current_pos);
                    }
                }
            }

            if found_word {
                words_found += 1;
            }
        }

        self.clamp(current_pos)
    }

    fn prev_word_start(&self, pos: Position, count: u32) -> Position {
        let mut current_pos = pos;
        let mut words_found = 0;

        while words_found < count {
            let found_word;

            // Move at least one position back
            if current_pos.col > 0 {
                current_pos.col -= 1;
            } else if current_pos.line > 0 {
                current_pos.line -= 1;
                current_pos.col = self.line_len(current_pos.line).saturating_sub(1);
            } else {
                return Position { line: 0, col: 0 };
            }

            // Scan backward to find word start
            loop {
                let graphemes = self.graphemes_at_col(current_pos.line, 0);
                if (current_pos.col as usize) < graphemes.len()
                    && let Some(ch) = graphemes[current_pos.col as usize].chars().next()
                    && Self::is_word_char(ch)
                {
                    // Check if this is the start of a word
                    if current_pos.col == 0 {
                        found_word = true;
                        break;
                    } else if let Some(prev_grapheme) = graphemes.get(current_pos.col as usize - 1)
                        && let Some(prev_ch) = prev_grapheme.chars().next()
                        && !Self::is_word_char(prev_ch)
                    {
                        found_word = true;
                        break;
                    }
                }

                // Move back
                if current_pos.col > 0 {
                    current_pos.col -= 1;
                } else if current_pos.line > 0 {
                    current_pos.line -= 1;
                    current_pos.col = self.line_len(current_pos.line).saturating_sub(1);
                } else {
                    return Position { line: 0, col: 0 };
                }
            }

            if found_word {
                words_found += 1;
            }
        }

        self.clamp(current_pos)
    }

    fn next_paragraph_start(&self, pos: Position, count: u32) -> Position {
        let mut current_line = pos.line;
        let mut paragraphs_found = 0;

        while paragraphs_found < count && current_line < self.line_count() {
            // Skip current paragraph (non-blank lines)
            while current_line < self.line_count() && !self.is_blank_line(current_line) {
                current_line += 1;
            }

            // Skip blank lines
            while current_line < self.line_count() && self.is_blank_line(current_line) {
                current_line += 1;
            }

            // If we found a non-blank line, that's the start of a paragraph
            if current_line < self.line_count() {
                paragraphs_found += 1;
                if paragraphs_found == count {
                    break;
                }
            }
        }

        if current_line >= self.line_count() {
            current_line = self.line_count().saturating_sub(1);
        }

        self.line_start(current_line)
    }

    fn prev_paragraph_start(&self, pos: Position, count: u32) -> Position {
        let mut current_line = pos.line;
        let mut paragraphs_found = 0;

        while paragraphs_found < count && current_line > 0 {
            // Move to previous line
            current_line = current_line.saturating_sub(1);

            // Skip backward through current paragraph
            while current_line > 0 && !self.is_blank_line(current_line) {
                current_line = current_line.saturating_sub(1);
            }

            // Skip blank lines
            while current_line > 0 && self.is_blank_line(current_line) {
                current_line = current_line.saturating_sub(1);
            }

            // Find start of paragraph
            while current_line > 0 && !self.is_blank_line(current_line.saturating_sub(1)) {
                current_line = current_line.saturating_sub(1);
            }

            paragraphs_found += 1;
        }

        self.line_start(current_line)
    }

    fn find_in_line(&self, pos: Position, ch: char, _before: bool, count: u32) -> Option<Position> {
        let graphemes = self.graphemes_at_col(pos.line, 0);
        let mut matches_found = 0;
        let start_col = (pos.col + 1) as usize; // Start searching after current position

        for (idx, grapheme) in graphemes.iter().enumerate().skip(start_col) {
            if grapheme.chars().any(|c| c == ch) {
                matches_found += 1;
                if matches_found == count {
                    // Always return the position of the found character
                    // The engine will decide how to use it based on 'f' or 't'
                    return Some(Position {
                        line: pos.line,
                        col: idx as u32,
                    });
                }
            }
        }

        None
    }

    fn slice_to_string(&self, range: Range) -> String {
        if range.start == range.end {
            return String::new();
        }

        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;

        if start_line == end_line {
            // Single line case
            let line = self.line_str(range.start.line);
            let graphemes: Vec<&str> = line.graphemes(true).collect();
            let start_col = range.start.col as usize;
            let end_col = range.end.col.min(graphemes.len() as u32) as usize;

            graphemes[start_col..end_col].join("")
        } else {
            // Multi-line case
            let mut result = String::new();

            // First line
            let first_line = self.line_str(range.start.line);
            let first_graphemes: Vec<&str> = first_line.graphemes(true).collect();
            let start_col = range.start.col as usize;
            result.push_str(&first_graphemes[start_col..].join(""));
            result.push('\n');

            // Middle lines
            for line_idx in (start_line + 1)..end_line {
                let line_ref = self.rope.line(line_idx);
                result.push_str(line_ref.as_str().unwrap_or(""));
            }

            // Last line
            if end_line < self.rope.len_lines() {
                let last_line = self.line_str(range.end.line);
                let last_graphemes: Vec<&str> = last_line.graphemes(true).collect();
                let end_col = range.end.col.min(last_graphemes.len() as u32) as usize;
                result.push_str(&last_graphemes[0..end_col].join(""));
            }

            result
        }
    }

    fn search_forward(&self, from: Position, needle: &str, wrap: bool) -> Option<Position> {
        if needle.is_empty() {
            return None;
        }

        let total_lines = self.line_count() as usize;

        // Search from current position to end of file
        for line_idx in from.line as usize..total_lines {
            let line = self.line_str(line_idx as u32);
            let graphemes: Vec<&str> = line.graphemes(true).collect();

            let start_col = if line_idx == from.line as usize {
                (from.col + 1) as usize // Start searching after current position
            } else {
                0
            };

            // Search for needle in this line starting from start_col
            for col in start_col..graphemes.len() {
                let remaining = graphemes[col..].join("");
                if remaining.starts_with(needle) {
                    return Some(Position {
                        line: line_idx as u32,
                        col: col as u32,
                    });
                }
            }
        }

        // If wrap is enabled, search from beginning to original position
        if wrap {
            for line_idx in 0..=from.line as usize {
                let line = self.line_str(line_idx as u32);
                let graphemes: Vec<&str> = line.graphemes(true).collect();

                let end_col = if line_idx == from.line as usize {
                    (from.col + 1) as usize
                } else {
                    graphemes.len()
                };

                for col in 0..end_col {
                    let remaining = graphemes[col..].join("");
                    if remaining.starts_with(needle) {
                        return Some(Position {
                            line: line_idx as u32,
                            col: col as u32,
                        });
                    }
                }
            }
        }

        None
    }

    fn search_backward(&self, from: Position, needle: &str, wrap: bool) -> Option<Position> {
        if needle.is_empty() {
            return None;
        }

        // Search from current position backward to beginning of file
        for line_idx in (0..=from.line as usize).rev() {
            let line = self.line_str(line_idx as u32);
            let graphemes: Vec<&str> = line.graphemes(true).collect();

            let end_col = if line_idx == from.line as usize {
                from.col as usize // Search up to (not including) current position
            } else {
                graphemes.len()
            };

            // Search backward in this line
            for col in (0..end_col).rev() {
                let remaining = graphemes[col..].join("");
                if remaining.starts_with(needle) {
                    return Some(Position {
                        line: line_idx as u32,
                        col: col as u32,
                    });
                }
            }
        }

        // If wrap is enabled, search from end backward to original position
        if wrap {
            let total_lines = self.line_count() as usize;
            for line_idx in ((from.line as usize)..total_lines).rev() {
                let line = self.line_str(line_idx as u32);
                let graphemes: Vec<&str> = line.graphemes(true).collect();

                let start_col = if line_idx == from.line as usize {
                    from.col as usize
                } else {
                    0
                };

                for col in (start_col..graphemes.len()).rev() {
                    let remaining = graphemes[col..].join("");
                    if remaining.starts_with(needle) {
                        return Some(Position {
                            line: line_idx as u32,
                            col: col as u32,
                        });
                    }
                }
            }
        }

        None
    }
}
