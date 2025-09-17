use crate::key::{InputEvent, KeyCode};
use crate::traits::TextOps;
use crate::types::{Command, Mode, Position, Range, Selection, VisualKind};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKey {
    None,
    G,                  // for 'gg' sequence
    D,                  // for 'dd' sequence
    F { before: bool }, // for 'f' and 't' find character motions
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Delete,
    Yank, // skeleton; fully implemented later
}

#[derive(Debug, Clone)]
pub struct Engine {
    mode: Mode,
    preferred_col: Option<u32>,
    counts: Counts,
    pending: PendingKey,
    op_pending: Option<Operator>,
    visual_anchor: Option<Position>, // when in Visual mode
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
            op_pending: None,
            visual_anchor: None,
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

    fn clear_pending(&mut self) {
        self.pending = PendingKey::None;
    }

    fn clear_op(&mut self) {
        self.op_pending = None;
    }

    fn apply_delete(&self, start: Position, end: Position) -> Vec<Command> {
        let range = if start <= end {
            Range { start, end }
        } else {
            Range {
                start: end,
                end: start,
            }
        };
        vec![Command::Delete { range }]
    }

    pub fn handle_event<T: TextOps>(
        &mut self,
        text: &T,
        cursor: Position,
        input: InputEvent,
    ) -> (Position, Vec<Command>) {
        // Ensure cursor is within valid bounds before processing
        let cursor = text.clamp(cursor);

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
                // Handle pending sequences
                match (self.pending, ke.code) {
                    (PendingKey::G, KeyCode::Char('g')) => {
                        self.clear_pending();
                        let count = self.counts.current.take();
                        let target_line = match count {
                            Some(n) if n > 0 => (n - 1).min(text.line_count().saturating_sub(1)),
                            _ => 0,
                        };
                        let pos = text.line_start(target_line);
                        self.preferred_col = Some(0);
                        return (pos, vec![Command::SetCursor(pos)]);
                    }
                    (PendingKey::D, KeyCode::Char('d')) => {
                        self.clear_pending();
                        self.clear_op();
                        let count = self.counts.take_or(1);
                        // Delete current line and next (count-1) lines
                        let start = text.line_start(cursor.line);
                        let end_line =
                            (cursor.line + count - 1).min(text.line_count().saturating_sub(1));
                        let end = text.line_end(end_line);
                        // Include newline for line deletion
                        let end_pos = Position {
                            line: end.line + 1,
                            col: 0,
                        };
                        let cmds = self.apply_delete(start, end_pos);
                        return (start, cmds);
                    }
                    (PendingKey::F { before }, KeyCode::Char(ch)) => {
                        self.clear_pending();
                        let count = self.counts.take_or(1);
                        if let Some(pos) = text.find_in_line(cursor, ch, before, count) {
                            // If operator is pending, apply it
                            if let Some(op) = self.op_pending {
                                self.clear_op();
                                let cmds = match op {
                                    Operator::Delete => {
                                        // For 'f', include the target char; for 't', stop before
                                        let end =
                                            if before { pos } else { text.move_right(pos, 1) };
                                        self.apply_delete(cursor, end)
                                    }
                                    Operator::Yank => vec![], // implement in Phase 4
                                };
                                return (cursor, cmds);
                            } else {
                                // Just move
                                self.preferred_col = None;
                                return (pos, vec![Command::SetCursor(pos)]);
                            }
                        } else {
                            // Character not found, clear operator if any
                            self.clear_op();
                            return (cursor, vec![]);
                        }
                    }
                    _ => {
                        // Clear pending if not matched
                        if self.pending != PendingKey::None {
                            self.clear_pending();
                        }
                    }
                }

                // Count digits
                if let KeyCode::Char(c) = ke.code
                    && c.is_ascii_digit()
                {
                    // Leading zero is 0 motion only if no count started
                    if c == '0' && self.counts.current.is_none() && self.op_pending.is_none() {
                        let pos = text.line_start(cursor.line);
                        self.preferred_col = Some(0);
                        return (pos, vec![Command::SetCursor(pos)]);
                    } else if c == '0' && self.counts.current.is_none() && self.op_pending.is_some()
                    {
                        // 0 is a motion when operator pending
                        // Fall through to motion handling
                    } else {
                        self.counts.push_digit((c as u8 - b'0') as u32);
                        return (cursor, vec![]);
                    }
                }

                // If operator is pending, next motion resolves a range
                if let Some(op) = self.op_pending {
                    let count = self.counts.take_or(1);
                    let mut end = cursor;
                    let mut handled = true;

                    match ke.code {
                        KeyCode::Char('h') => {
                            end = text.move_left(cursor, count);
                        }
                        KeyCode::Char('l') => {
                            end = text.move_right(cursor, count);
                        }
                        KeyCode::Char('k') => {
                            end = text.move_up(cursor, count, None);
                        }
                        KeyCode::Char('j') => {
                            end = text.move_down(cursor, count, None);
                        }
                        KeyCode::Char('0') => {
                            end = text.line_start(cursor.line);
                        }
                        KeyCode::Char('$') => {
                            end = text.line_end(cursor.line);
                            // For line-end motion with delete, include the character
                            if matches!(op, Operator::Delete) {
                                end = text.move_right(end, 1);
                            }
                        }
                        KeyCode::Char('w') => {
                            end = text.next_word_start(cursor, count);
                        }
                        KeyCode::Char('b') => {
                            end = text.prev_word_start(cursor, count);
                        }
                        KeyCode::Char('{') => {
                            end = text.prev_paragraph_start(cursor, count);
                        }
                        KeyCode::Char('}') => {
                            end = text.next_paragraph_start(cursor, count);
                        }
                        KeyCode::Char('f') => {
                            // Enter pending state for f motion
                            self.pending = PendingKey::F { before: false };
                            handled = false;
                        }
                        KeyCode::Char('t') => {
                            // Enter pending state for t motion
                            self.pending = PendingKey::F { before: true };
                            handled = false;
                        }
                        _ => {
                            handled = false;
                        }
                    }

                    if handled {
                        let cmds = match op {
                            Operator::Delete => self.apply_delete(cursor, end),
                            Operator::Yank => vec![], // implement in Phase 4
                        };
                        self.clear_op();
                        // Move cursor to start of deleted range
                        let new_cursor = if cursor <= end { cursor } else { end };
                        return (new_cursor, cmds);
                    }
                    // If not handled, continue processing the key normally
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
                    KeyCode::Char('d') => {
                        self.pending = PendingKey::D; // to allow 'dd'
                        self.op_pending = Some(Operator::Delete);
                        (cursor, vec![])
                    }
                    KeyCode::Char('y') => {
                        self.op_pending = Some(Operator::Yank);
                        (cursor, vec![])
                    }
                    KeyCode::Char('x') => {
                        let count = self.counts.take_or(1);
                        // Delete character(s) under cursor
                        let end = text.move_right(cursor, count);
                        if end == cursor {
                            // Nothing to delete
                            return (cursor, vec![]);
                        }
                        let cmds = self.apply_delete(cursor, end);
                        (cursor, cmds)
                    }
                    KeyCode::Char('v') => {
                        self.mode = Mode::Visual(VisualKind::CharWise);
                        self.visual_anchor = Some(cursor);
                        self.clear_pending();
                        self.clear_op();
                        (
                            cursor,
                            vec![Command::SetSelection(Some(Selection {
                                start: cursor,
                                end: cursor,
                                kind: VisualKind::CharWise,
                            }))],
                        )
                    }
                    KeyCode::Char('V') => {
                        self.mode = Mode::Visual(VisualKind::LineWise);
                        let start = text.line_start(cursor.line);
                        let end = text.line_end(cursor.line);
                        self.visual_anchor = Some(cursor);
                        self.clear_pending();
                        self.clear_op();
                        (
                            cursor,
                            vec![Command::SetSelection(Some(Selection {
                                start,
                                end,
                                kind: VisualKind::LineWise,
                            }))],
                        )
                    }
                    KeyCode::Char('w') => {
                        let count = self.counts.take_or(1);
                        let pos = text.next_word_start(cursor, count);
                        self.preferred_col = None;
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('b') => {
                        let count = self.counts.take_or(1);
                        let pos = text.prev_word_start(cursor, count);
                        self.preferred_col = None;
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('{') => {
                        let count = self.counts.take_or(1);
                        let pos = text.prev_paragraph_start(cursor, count);
                        self.preferred_col = Some(0);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('}') => {
                        let count = self.counts.take_or(1);
                        let pos = text.next_paragraph_start(cursor, count);
                        self.preferred_col = Some(0);
                        (pos, vec![Command::SetCursor(pos)])
                    }
                    KeyCode::Char('f') => {
                        self.pending = PendingKey::F { before: false };
                        (cursor, vec![])
                    }
                    KeyCode::Char('t') => {
                        self.pending = PendingKey::F { before: true };
                        (cursor, vec![])
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

            (Mode::Visual(kind), InputEvent::Key(ke)) => {
                let kind = *kind; // Copy to avoid borrow issues
                match ke.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.visual_anchor = None;
                        self.clear_pending();
                        return (cursor, vec![Command::SetSelection(None)]);
                    }
                    KeyCode::Char('v') if matches!(kind, VisualKind::CharWise) => {
                        // Toggle off from charwise
                        self.mode = Mode::Normal;
                        self.visual_anchor = None;
                        return (cursor, vec![Command::SetSelection(None)]);
                    }
                    KeyCode::Char('V') => {
                        // Switch to linewise or toggle off
                        if matches!(kind, VisualKind::LineWise) {
                            self.mode = Mode::Normal;
                            self.visual_anchor = None;
                            return (cursor, vec![Command::SetSelection(None)]);
                        } else {
                            self.mode = Mode::Visual(VisualKind::LineWise);
                            // Update selection to line bounds
                            if let Some(anchor) = self.visual_anchor {
                                let (start_line, end_line) = if anchor.line <= cursor.line {
                                    (anchor.line, cursor.line)
                                } else {
                                    (cursor.line, anchor.line)
                                };
                                let start = text.line_start(start_line);
                                let end = text.line_end(end_line);
                                return (
                                    cursor,
                                    vec![Command::SetSelection(Some(Selection {
                                        start,
                                        end,
                                        kind: VisualKind::LineWise,
                                    }))],
                                );
                            }
                        }
                    }
                    KeyCode::Char('h')
                    | KeyCode::Char('j')
                    | KeyCode::Char('k')
                    | KeyCode::Char('l')
                    | KeyCode::Char('0')
                    | KeyCode::Char('$')
                    | KeyCode::Char('g')
                    | KeyCode::Char('G')
                    | KeyCode::Char('w')
                    | KeyCode::Char('b')
                    | KeyCode::Char('{')
                    | KeyCode::Char('}') => {
                        // Handle movement
                        let count = self.counts.take_or(1);
                        let new_cursor = match ke.code {
                            KeyCode::Char('h') => text.move_left(cursor, count),
                            KeyCode::Char('l') => text.move_right(cursor, count),
                            KeyCode::Char('k') => {
                                let pos = text.move_up(cursor, count, self.preferred_col);
                                self.preferred_col = Some(pos.col);
                                pos
                            }
                            KeyCode::Char('j') => {
                                let pos = text.move_down(cursor, count, self.preferred_col);
                                self.preferred_col = Some(pos.col);
                                pos
                            }
                            KeyCode::Char('0') => {
                                self.preferred_col = Some(0);
                                text.line_start(cursor.line)
                            }
                            KeyCode::Char('$') => {
                                self.preferred_col = None;
                                text.line_end(cursor.line)
                            }
                            KeyCode::Char('g') => {
                                // Handle pending 'g' for gg
                                if self.pending == PendingKey::G {
                                    self.clear_pending();
                                    let count = self.counts.current.take();
                                    let target_line = match count {
                                        Some(n) if n > 0 => {
                                            (n - 1).min(text.line_count().saturating_sub(1))
                                        }
                                        _ => 0,
                                    };
                                    self.preferred_col = Some(0);
                                    text.line_start(target_line)
                                } else {
                                    // Set pending for gg
                                    self.pending = PendingKey::G;
                                    return (cursor, vec![]);
                                }
                            }
                            KeyCode::Char('G') => {
                                let target_line = match self.counts.current.take() {
                                    Some(n) if n > 0 => {
                                        (n - 1).min(text.line_count().saturating_sub(1))
                                    }
                                    _ => text.line_count().saturating_sub(1),
                                };
                                self.preferred_col = Some(0);
                                text.line_start(target_line)
                            }
                            KeyCode::Char('w') => {
                                self.preferred_col = None;
                                text.next_word_start(cursor, count)
                            }
                            KeyCode::Char('b') => {
                                self.preferred_col = None;
                                text.prev_word_start(cursor, count)
                            }
                            KeyCode::Char('{') => {
                                self.preferred_col = Some(0);
                                text.prev_paragraph_start(cursor, count)
                            }
                            KeyCode::Char('}') => {
                                self.preferred_col = Some(0);
                                text.next_paragraph_start(cursor, count)
                            }
                            _ => cursor,
                        };

                        // Update selection based on anchor and new cursor
                        if let Some(anchor) = self.visual_anchor {
                            let selection = match kind {
                                VisualKind::CharWise => {
                                    let (start, end) = if anchor <= new_cursor {
                                        (anchor, new_cursor)
                                    } else {
                                        (new_cursor, anchor)
                                    };
                                    Selection {
                                        start,
                                        end,
                                        kind: VisualKind::CharWise,
                                    }
                                }
                                VisualKind::LineWise => {
                                    let (start_line, end_line) = if anchor.line <= new_cursor.line {
                                        (anchor.line, new_cursor.line)
                                    } else {
                                        (new_cursor.line, anchor.line)
                                    };
                                    let start = text.line_start(start_line);
                                    let end = text.line_end(end_line);
                                    Selection {
                                        start,
                                        end,
                                        kind: VisualKind::LineWise,
                                    }
                                }
                            };
                            return (
                                new_cursor,
                                vec![
                                    Command::SetCursor(new_cursor),
                                    Command::SetSelection(Some(selection)),
                                ],
                            );
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(anchor) = self.visual_anchor {
                            let selection = match kind {
                                VisualKind::CharWise => {
                                    let (start, end) = if anchor <= cursor {
                                        (anchor, cursor)
                                    } else {
                                        (cursor, anchor)
                                    };
                                    // For charwise visual, include the character under cursor
                                    let end = text.move_right(end, 1);
                                    (start, end)
                                }
                                VisualKind::LineWise => {
                                    let (start_line, end_line) = if anchor.line <= cursor.line {
                                        (anchor.line, cursor.line)
                                    } else {
                                        (cursor.line, anchor.line)
                                    };
                                    let start = text.line_start(start_line);
                                    // Include newline for line deletion
                                    let end = Position {
                                        line: end_line + 1,
                                        col: 0,
                                    };
                                    (start, end)
                                }
                            };
                            self.mode = Mode::Normal;
                            self.visual_anchor = None;
                            let cmds = self.apply_delete(selection.0, selection.1);
                            let mut result = cmds;
                            result.push(Command::SetSelection(None));
                            return (selection.0, result);
                        }
                    }
                    _ => {
                        // Unknown key in visual mode
                        return (cursor, vec![]);
                    }
                }
                (cursor, vec![])
            }

            _ => (cursor, vec![]),
        }
    }
}
