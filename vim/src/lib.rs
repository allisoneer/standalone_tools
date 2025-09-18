//! # vim_mini - A minimal, high-performance Vim-like modal input engine
//!
//! `vim_mini` is a platform-agnostic Rust library that provides a minimal but complete
//! implementation of Vim-like modal editing. It interprets keystrokes, manages modal state,
//! and emits commands that host applications can apply to their own text buffers.
//!
//! ## Design Philosophy
//!
//! This library follows a strict separation of concerns:
//! - **The library handles**: Modal logic, key interpretation, motion resolution, selection semantics
//! - **The host handles**: Text storage, rendering, undo/redo, file I/O, syntax highlighting
//!
//! ## Key Features
//!
//! - **Modes**: Normal, Insert, Visual (character/line), and Search
//! - **Motions**: `h j k l`, `w b`, `0 $`, `gg G`, `{ }`, `f/t<char>` with counts
//! - **Operators**: `d` (delete), `y` (yank), `x` (delete char), `p` (paste)
//! - **Visual Mode**: Character-wise (`v`) and line-wise (`V`) selection
//! - **Search**: Forward search with `/`, navigate with `n`/`N`
//! - **Unicode-aware**: All operations work correctly with grapheme clusters (emoji, combining marks)
//! - **High Performance**: Zero-allocation design, <5ms keystroke latency
//!
//! ## Quick Start
//!
//! ```no_run
//! use vim_mini::{Engine, InputEvent, KeyCode, KeyEvent};
//! use vim_mini::traits::{TextOps, Clipboard};
//! use vim_mini::types::{Position, Command};
//!
//! // Your text buffer implementation
//! struct MyBuffer { /* ... */ }
//! impl TextOps for MyBuffer { /* ... */ }
//!
//! // Your clipboard implementation
//! struct MyClipboard { /* ... */ }
//! impl Clipboard for MyClipboard { /* ... */ }
//!
//! // Create engine and process keystrokes
//! let mut engine = Engine::new();
//! let mut buffer = MyBuffer::new();
//! let mut clipboard = MyClipboard::new();
//! let mut cursor = Position::ZERO;
//!
//! // Handle a keystroke
//! let input = InputEvent::Key(KeyEvent {
//!     code: KeyCode::Char('j'),
//!     mods: Default::default()
//! });
//! let (new_cursor, commands) = engine.handle_event(&buffer, &mut clipboard, cursor, input);
//!
//! // Apply the commands to your buffer
//! for cmd in commands {
//!     match cmd {
//!         Command::SetCursor(pos) => { /* update cursor */ },
//!         Command::Delete { range } => { /* delete text */ },
//!         Command::InsertText { at, text } => { /* insert text */ },
//!         Command::SetSelection(sel) => { /* update selection */ },
//!     }
//! }
//! ```
//!
//! ## Integration Guide
//!
//! To integrate vim_mini into your application:
//!
//! 1. **Implement the `TextOps` trait** for your text buffer
//! 2. **Implement the `Clipboard` trait** for clipboard access
//! 3. **Map platform key events** to `InputEvent`
//! 4. **Apply emitted commands** to update your text buffer
//! 5. **Render the current mode and selection** in your UI
//!
//! See the examples directory for complete integration examples with terminal and GUI applications.
//!
//! ## What's NOT Included
//!
//! To keep the library minimal and focused:
//! - No dot-repeat (`.`)
//! - No macros or registers (except system clipboard)
//! - No ex commands (`:`)
//! - No marks or jumplists
//! - No text objects beyond basic word/line
//! - No undo/redo (hosts should implement this)
//!
//! ## Performance
//!
//! The library is designed for high performance:
//! - Zero allocations in the hot path
//! - Trait-based API allows hosts to optimize text operations
//! - Benchmarks show <5ms median latency for complex operations
//!
//! ## Examples
//!
//! See the `examples/` directory for:
//! - `tui_crossterm.rs` - Terminal integration with crossterm
//! - `egui_app.rs` - GUI integration with egui

pub mod engine;
pub mod key;
pub mod traits;
pub mod types;

pub use crate::engine::{Engine, EngineBuilder, EngineSnapshot};
pub use crate::key::{InputEvent, KeyCode, KeyEvent, Modifiers};
pub use crate::traits::{Clipboard, TextOps};
pub use crate::types::{Command, Mode, Position, Range, Selection, VisualKind};
