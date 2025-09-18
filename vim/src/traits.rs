use crate::types::{Position, Range};

/// Operations on text buffers required by the vim engine.
///
/// This trait defines all text operations that the vim engine needs to perform.
/// Implementors should ensure all operations are grapheme-aware (not byte or char based)
/// to correctly handle Unicode text including emoji and combining characters.
///
/// # Examples
///
/// ```no_run
/// use vim_mini::traits::TextOps;
/// use vim_mini::types::{Position, Range};
///
/// struct MyBuffer {
///     lines: Vec<String>,
/// }
///
/// impl TextOps for MyBuffer {
///     fn line_count(&self) -> u32 {
///         self.lines.len() as u32
///     }
///     // ... implement other methods
/// }
/// ```
pub trait TextOps {
    /// Returns the number of lines in the buffer.
    fn line_count(&self) -> u32;

    /// Returns the length of a line in grapheme clusters (not bytes or chars).
    fn line_len(&self, line: u32) -> u32;

    /// Move left by `count` grapheme clusters from the given position.
    /// Should not move past the beginning of the line.
    fn move_left(&self, pos: Position, count: u32) -> Position;

    /// Move right by `count` grapheme clusters from the given position.
    /// Should not move past the end of the line.
    fn move_right(&self, pos: Position, count: u32) -> Position;

    /// Move up by `count` lines, preserving the preferred column if possible.
    ///
    /// The `preferred_col` is used to maintain the cursor column when moving
    /// through lines of different lengths.
    fn move_up(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position;

    /// Move down by `count` lines, preserving the preferred column if possible.
    fn move_down(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position;

    /// Returns the position at the start of the given line.
    fn line_start(&self, line: u32) -> Position;

    /// Returns the position at the last character of the line (before any newline).
    fn line_end(&self, line: u32) -> Position;

    /// Clamps a position to be within valid buffer bounds.
    ///
    /// This ensures the position is not beyond the last line or column.
    fn clamp(&self, pos: Position) -> Position {
        let last_line = self.line_count().saturating_sub(1);
        let line = pos.line.min(last_line);
        let col = pos.col.min(self.line_len(line));
        Position { line, col }
    }

    /// Find the start of the next word from the given position.
    ///
    /// Words are defined as sequences of alphanumeric characters and underscores.
    /// The `count` parameter specifies how many words to skip.
    fn next_word_start(&self, pos: Position, count: u32) -> Position;

    /// Find the start of the previous word from the given position.
    fn prev_word_start(&self, pos: Position, count: u32) -> Position;

    /// Find the start of the next paragraph.
    ///
    /// Paragraphs are separated by one or more blank lines.
    fn next_paragraph_start(&self, pos: Position, count: u32) -> Position;

    /// Find the start of the previous paragraph.
    fn prev_paragraph_start(&self, pos: Position, count: u32) -> Position;

    /// Find a character in the current line.
    ///
    /// - If `before` is false, finds the character position ('f' behavior)
    /// - If `before` is true, finds the position before the character ('t' behavior)
    /// - Returns None if the character is not found
    fn find_in_line(&self, pos: Position, ch: char, before: bool, count: u32) -> Option<Position>;

    /// Extract text from the buffer as a string.
    ///
    /// Used for yanking (copying) text. The range is half-open [start, end).
    fn slice_to_string(&self, range: Range) -> String;

    /// Search forward for a substring.
    ///
    /// - Starts searching after the `from` position
    /// - If `wrap` is true and no match is found, wraps to the beginning
    /// - Returns the position at the start of the match
    fn search_forward(&self, from: Position, needle: &str, wrap: bool) -> Option<Position>;

    /// Search backward for a substring.
    ///
    /// - Starts searching before the `from` position
    /// - If `wrap` is true and no match is found, wraps to the end
    /// - Returns the position at the start of the match
    fn search_backward(&self, from: Position, needle: &str, wrap: bool) -> Option<Position>;
}

/// Clipboard operations for yanking and pasting.
///
/// Implementors can provide system clipboard integration or
/// use an internal buffer for clipboard operations.
///
/// # Examples
///
/// ```no_run
/// use vim_mini::traits::Clipboard;
///
/// struct SystemClipboard;
///
/// impl Clipboard for SystemClipboard {
///     fn get(&mut self) -> Option<String> {
///         // Get from system clipboard
///         None
///     }
///
///     fn set(&mut self, text: String) {
///         // Set system clipboard
///     }
/// }
/// ```
pub trait Clipboard {
    /// Get the current clipboard contents.
    fn get(&mut self) -> Option<String>;

    /// Set the clipboard contents.
    fn set(&mut self, text: String);
}
