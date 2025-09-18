//! Benchmarks for vim_mini keystroke performance.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ropey::Rope;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use vim_mini::{
    Engine, InputEvent, KeyCode, KeyEvent, Modifiers,
    traits::{Clipboard, TextOps},
    types::*,
};

/// Mock clipboard for benchmarking
struct MockClipboard {
    content: Option<String>,
}

impl Clipboard for MockClipboard {
    fn get(&mut self) -> Option<String> {
        self.content.clone()
    }

    fn set(&mut self, text: String) {
        self.content = Some(text);
    }
}

/// Rope-based buffer for benchmarking
struct BenchBuffer {
    rope: Rope,
}

impl BenchBuffer {
    fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }
}

impl TextOps for BenchBuffer {
    fn line_count(&self) -> u32 {
        self.rope.len_lines() as u32
    }

    fn line_len(&self, line: u32) -> u32 {
        if line >= self.line_count() {
            return 0;
        }
        self.rope
            .line(line as usize)
            .as_str()
            .unwrap_or("")
            .graphemes(true)
            .count() as u32
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
        // Simplified for benchmarking
        let mut result = pos;
        for _ in 0..count {
            result = self.move_right(result, 5);
            if result.col >= self.line_len(result.line) && result.line < self.line_count() - 1 {
                result.line += 1;
                result.col = 0;
            }
        }
        result
    }

    fn prev_word_start(&self, pos: Position, count: u32) -> Position {
        // Simplified for benchmarking
        let mut result = pos;
        for _ in 0..count {
            if result.col == 0 && result.line > 0 {
                result.line -= 1;
                result.col = self.line_len(result.line);
            } else {
                result = self.move_left(result, 5);
            }
        }
        result
    }

    fn next_paragraph_start(&self, pos: Position, _count: u32) -> Position {
        // Find next blank line
        for line in (pos.line + 1)..self.line_count() {
            if self.line_len(line) == 0 {
                return self.line_start(line);
            }
        }
        self.line_start(self.line_count().saturating_sub(1))
    }

    fn prev_paragraph_start(&self, pos: Position, _count: u32) -> Position {
        // Find previous blank line
        for line in (0..pos.line).rev() {
            if self.line_len(line) == 0 {
                return self.line_start(line);
            }
        }
        Position::ZERO
    }

    fn find_in_line(
        &self,
        pos: Position,
        ch: char,
        _before: bool,
        _count: u32,
    ) -> Option<Position> {
        let line_str = self.rope.line(pos.line as usize);
        for (i, grapheme) in line_str
            .as_str()
            .unwrap_or("")
            .graphemes(true)
            .enumerate()
            .skip(pos.col as usize + 1)
        {
            if grapheme.starts_with(ch) {
                return Some(Position {
                    line: pos.line,
                    col: i as u32,
                });
            }
        }
        None
    }

    fn slice_to_string(&self, range: Range) -> String {
        // Simplified implementation
        format!(
            "{}:{} to {}:{}",
            range.start.line, range.start.col, range.end.line, range.end.col
        )
    }

    fn search_forward(&self, from: Position, needle: &str, _wrap: bool) -> Option<Position> {
        // Simplified search
        for line in from.line..self.line_count() {
            if let Some(line_str) = self.rope.line(line as usize).as_str()
                && line_str.contains(needle)
            {
                return Some(self.line_start(line));
            }
        }
        None
    }

    fn search_backward(&self, _from: Position, _needle: &str, _wrap: bool) -> Option<Position> {
        None // Simplified
    }
}

fn generate_sample_text(lines: usize) -> String {
    let mut text = String::new();
    for i in 0..lines {
        text.push_str(&format!(
            "This is line {} with some sample text for benchmarking vim operations.\n",
            i + 1
        ));
        if i % 10 == 0 {
            text.push('\n'); // Add blank lines for paragraphs
        }
    }
    text
}

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: Modifiers::empty(),
    })
}

fn benchmark_simple_movements(c: &mut Criterion) {
    let text = generate_sample_text(1000);
    let buffer = BenchBuffer::new(&text);
    let mut engine = Engine::new();
    let mut clipboard = MockClipboard { content: None };
    let mut cursor = Position::ZERO;

    c.bench_function("simple movements (hjkl)", |b| {
        b.iter(|| {
            let movements = vec!['j', 'j', 'l', 'l', 'h', 'k'];
            for m in &movements {
                let (new_cursor, _) =
                    engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key(*m)));
                cursor = new_cursor;
            }
        });
    });
}

