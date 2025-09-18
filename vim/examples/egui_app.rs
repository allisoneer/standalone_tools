//! GUI example using egui.
//!
//! This example demonstrates how to integrate vim_mini into a GUI application.
//! Run with: cargo run --example egui_app

use eframe::egui;
use vim_mini::{
    Engine, InputEvent, KeyCode, KeyEvent, Modifiers,
    traits::{Clipboard, TextOps},
    types::*,
};

/// Simple text buffer backed by a string
struct StringBuffer {
    content: String,
    lines: Vec<String>,
}

impl StringBuffer {
    fn new(initial: &str) -> Self {
        let content = initial.to_string();
        let lines = content.lines().map(|s| s.to_string()).collect();
        Self { content, lines }
    }

    fn rebuild_lines(&mut self) {
        self.lines = self.content.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }

    fn position_to_byte_offset(&self, pos: Position) -> usize {
        let mut offset = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if i < pos.line as usize {
                offset += line.len() + 1; // +1 for newline
            } else {
                let chars: Vec<_> = line.chars().collect();
                for ch in chars.iter().take(pos.col.min(chars.len() as u32) as usize) {
                    offset += ch.len_utf8();
                }
                break;
            }
        }
        offset
    }

    fn apply_command(&mut self, cmd: &Command) {
        match cmd {
            Command::Delete { range } => {
                let start = self.position_to_byte_offset(range.start);
                let end = self.position_to_byte_offset(range.end);
                self.content.drain(start..end);
                self.rebuild_lines();
            }
            Command::InsertText { at, text } => {
                let offset = self.position_to_byte_offset(*at);
                self.content.insert_str(offset, text);
                self.rebuild_lines();
            }
            _ => {}
        }
    }
}

impl TextOps for StringBuffer {
    fn line_count(&self) -> u32 {
        self.lines.len() as u32
    }

    fn line_len(&self, line: u32) -> u32 {
        self.lines
            .get(line as usize)
            .map(|l| l.chars().count() as u32)
            .unwrap_or(0)
    }

    fn move_left(&self, pos: Position, count: u32) -> Position {
        Position {
            line: pos.line,
            col: pos.col.saturating_sub(count),
        }
    }

    fn move_right(&self, pos: Position, count: u32) -> Position {
        let max_col = self.line_len(pos.line);
        Position {
            line: pos.line,
            col: (pos.col + count).min(max_col),
        }
    }

    fn move_up(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position {
        let new_line = pos.line.saturating_sub(count);
        let col = preferred_col
            .unwrap_or(pos.col)
            .min(self.line_len(new_line));
        Position {
            line: new_line,
            col,
        }
    }

    fn move_down(&self, pos: Position, count: u32, preferred_col: Option<u32>) -> Position {
        let new_line = (pos.line + count).min(self.line_count().saturating_sub(1));
        let col = preferred_col
            .unwrap_or(pos.col)
            .min(self.line_len(new_line));
        Position {
            line: new_line,
            col,
        }
    }

    fn line_start(&self, line: u32) -> Position {
        Position { line, col: 0 }
    }

    fn line_end(&self, line: u32) -> Position {
        let len = self.line_len(line);
        Position {
            line,
            col: len.saturating_sub(1),
        }
    }

    fn next_word_start(&self, pos: Position, count: u32) -> Position {
        // Simplified implementation
        let mut result = pos;
        for _ in 0..count {
            result = self.move_right(result, 5);
        }
        result
    }

    fn prev_word_start(&self, pos: Position, count: u32) -> Position {
        // Simplified implementation
        let mut result = pos;
        for _ in 0..count {
            result = self.move_left(result, 5);
        }
        result
    }

    fn next_paragraph_start(&self, pos: Position, _count: u32) -> Position {
        self.move_down(pos, 3, None)
    }

    fn prev_paragraph_start(&self, pos: Position, _count: u32) -> Position {
        self.move_up(pos, 3, None)
    }

    fn find_in_line(
        &self,
        pos: Position,
        ch: char,
        _before: bool,
        _count: u32,
    ) -> Option<Position> {
        if let Some(line) = self.lines.get(pos.line as usize) {
            let chars: Vec<_> = line.chars().collect();
            for (i, &c) in chars.iter().enumerate().skip((pos.col + 1) as usize) {
                if c == ch {
                    return Some(Position {
                        line: pos.line,
                        col: i as u32,
                    });
                }
            }
        }
        None
    }

    fn slice_to_string(&self, range: Range) -> String {
        let start = self.position_to_byte_offset(range.start);
        let end = self.position_to_byte_offset(range.end);
        self.content[start..end].to_string()
    }

    fn search_forward(&self, from: Position, needle: &str, _wrap: bool) -> Option<Position> {
        let start_offset = self.position_to_byte_offset(from) + 1;
        if let Some(found) = self.content[start_offset..].find(needle) {
            let abs_offset = start_offset + found;
            // Convert back to position
            let mut line = 0;
            let mut offset = 0;
            for (i, l) in self.lines.iter().enumerate() {
                if offset + l.len() + 1 > abs_offset {
                    line = i;
                    break;
                }
                offset += l.len() + 1;
            }
            let col = (abs_offset - offset) as u32;
            return Some(Position {
                line: line as u32,
                col,
            });
        }
        None
    }

    fn search_backward(&self, from: Position, needle: &str, _wrap: bool) -> Option<Position> {
        let end_offset = self.position_to_byte_offset(from);
        if let Some(found) = self.content[..end_offset].rfind(needle) {
            // Convert back to position
            let mut line = 0;
            let mut offset = 0;
            for (i, l) in self.lines.iter().enumerate() {
                if offset + l.len() + 1 > found {
                    line = i;
                    break;
                }
                offset += l.len() + 1;
            }
            let col = (found - offset) as u32;
            return Some(Position {
                line: line as u32,
                col,
            });
        }
        None
    }
}

struct InternalClipboard {
    content: Option<String>,
}

impl Clipboard for InternalClipboard {
    fn get(&mut self) -> Option<String> {
        self.content.clone()
    }

