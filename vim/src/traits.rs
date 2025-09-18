use crate::types::{Position, Range};

pub trait TextOps {
    // Basic queries
    fn line_count(&self) -> u32;
    fn line_len(&self, line: u32) -> u32; // grapheme columns

    // Grapheme-aware relative moves (count >=1)
    fn move_left(&self, pos: Position, count: u32) -> Position;
    fn move_right(&self, pos: Position, count: u32) -> Position;

    // Vertical moves: preserve preferred column where possible (host handles virtual columns)
    fn move_up(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position;
    fn move_down(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position;

    fn line_start(&self, line: u32) -> Position;
    fn line_end(&self, line: u32) -> Position; // last character (before newline if any)

    fn clamp(&self, pos: Position) -> Position {
        let last_line = self.line_count().saturating_sub(1);
        let line = pos.line.min(last_line);
        let col = pos.col.min(self.line_len(line));
        Position { line, col }
    }

    // Word motions - Phase 3
    fn next_word_start(&self, pos: Position, count: u32) -> Position;
    fn prev_word_start(&self, pos: Position, count: u32) -> Position;

    // Paragraph motions - Phase 3
    fn next_paragraph_start(&self, pos: Position, count: u32) -> Position;
    fn prev_paragraph_start(&self, pos: Position, count: u32) -> Position;

    // Find in current line - Phase 3
    // find in current line; if before=true emulate 't', else 'f'
    fn find_in_line(&self, pos: Position, ch: char, before: bool, count: u32) -> Option<Position>;

    // Text extraction - Phase 4
    // Extract text from a range for yanking
    fn slice_to_string(&self, range: Range) -> String;

    // Search functionality - Phase 5
    // Search forward/back from (line,col) including wrap; returns position at start of match
    fn search_forward(&self, from: Position, needle: &str, wrap: bool) -> Option<Position>;
    fn search_backward(&self, from: Position, needle: &str, wrap: bool) -> Option<Position>;
}

pub trait Clipboard {
    fn get(&mut self) -> Option<String>;
    fn set(&mut self, text: String);
}