fn benchmark_word_movements(c: &mut Criterion) {
    let text = generate_sample_text(1000);
    let buffer = BenchBuffer::new(&text);
    let mut engine = Engine::new();
    let mut clipboard = MockClipboard { content: None };
    let mut cursor = Position::ZERO;

    c.bench_function("word movements (w/b)", |b| {
        b.iter(|| {
            let movements = vec!['w', 'w', 'w', 'b', 'w'];
            for m in &movements {
                let (new_cursor, _) =
                    engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key(*m)));
                cursor = new_cursor;
            }
        });
    });
}

fn benchmark_delete_operations(c: &mut Criterion) {
    let text = generate_sample_text(1000);
    let buffer = BenchBuffer::new(&text);
    let mut engine = Engine::new();
    let mut clipboard = MockClipboard { content: None };
    let cursor = Position { line: 50, col: 10 };

    c.bench_function("delete operations (dw, dd)", |b| {
        b.iter(|| {
            // Delete word
            let _ = engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('d')));
            let (_, commands) =
                engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('w')));
            black_box(commands);

            // Delete line
            let _ = engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('d')));
            let (_, commands) =
                engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('d')));
            black_box(commands);
        });
    });
}

fn benchmark_visual_selection(c: &mut Criterion) {
    let text = generate_sample_text(1000);
    let buffer = BenchBuffer::new(&text);
    let mut engine = Engine::new();
    let mut clipboard = MockClipboard { content: None };
    let mut cursor = Position { line: 50, col: 10 };

    c.bench_function("visual selection", |b| {
        b.iter(|| {
            // Enter visual mode
            let (new_cursor, _) =
                engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('v')));
            cursor = new_cursor;

            // Move to select
            for _ in 0..5 {
                let (new_cursor, _) =
                    engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('w')));
                cursor = new_cursor;
            }

            // Exit visual mode
            let (new_cursor, _) = engine.handle_event(
                &buffer,
                &mut clipboard,
                cursor,
                black_box(InputEvent::Key(KeyEvent {
                    code: KeyCode::Esc,
                    mods: Modifiers::empty(),
                })),
            );
            cursor = new_cursor;
        });
    });
}

fn benchmark_search_operations(c: &mut Criterion) {
    let text = generate_sample_text(1000);
    let buffer = BenchBuffer::new(&text);
    let mut engine = Engine::new();
    let mut clipboard = MockClipboard { content: None };
    let cursor = Position::ZERO;

    c.bench_function("search operations", |b| {
        b.iter(|| {
            // Enter search mode
            let _ = engine.handle_event(&buffer, &mut clipboard, cursor, black_box(key('/')));

            // Type search query
            for ch in "line".chars() {
                let _ = engine.handle_event(
                    &buffer,
                    &mut clipboard,
                    cursor,
                    black_box(InputEvent::ReceivedChar(ch)),
                );
            }

            // Execute search
            let (_, _) = engine.handle_event(
                &buffer,
                &mut clipboard,
                cursor,
                black_box(InputEvent::Key(KeyEvent {
                    code: KeyCode::Enter,
                    mods: Modifiers::empty(),
                })),
            );
        });
    });
}

fn benchmark_complex_sequence(c: &mut Criterion) {
    let text = generate_sample_text(1000);
    let buffer = BenchBuffer::new(&text);
    let mut engine = Engine::new();
    let mut clipboard = MockClipboard { content: None };
    let mut cursor = Position::ZERO;

    c.bench_function("complex keystroke sequence", |b| {
        b.iter(|| {
            // A realistic editing sequence
            let sequence = vec![
                key('5'),
                key('j'), // Move down 5 lines
                key('w'),
                key('w'), // Move two words
                key('d'),
                key('w'), // Delete word
                key('i'), // Enter insert mode
            ];

            for input in &sequence {
                let (new_cursor, commands) =
                    engine.handle_event(&buffer, &mut clipboard, cursor, black_box(input.clone()));
                cursor = new_cursor;
                black_box(commands);
            }

            // Type some text in insert mode
            for ch in "hello world".chars() {
                let (new_cursor, commands) = engine.handle_event(
                    &buffer,
                    &mut clipboard,
                    cursor,
                    black_box(InputEvent::ReceivedChar(ch)),
                );
                cursor = new_cursor;
                black_box(commands);
            }

            // Exit insert mode
            let (new_cursor, _) = engine.handle_event(
                &buffer,
                &mut clipboard,
                cursor,
                black_box(InputEvent::Key(KeyEvent {
                    code: KeyCode::Esc,
                    mods: Modifiers::empty(),
                })),
            );
            cursor = new_cursor;
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = benchmark_simple_movements,
              benchmark_word_movements,
              benchmark_delete_operations,
              benchmark_visual_selection,
              benchmark_search_operations,
              benchmark_complex_sequence
}
criterion_main!(benches);
