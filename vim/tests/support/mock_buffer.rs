use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use vim_mini::traits::TextOps;
use vim_mini::types::Position;

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
}
