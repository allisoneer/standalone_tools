use crate::key::{InputEvent, KeyCode};
use crate::traits::TextOps;
use crate::types::{Command, Mode, Position};

#[derive(Debug, Default, Clone)]
struct Counts {
    current: Option<u32>,
}

impl Counts {
    fn push_digit(&mut self, d: u32) {
        let next = self
            .current
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(d);
        self.current = Some(next);
    }

    fn take_or(&mut self, default_: u32) -> u32 {
        let v = self.current.take().unwrap_or(default_);
        v.max(1)
    }
}

#[derive(Debug, Clone)]
enum PendingKey {
    None,
    G, // for 'gg' sequence
}

#[derive(Debug, Clone)]
pub struct Engine {
    mode: Mode,
    preferred_col: Option<u32>,
    counts: Counts,
    pending: PendingKey,
}

#[derive(Debug, Clone)]
pub struct EngineSnapshot {
    pub mode: Mode,
    pub preferred_col: Option<u32>,
    pub pending_count: Option<u32>,
}

pub struct EngineBuilder {
    mode: Mode,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self { mode: Mode::Normal }
    }
}

impl EngineBuilder {
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    pub fn build(self) -> Engine {
        Engine {
            mode: self.mode,
            preferred_col: None,
            counts: Counts::default(),
            pending: PendingKey::None,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        EngineBuilder::default().build()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot {
            mode: self.mode,
            preferred_col: self.preferred_col,
            pending_count: self.counts.current,
        }
    }

    pub fn handle_event<T: TextOps>(
        &mut self,
        text: &T,
        cursor: Position,
        input: InputEvent,
    ) -> (Position, Vec<Command>) {
        match (&self.mode, input) {
            (Mode::Insert, InputEvent::Key(ke)) => {
                if let KeyCode::Esc = ke.code {
                    self.mode = Mode::Normal;
                    return (cursor, vec![]);
                }
                // Insert-mode special keys in later phase (Backspace, Enter)
                (cursor, vec![])
            }
            (Mode::Insert, InputEvent::ReceivedChar(ch)) => {
                // Direct insertion; host applies this edit
                let cmd = Command::InsertText {
                    at: cursor,
                    text: ch.to_string(),
                };
                (
                    Position {
                        line: cursor.line,
                        col: cursor.col + 1,
                    },
                    vec![cmd],
                )
            }

            (Mode::Normal, InputEvent::Key(ke)) => {
                // Handle pending 'g' for 'gg'
                if let PendingKey::G = self.pending {
                    self.pending = PendingKey::None;
                    if let KeyCode::Char('g') = ke.code {
                        let count = self.counts.current.take();
                        let target_line = match count {
                            Some(n) if n > 0 => (n - 1).min(text.line_count().saturating_sub(1)),
                            _ => 0,
                        };
                        let pos = text.line_start(target_line);
                        return (pos, vec![Command::SetCursor(pos)]);
                    }
                    // If not 'g', fall through and process normally
                }

                // Count digits
                if let KeyCode::Char(c) = ke.code
                    && c.is_ascii_digit()
                {
                    // Leading zero is 0 motion only if no count started
                    if c == '0' && self.counts.current.is_none() {
                        let pos = text.line_start(cursor.line);
                        self.preferred_col = Some(0);
                        return (pos, vec![Command::SetCursor(pos)]);
                    } else {
                        self.counts.push_digit((c as u8 - b'0') as u32);
                        return (cursor, vec![]);
                    }
                }

                // Motions and mode switches
                match ke.code {
                    KeyCode::Char('h') => {
                        let count = self.counts.take_or(1);
                        let pos = text.move_left(cursor, count);
                        self.preferred_col = None;
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('l') => {
                        let count = self.counts.take_or(1);
                        let pos = text.move_right(cursor, count);
                        self.preferred_col = None;
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('k') => {
                        let count = self.counts.take_or(1);
                        let pos = text.move_up(cursor, count, self.preferred_col);
                        self.preferred_col = Some(pos.col);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('j') => {
                        let count = self.counts.take_or(1);
                        let pos = text.move_down(cursor, count, self.preferred_col);
                        self.preferred_col = Some(pos.col);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('0') => {
                        let pos = text.line_start(cursor.line);
                        self.counts.current = None;
                        self.preferred_col = Some(0);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('$') => {
                        let pos = text.line_end(cursor.line);
                        self.counts.current = None;
                        self.preferred_col = None;
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('g') => {
                        self.pending = PendingKey::G;
                        (cursor, vec![])
                    }
                    KeyCode::Char('G') => {
                        let count = self.counts.current.take();
                        let target_line = match count {
                            Some(n) if n > 0 => (n - 1).min(text.line_count().saturating_sub(1)),
                            _ => text.line_count().saturating_sub(1),
                        };
                        let pos = text.line_start(target_line);
                        self.preferred_col = Some(0);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('i') => {
                        self.mode = Mode::Insert;
                        self.counts.current = None;
                        self.pending = PendingKey::None;
                        (cursor, vec![])
                    }
                    KeyCode::Char('a') => {
                        self.mode = Mode::Insert;
                        self.counts.current = None;
                        self.pending = PendingKey::None;
                        // move right by 1 if possible
                        let pos = text.move_right(cursor, 1);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('I') => {
                        self.mode = Mode::Insert;
                        self.counts.current = None;
                        self.pending = PendingKey::None;
                        let pos = text.line_start(cursor.line);
                        self.preferred_col = Some(0);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('A') => {
                        self.mode = Mode::Insert;
                        self.counts.current = None;
                        self.pending = PendingKey::None;
                        let pos = text.line_end(cursor.line);
                        self.preferred_col = None;
                        // Move one past line end for append
                        let pos = text.move_right(pos, 1);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Esc => {
                        self.counts.current = None;
                        self.pending = PendingKey::None;
                        self.preferred_col = None;
                        (cursor, vec![])
                    }
                    _ => {
                        // Unknown key, clear pending state
                        self.pending = PendingKey::None;
                        (cursor, vec![])
                    }
                }
            }

            // Visual and Search modes come later phases
            _ => (cursor, vec![]),
        }
    }
}
