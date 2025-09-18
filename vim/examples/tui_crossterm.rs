//! Terminal UI example using crossterm and ratatui.
//!
//! This example demonstrates how to integrate vim_mini into a terminal application.
//! Run with: cargo run --example tui_crossterm

use crossterm::{
    event::{self, Event, KeyCode as CKeyCode, KeyEvent as CKeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use ropey::Rope;
use std::io;
use unicode_segmentation::UnicodeSegmentation;
use vim_mini::{
    Engine, InputEvent, KeyCode, KeyEvent, Modifiers,
    traits::{Clipboard, TextOps},
    types::*,
};

/// Simple clipboard implementation using an internal buffer
struct SimpleClipboard {
    content: Option<String>,
}

impl Clipboard for SimpleClipboard {
    fn get(&mut self) -> Option<String> {
        self.content.clone()
    }

    fn set(&mut self, text: String) {
        self.content = Some(text);
    }
}

/// Text buffer implementation using ropey
struct RopeBuffer {
    rope: Rope,
}

impl RopeBuffer {
    fn new() -> Self {
        Self {
            rope: Rope::from(
                "Welcome to vim_mini!\n\nPress 'i' to enter insert mode.\nPress 'Esc' to return to normal mode.\nPress ':q<Enter>' to quit.\n\nTry vim commands like:\n- hjkl for movement\n- dd to delete a line\n- yy to yank (copy) a line\n- p to paste\n- / to search\n",
            ),
        }
    }

    fn apply_command(&mut self, cmd: &Command) {
        match cmd {
            Command::Delete { range } => {
                let start_idx = self.position_to_char_idx(range.start);
                let end_idx = self.position_to_char_idx(range.end);
                self.rope.remove(start_idx..end_idx);
            }
            Command::InsertText { at, text } => {
                let idx = self.position_to_char_idx(*at);
                self.rope.insert(idx, text);
            }
            _ => {} // Cursor and selection handled by app state
        }
    }

    fn position_to_char_idx(&self, pos: Position) -> usize {
        if pos.line >= self.rope.len_lines() as u32 {
            return self.rope.len_chars();
        }

        let line_start_idx = self.rope.line_to_char(pos.line as usize);
        let line = self.rope.line(pos.line as usize);
        let mut char_idx = line_start_idx;

        for (char_count, grapheme) in line.as_str().unwrap_or("").graphemes(true).enumerate() {
            if char_count >= pos.col as usize {
                break;
            }
            char_idx += grapheme.len();
        }

        char_idx
    }

    fn line_text(&self, line: u32) -> String {
        if line < self.rope.len_lines() as u32 {
            self.rope.line(line as usize).to_string()
        } else {
            String::new()
        }
    }
}

impl TextOps for RopeBuffer {
    fn line_count(&self) -> u32 {
        self.rope.len_lines() as u32
    }

    fn line_len(&self, line: u32) -> u32 {
        if line >= self.line_count() {
            return 0;
        }
        let line_str = self.rope.line(line as usize);
        line_str.as_str().unwrap_or("").graphemes(true).count() as u32
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
        // Simplified word motion
        let mut result = pos;
        for _ in 0..count {
            result = self.move_right(result, 5);
        }
        result
    }

    fn prev_word_start(&self, pos: Position, count: u32) -> Position {
        // Simplified word motion
        let mut result = pos;
        for _ in 0..count {
            result = self.move_left(result, 5);
        }
        result
    }

    fn next_paragraph_start(&self, _pos: Position, _count: u32) -> Position {
        // Simplified - just move down 3 lines
        self.move_down(_pos, 3, None)
    }

    fn prev_paragraph_start(&self, _pos: Position, _count: u32) -> Position {
        // Simplified - just move up 3 lines
        self.move_up(_pos, 3, None)
    }

    fn find_in_line(
        &self,
        pos: Position,
        ch: char,
        _before: bool,
        _count: u32,
    ) -> Option<Position> {
        let line_str = self.line_text(pos.line);
        let mut col = 0;
        for grapheme in line_str.graphemes(true).skip(pos.col as usize + 1) {
            col += 1;
            if grapheme.starts_with(ch) {
                return Some(Position {
                    line: pos.line,
                    col: pos.col + col,
                });
            }
        }
        None
    }

    fn slice_to_string(&self, range: Range) -> String {
        let start = self.position_to_char_idx(range.start);
        let end = self.position_to_char_idx(range.end);
        self.rope.slice(start..end).to_string()
    }

    fn search_forward(&self, from: Position, needle: &str, _wrap: bool) -> Option<Position> {
        let start_idx = self.position_to_char_idx(from) + 1;
        let text = self.rope.slice(start_idx..).to_string();

        if let Some(offset) = text.find(needle) {
            let found_idx = start_idx + offset;
            // Convert back to position
            let line = self.rope.char_to_line(found_idx);
            let line_start = self.rope.line_to_char(line);
            let col = self
                .rope
                .slice(line_start..found_idx)
                .as_str()
                .unwrap_or("")
                .graphemes(true)
                .count();
            return Some(Position {
                line: line as u32,
                col: col as u32,
            });
        }
        None
    }

    fn search_backward(&self, from: Position, needle: &str, _wrap: bool) -> Option<Position> {
        let end_idx = self.position_to_char_idx(from);
        let text = self.rope.slice(..end_idx).to_string();

        if let Some(offset) = text.rfind(needle) {
            // Convert back to position
            let line = self.rope.char_to_line(offset);
            let line_start = self.rope.line_to_char(line);
            let col = self
                .rope
                .slice(line_start..offset)
                .as_str()
                .unwrap_or("")
                .graphemes(true)
                .count();
            return Some(Position {
                line: line as u32,
                col: col as u32,
            });
        }
        None
    }
}

struct App {
    engine: Engine,
    buffer: RopeBuffer,
    clipboard: SimpleClipboard,
    cursor: Position,
    selection: Option<Selection>,
    message: String,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            engine: Engine::new(),
            buffer: RopeBuffer::new(),
            clipboard: SimpleClipboard { content: None },
            cursor: Position::ZERO,
            selection: None,
            message: String::new(),
            should_quit: false,
        }
    }

    fn handle_crossterm_event(&mut self, event: CKeyEvent) {
        let vim_event = convert_crossterm_event(event);

        // Handle quit command
        if let InputEvent::Key(ke) = &vim_event
            && self.message == ":q"
            && ke.code == KeyCode::Enter
        {
            self.should_quit = true;
            return;
        }

        let (new_cursor, commands) =
            self.engine
                .handle_event(&self.buffer, &mut self.clipboard, self.cursor, vim_event);

        // Apply commands
        for cmd in commands {
            match &cmd {
                Command::SetCursor(pos) => self.cursor = *pos,
                Command::SetSelection(sel) => self.selection = *sel,
                _ => self.buffer.apply_command(&cmd),
            }
        }

        self.cursor = new_cursor;

        // Update message based on mode
        let snapshot = self.engine.snapshot();
        self.message = match snapshot.mode {
            Mode::Normal => "-- NORMAL --".to_string(),
            Mode::Insert => "-- INSERT --".to_string(),
            Mode::Visual(_) => "-- VISUAL --".to_string(),
            Mode::SearchPrompt => format!("/{}", self.message.trim_start_matches('/')),
        };
    }
}

