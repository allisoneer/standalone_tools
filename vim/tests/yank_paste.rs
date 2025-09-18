use vim_mini::{
    Engine, InputEvent, KeyCode, KeyEvent,
    traits::Clipboard,
    types::{Command, Position},
};

mod support;
use support::mock_buffer::MockBuffer;
use support::mock_clipboard::MockClipboard;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: vim_mini::key::Modifiers::empty(),
    })
}

#[test]
fn test_yy_yanks_line() {
    let buf = MockBuffer::new("line one\nline two\nline three\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 1, col: 0 };

    // yy on second line
    let (_, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    assert_eq!(cmds.len(), 0); // No commands on first 'y'
    let (_, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    assert_eq!(cmds.len(), 0); // yy doesn't emit commands, just copies

    // Check clipboard content
    assert_eq!(clipboard.get(), Some("line two\n".to_string()));
}

#[test]
fn test_2yy_yanks_two_lines() {
    let buf = MockBuffer::new("line one\nline two\nline three\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // 2yy yanks 2 lines
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('2'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));

    assert_eq!(clipboard.get(), Some("line one\nline two\n".to_string()));
}

#[test]
fn test_yw_yanks_word() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // yw yanks word
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('w'));

    assert_eq!(clipboard.get(), Some("hello ".to_string()));
}

#[test]
fn test_y_dollar_yanks_to_line_end() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 6 }; // at 'w'

    // y$ yanks to end of line
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('$'));

    assert_eq!(clipboard.get(), Some("world".to_string()));
}

#[test]
fn test_visual_yank_charwise() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // Enter visual mode and select "hello"
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('v'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('l')); // at 'o'

    // Yank selection
    let (_, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    assert!(
        cmds.iter()
            .any(|c| matches!(c, Command::SetSelection(None)))
    );

    assert_eq!(clipboard.get(), Some("hello".to_string()));
}

#[test]
fn test_visual_yank_linewise() {
    let buf = MockBuffer::new("line one\nline two\nline three\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // Enter linewise visual mode and select two lines
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('V'));
    let (cur, _) = eng.handle_event(&buf, &mut clipboard, cur, key('j'));

    // Yank selection
    let (_, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    assert!(
        cmds.iter()
            .any(|c| matches!(c, Command::SetSelection(None)))
    );

    assert_eq!(clipboard.get(), Some("line one\nline two\n".to_string()));
}

#[test]
fn test_p_pastes_after_cursor() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Set clipboard content
    clipboard.set("TEST".to_string());

    let cur = Position { line: 0, col: 5 }; // at space
    let (new_cur, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('p'));

    // Should paste after cursor
    assert_eq!(new_cur, Position { line: 0, col: 6 });
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], Command::InsertText { at, text }
        if *at == Position { line: 0, col: 6 } && text == "TEST"));
}

#[test]
fn test_p_pastes_linewise_on_next_line() {
    let buf = MockBuffer::new("line one\nline two\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // First yank a line with yy
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));

    // Now paste with p
    let (new_cur, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('p'));

    // Should paste on next line
    assert_eq!(new_cur, Position { line: 1, col: 0 });
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], Command::InsertText { at, text }
        if *at == Position { line: 1, col: 0 } && text == "line one\n"));
}

#[test]
fn test_count_p_pastes_multiple_times() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Set clipboard content
    clipboard.set("X".to_string());

    let cur = Position { line: 0, col: 5 };

    // 3p should paste 3 times
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('3'));
    let (new_cur, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('p'));

    assert_eq!(new_cur, Position { line: 0, col: 6 });
    assert_eq!(cmds.len(), 3);

    // Check each paste position
    assert!(matches!(&cmds[0], Command::InsertText { at, text }
        if *at == Position { line: 0, col: 6 } && text == "X"));
    assert!(matches!(&cmds[1], Command::InsertText { at, text }
        if *at == Position { line: 0, col: 7 } && text == "X"));
    assert!(matches!(&cmds[2], Command::InsertText { at, text }
        if *at == Position { line: 0, col: 8 } && text == "X"));
}

#[test]
fn test_p_with_empty_clipboard_does_nothing() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 5 };

    // Try to paste with empty clipboard
    let (new_cur, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('p'));

    // Should do nothing
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 0);
}

#[test]
fn test_yank_with_find_motion() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // yfw yanks up to and including 'w'
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('f'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('w'));

    assert_eq!(clipboard.get(), Some("hello w".to_string()));
}

#[test]
fn test_yank_with_till_motion() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // ytw yanks up to but not including 'w'
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('y'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('t'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('w'));

    assert_eq!(clipboard.get(), Some("hello ".to_string()));
}
