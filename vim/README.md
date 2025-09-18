# vim_mini

A minimal, high-performance Vim-like modal input engine for Rust applications.

[![Crates.io](https://img.shields.io/crates/v/vim_mini.svg)](https://crates.io/crates/vim_mini)
[![Documentation](https://docs.rs/vim_mini/badge.svg)](https://docs.rs/vim_mini)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

`vim_mini` provides a clean, trait-based API for adding Vim-style modal editing to any Rust application. It handles the complex state machine of modal editing while letting you maintain complete control over your text storage and rendering.

## Features

- 🚀 **High Performance**: Zero-allocation design with <5ms keystroke latency
- 🔤 **Unicode Support**: Full grapheme cluster awareness for emoji and international text
- 🎯 **Minimal API**: Clean trait-based design that's easy to integrate
- 🧩 **Platform Agnostic**: Works with any UI framework or terminal library
- ✨ **Pure Rust**: No dependencies on external editors or binaries

### Supported Vim Features

- **Modes**: Normal, Insert, Visual (character/line), Search
- **Motions**: `h j k l`, `w b`, `0 $`, `gg G`, `{ }`, `f/t<char>` with counts
- **Operators**: `d` (delete), `y` (yank), `x`, `p` (paste)
- **Visual Mode**: `v` (character-wise), `V` (line-wise)
- **Search**: `/` forward search, `n`/`N` navigation
- **Counts**: Prefix commands with numbers (e.g., `5j`, `3dw`)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vim_mini = "0.1"
```

Basic usage:

```rust
use vim_mini::{Engine, InputEvent, KeyCode, KeyEvent};
use vim_mini::traits::{TextOps, Clipboard};
use vim_mini::types::{Position, Command};

// Implement TextOps for your buffer
struct MyBuffer { /* ... */ }
impl TextOps for MyBuffer { /* ... */ }

// Implement Clipboard
struct MyClipboard { /* ... */ }
impl Clipboard for MyClipboard { /* ... */ }

// Create engine and process keystrokes
let mut engine = Engine::new();
let mut buffer = MyBuffer::new();
let mut clipboard = MyClipboard::new();
let mut cursor = Position::ZERO;

// Handle a keystroke
let input = InputEvent::Key(KeyEvent {
    code: KeyCode::Char('j'),
    mods: Default::default()
});
let (new_cursor, commands) = engine.handle_event(&buffer, &mut clipboard, cursor, input);

// Apply the commands
for cmd in commands {
    match cmd {
        Command::SetCursor(pos) => cursor = pos,
        Command::Delete { range } => { /* delete text */ },
        Command::InsertText { at, text } => { /* insert text */ },
        Command::SetSelection(sel) => { /* update selection */ },
    }
}
```

## Integration Guide

### 1. Implement the Required Traits

#### TextOps

The `TextOps` trait defines how the engine interacts with your text buffer:

```rust
impl TextOps for MyBuffer {
    fn line_count(&self) -> u32 { /* ... */ }
    fn line_len(&self, line: u32) -> u32 { /* ... */ }
    fn move_left(&self, pos: Position, count: u32) -> Position { /* ... */ }
    fn move_right(&self, pos: Position, count: u32) -> Position { /* ... */ }
    // ... other required methods
}
```

**Important**: All position calculations must be grapheme-aware (not byte or char based) to correctly handle Unicode text.

#### Clipboard

The `Clipboard` trait provides copy/paste functionality:

```rust
impl Clipboard for MyClipboard {
    fn get(&mut self) -> Option<String> {
        // Return clipboard contents
    }

    fn set(&mut self, text: String) {
        // Store text in clipboard
    }
}
```

### 2. Map Platform Events

Convert your platform's key events to vim_mini's `InputEvent`:

```rust
// In normal/visual modes, use Key events
let vim_event = InputEvent::Key(KeyEvent {
    code: KeyCode::Char('h'),
    mods: Modifiers::empty(),
});

// In insert/search modes, use ReceivedChar for text input
let vim_event = InputEvent::ReceivedChar('a');
```

### 3. Process Commands

The engine returns commands that you need to apply to your buffer:

```rust
match command {
    Command::SetCursor(pos) => {
        // Update cursor position
    }
    Command::SetSelection(Some(sel)) => {
        // Highlight selected text
    }
    Command::Delete { range } => {
        // Remove text from range.start to range.end
    }
    Command::InsertText { at, text } => {
        // Insert text at the specified position
    }
    _ => {}
}
```

### 4. Display Mode Information

Use `engine.snapshot()` to get the current state for your UI:

```rust
let snapshot = engine.snapshot();
let mode_text = match snapshot.mode {
    Mode::Normal => "NORMAL",
    Mode::Insert => "INSERT",
    Mode::Visual(_) => "VISUAL",
    Mode::SearchPrompt => "SEARCH",
};
```

## Examples

See the `examples/` directory for complete integration examples:

- `tui_crossterm.rs` - Terminal UI with crossterm and ratatui
- `egui_app.rs` - GUI application with egui

Run an example:

```bash
cargo run --example tui_crossterm
```

## Performance

The library is designed for high performance with:

- Zero allocations in the hot path
- Efficient state machine implementation
- Benchmark suite showing <5ms latency for complex operations

Run benchmarks:

```bash
cargo bench
```

## Optional Features

- `clipboard` - System clipboard integration using arboard (off by default)

```toml
[dependencies]
vim_mini = { version = "0.1", features = ["clipboard"] }
```

## Design Philosophy

vim_mini follows a strict separation of concerns:

- **The library handles**: Modal state, key sequence parsing, motion calculations
- **Your application handles**: Text storage, rendering, undo/redo, file I/O

This design allows you to integrate vim-style editing into any application without changing your existing architecture.

## What's NOT Included

To keep the library minimal and focused:

- No dot-repeat (`.`)
- No macros or registers (except system clipboard)
- No ex commands (`:`)
- No marks or jumplists
- No undo/redo (implement in your application)
- No syntax highlighting or rendering

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Inspired by the elegant modal editing of Vim and the need for a lightweight, embeddable solution for Rust applications.