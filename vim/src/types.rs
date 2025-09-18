/// A position within a text buffer.
///
/// Positions are zero-indexed and column values are counted in grapheme clusters,
/// not bytes or chars. This ensures correct handling of emoji and combining characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// Zero-based line number.
    pub line: u32,
    /// Zero-based column position in grapheme clusters.
    pub col: u32,
}

impl Position {
    /// The origin position (0, 0).
    pub const ZERO: Position = Position { line: 0, col: 0 };
}

/// A range of text defined by start and end positions.
///
/// Ranges are half-open intervals [start, end), meaning the start position
/// is included but the end position is excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The start position (inclusive).
    pub start: Position,
    /// The end position (exclusive).
    pub end: Position,
}

/// The current mode of the vim engine.
///
/// Vim is a modal editor where the same keys perform different
/// actions depending on the current mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal mode - for navigation and operators.
    Normal,
    /// Insert mode - for typing text.
    Insert,
    /// Visual mode - for selecting text.
    Visual(VisualKind),
    /// Search prompt mode - entering a search query.
    SearchPrompt,
}

/// The type of visual selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualKind {
    /// Character-wise selection (v).
    CharWise,
    /// Line-wise selection (V).
    LineWise,
}

/// A text selection with its type.
///
/// Selections track both the anchor point and current position,
/// as well as whether the selection is character or line-wise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The start of the selection.
    pub start: Position,
    /// The end of the selection.
    pub end: Position,
    /// The type of selection (character or line).
    pub kind: VisualKind,
}

/// Commands emitted by the vim engine for the host to execute.
///
/// These commands represent the concrete actions that should be
/// applied to the text buffer. The host is responsible for implementing
/// these operations on their text storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Update the cursor position.
    SetCursor(Position),
    /// Set or clear the current selection.
    SetSelection(Option<Selection>),

    /// Delete text in the specified range.
    Delete { range: Range },
    /// Insert text at the specified position.
    InsertText { at: Position, text: String },
}