    fn set(&mut self, text: String) {
        self.content = Some(text);
    }
}

struct VimApp {
    engine: Engine,
    buffer: StringBuffer,
    clipboard: InternalClipboard,
    cursor: Position,
    selection: Option<Selection>,
    search_query: String,
}

impl Default for VimApp {
    fn default() -> Self {
        Self {
            engine: Engine::new(),
            buffer: StringBuffer::new(
                "Welcome to vim_mini GUI demo!\n\n\
                 Use standard vim keys:\n\
                 - i: insert mode\n\
                 - hjkl: movement\n\
                 - dd: delete line\n\
                 - yy: yank line\n\
                 - p: paste\n\
                 - v: visual mode\n\
                 - /: search\n",
            ),
            clipboard: InternalClipboard { content: None },
            cursor: Position::ZERO,
            selection: None,
            search_query: String::new(),
        }
    }
}

impl VimApp {
    fn handle_key_event(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        let vim_event = convert_egui_event(key, modifiers, &self.engine.snapshot().mode);

        if let Some(event) = vim_event {
            let (new_cursor, commands) =
                self.engine
                    .handle_event(&self.buffer, &mut self.clipboard, self.cursor, event);

            for cmd in commands {
                match &cmd {
                    Command::SetCursor(pos) => self.cursor = *pos,
                    Command::SetSelection(sel) => self.selection = *sel,
                    _ => self.buffer.apply_command(&cmd),
                }
            }

            self.cursor = new_cursor;
        }
    }

    fn handle_char_input(&mut self, ch: char) {
        let mode = self.engine.snapshot().mode;
        let event = match mode {
            Mode::Insert | Mode::SearchPrompt => InputEvent::ReceivedChar(ch),
            _ => InputEvent::Key(KeyEvent {
                code: KeyCode::Char(ch),
                mods: Modifiers::empty(),
            }),
        };

        let (new_cursor, commands) =
            self.engine
                .handle_event(&self.buffer, &mut self.clipboard, self.cursor, event);

        for cmd in commands {
            match &cmd {
                Command::SetCursor(pos) => self.cursor = *pos,
                Command::SetSelection(sel) => self.selection = *sel,
                _ => self.buffer.apply_command(&cmd),
            }
        }

        self.cursor = new_cursor;

        // Update search query in search mode
        if let Mode::SearchPrompt = mode {
            if ch == '\n' {
                self.search_query.clear();
            } else {
                self.search_query.push(ch);
            }
        }
    }
}

fn convert_egui_event(
    key: egui::Key,
    _modifiers: egui::Modifiers,
    _mode: &Mode,
) -> Option<InputEvent> {
    match key {
        egui::Key::Escape => Some(InputEvent::Key(KeyEvent {
            code: KeyCode::Esc,
            mods: Modifiers::empty(),
        })),
        egui::Key::Enter => Some(InputEvent::Key(KeyEvent {
            code: KeyCode::Enter,
            mods: Modifiers::empty(),
        })),
        egui::Key::Backspace => Some(InputEvent::Key(KeyEvent {
            code: KeyCode::Backspace,
            mods: Modifiers::empty(),
        })),
        _ => None,
    }
}

impl eframe::App for VimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("vim_mini GUI Demo");

            // Mode display
            let mode_text = match self.engine.snapshot().mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Visual(_) => "VISUAL",
                Mode::SearchPrompt => &format!("SEARCH: /{}", self.search_query),
            };
            ui.label(format!("Mode: {}", mode_text));

            ui.separator();

            // Text editor area
            let text_edit_id = ui.make_persistent_id("text_editor");
            let mut text = self.buffer.content.clone();

            let response = ui.add(
                egui::TextEdit::multiline(&mut text)
                    .id(text_edit_id)
                    .desired_width(f32::INFINITY)
                    .desired_rows(20)
                    .font(egui::TextStyle::Monospace),
            );

            // Handle keyboard input
            if response.has_focus() {
                // Handle special keys
                for event in &ui.input(|i| i.events.clone()) {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        self.handle_key_event(*key, *modifiers);
                    } else if let egui::Event::Text(text) = event {
                        for ch in text.chars() {
                            self.handle_char_input(ch);
                        }
                    }
                }
            }

            // Set cursor position
            if let Some(mut state) = egui::TextEdit::load_state(ctx, text_edit_id) {
                let byte_offset = self.buffer.position_to_byte_offset(self.cursor);
                let cursor = egui::text::CCursor::new(byte_offset);
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                state.store(ctx, text_edit_id);
            }

            ui.separator();

            // Status line
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Line: {}, Col: {}",
                    self.cursor.line + 1,
                    self.cursor.col + 1
                ));
                if let Some(content) = self.clipboard.get() {
                    ui.label(format!("Clipboard: {} chars", content.len()));
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "vim_mini GUI Demo",
        options,
        Box::new(|_cc| Box::<VimApp>::default()),
    )
}
