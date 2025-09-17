#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub col: u32, // col is grapheme column
}

impl Position {
    pub const ZERO: Position = Position { line: 0, col: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position, // half-open [start, end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual(VisualKind),
    SearchPrompt, // added later
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualKind {
    CharWise,
    LineWise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
    pub kind: VisualKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Cursor/selection
    SetCursor(Position),
    SetSelection(Option<Selection>),

    // Text edits (host must apply)
    Delete { range: Range },
    InsertText { at: Position, text: String },
    // We'll add more in later phases (Yank, Paste, Transaction markers)
}
