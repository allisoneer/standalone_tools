use vim_mini::{Engine, InputEvent, KeyCode, KeyEvent};

mod support;
use support::mock_buffer::MockBuffer;
use support::mock_clipboard::MockClipboard;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: vim_mini::Modifiers::empty(),
    })
}

fn key_esc() -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Esc,
        mods: vim_mini::Modifiers::empty(),
    })
}

fn key_enter() -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Enter,
        mods: vim_mini::Modifiers::empty(),
    })
}

fn key_backspace() -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Backspace,
        mods: vim_mini::Modifiers::empty(),
    })
}

fn char(c: char) -> InputEvent {
    InputEvent::ReceivedChar(c)
}

#[test]
fn enter_search_mode() {
    let buf = MockBuffer::new("hello world\nfoo bar\nbaz");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 0 };

    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 0);

    let snapshot = eng.snapshot();
    assert!(matches!(snapshot.mode, vim_mini::Mode::SearchPrompt));
}

#[test]
fn search_forward_basic() {
    let buf = MockBuffer::new("hello world\nfoo bar\nbaz world");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 0 };

    // Enter search mode
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));

    // Type "world"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('w'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('o'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('r'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('d'));

    // Press Enter to search
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // Should find "world" on first line
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 6);

    // Back to Normal mode
    let snapshot = eng.snapshot();
    assert!(matches!(snapshot.mode, vim_mini::Mode::Normal));
}

#[test]
fn search_with_n_next() {
    let buf = MockBuffer::new("hello world\nfoo bar\nbaz world");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 0 };

    // Search for "world"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('w'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('o'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('r'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('d'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // First match at line 0, col 6
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 6);

    // Press 'n' to find next match
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('n'));

    // Should find "world" on third line
    assert_eq!(cur.line, 2);
    assert_eq!(cur.col, 4);

    // Press 'n' again - should wrap to first match
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('n'));
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 6);
}

#[test]
fn search_with_capital_n_reverse() {
    let buf = MockBuffer::new("hello world\nfoo bar\nbaz world");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 0 };

    // Search for "world"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('w'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('o'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('r'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('d'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // First match at line 0, col 6
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 6);

    // Press 'N' to find previous match (reverse direction)
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('N'));

    // Should wrap to last match
    assert_eq!(cur.line, 2);
    assert_eq!(cur.col, 4);
}

#[test]
fn search_cancel_with_esc() {
    let buf = MockBuffer::new("hello world\nfoo bar");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 1, col: 2 };

    // Enter search mode
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));

    // Type partial query
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('w'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('o'));

    // Cancel with Esc
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_esc());

    // Should be back at original position
    assert_eq!(cur.line, 1);
    assert_eq!(cur.col, 2);

    // Back to Normal mode
    let snapshot = eng.snapshot();
    assert!(matches!(snapshot.mode, vim_mini::Mode::Normal));

    // 'n' should not work (no last search)
    let (cur2, _) = eng.handle_event(&buf, &mut clipboard, cur, key('n'));
    assert_eq!(cur2, cur);
}

#[test]
fn search_with_backspace() {
    let buf = MockBuffer::new("hello world\nhelp me");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 0 };

    // Enter search mode
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));

    // Type "hello" but then backspace to "hel"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('h'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('e'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('o'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_backspace());
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_backspace());

    // Search for "hel"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // Should find "help" on second line (skipping "hello" at current position)
    assert_eq!(cur.line, 1);
    assert_eq!(cur.col, 0);

    // Press 'n' to find next "hel" (wraps back to "hello")
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('n'));
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 0);
}

#[test]
fn search_not_found() {
    let buf = MockBuffer::new("hello world\nfoo bar");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 5 };

    // Search for something that doesn't exist
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('x'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('y'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('z'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // Should stay at original position
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 5);
}

#[test]
fn search_wrap_around() {
    let buf = MockBuffer::new("first line\nsecond foo\nthird line");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Start on second line
    let cur = vim_mini::Position { line: 1, col: 8 };

    // Search for "line" - should wrap to first line
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('i'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('n'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('e'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // Should wrap around to first line
    assert_eq!(cur.line, 2);
    assert_eq!(cur.col, 6);

    // Next match should wrap to first line
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('n'));
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 6);
}

#[test]
fn search_empty_query() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 5 };

    // Enter search mode and immediately press Enter
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // Should stay at original position
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 5);
}

#[test]
fn search_unicode() {
    let buf = MockBuffer::new("hello 🌍 world\n你好 world");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = vim_mini::Position { line: 0, col: 0 };

    // Search for "world"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('/'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('w'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('o'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('r'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, char('d'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key_enter());

    // Should find "world" after the emoji
    assert_eq!(cur.line, 0);
    assert_eq!(cur.col, 8);

    // Next match
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('n'));
    assert_eq!(cur.line, 1);
    assert_eq!(cur.col, 3);
}