fn convert_crossterm_event(event: CKeyEvent) -> InputEvent {
    // In insert mode, regular characters should be ReceivedChar
    match event.code {
        CKeyCode::Char(c) => {
            let mods = if event.modifiers.contains(KeyModifiers::SHIFT) {
                Modifiers::SHIFT
            } else if event.modifiers.contains(KeyModifiers::CONTROL) {
                Modifiers::CTRL
            } else {
                Modifiers::empty()
            };

            InputEvent::Key(KeyEvent {
                code: KeyCode::Char(c),
                mods,
            })
        }
        CKeyCode::Esc => InputEvent::Key(KeyEvent {
            code: KeyCode::Esc,
            mods: Modifiers::empty(),
        }),
        CKeyCode::Enter => InputEvent::Key(KeyEvent {
            code: KeyCode::Enter,
            mods: Modifiers::empty(),
        }),
        CKeyCode::Backspace => InputEvent::Key(KeyEvent {
            code: KeyCode::Backspace,
            mods: Modifiers::empty(),
        }),
        _ => InputEvent::Key(KeyEvent {
            code: KeyCode::Esc,
            mods: Modifiers::empty(),
        }),
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
        .split(f.size());

    // Main text area
    let mut lines = vec![];
    for i in 0..app.buffer.line_count() {
        let line_text = app.buffer.line_text(i);
        let trimmed = line_text.trim_end_matches('\n').to_string();

        // Highlight selection if any
        if let Some(sel) = &app.selection
            && i >= sel.start.line
            && i <= sel.end.line
        {
            lines.push(Line::from(Span::styled(
                trimmed,
                Style::default().bg(Color::Blue),
            )));
            continue;
        }

        lines.push(Line::from(trimmed));
    }

    let text = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("vim_mini demo"),
    );
    f.render_widget(text, chunks[0]);

    // Status line
    let status = Paragraph::new(app.message.as_str())
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[1]);

    // Set cursor position
    f.set_cursor(
        chunks[0].x + 1 + app.cursor.col as u16,
        chunks[0].y + 1 + app.cursor.line as u16,
    );
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == CKeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }

            app.handle_crossterm_event(key);

            if app.should_quit {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
