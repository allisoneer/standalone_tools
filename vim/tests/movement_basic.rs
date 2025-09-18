use vim_mini::{
    Engine, InputEvent, KeyCode, KeyEvent, Modifiers,
    types::{Command, Mode, Position},
};
mod support;
use support::mock_buffer::MockBuffer;
use support::mock_clipboard::MockClipboard;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: Modifiers::empty(),
    })
}

fn esc() -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Esc,
        mods: Modifiers::empty(),
    })
}

#[test]
fn hjkl_moves() {
    let buf = MockBuffer::new("abc\nxyz\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let mut cur = Position { line: 0, col: 0 };

    // Move right with l
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    cur = c;
    assert_eq!(cur, Position { line: 0, col: 1 });
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], Command::SetCursor(p) if *p == cur));

    // Move down with j
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('j'));
    cur = c;
    assert_eq!(cur, Position { line: 1, col: 1 });
    assert_eq!(cmds.len(), 1);

    // Move left with h
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('h'));
    cur = c;
    assert_eq!(cur, Position { line: 1, col: 0 });
    assert_eq!(cmds.len(), 1);

    // Move up with k
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('k'));
    cur = c;
    assert_eq!(cur, Position { line: 0, col: 0 });
    assert_eq!(cmds.len(), 1);
}

#[test]
fn zero_and_dollar() {
    let buf = MockBuffer::new("abcdef\nxy\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Start mid-line
    let cur = Position { line: 0, col: 3 };

    // 0 goes to start of line
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('0'));
    assert_eq!(c, Position { line: 0, col: 0 });
    assert_eq!(cmds.len(), 1);

    // $ goes to end of line (last character)
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, c, key('$'));
    assert_eq!(c, Position { line: 0, col: 5 }); // 'f' is at index 5
    assert_eq!(cmds.len(), 1);
}

#[test]
fn g_and_big_g() {
    let buf = MockBuffer::new("line 1\nline 2\nline 3\nline 4");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 2, col: 0 };

    // G goes to last line
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('G'));
    assert_eq!(c.line, 3); // 0-indexed, so line 4 is index 3
    assert_eq!(c.col, 0);
    assert_eq!(cmds.len(), 1);

    // gg goes to first line
    let (c, _) = eng.handle_event(&buf, &mut clipboard, c, key('g'));
    assert_eq!(c.line, 3); // no change yet, g is pending
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, c, key('g'));
    assert_eq!(c.line, 0);
    assert_eq!(c.col, 0);
    assert_eq!(cmds.len(), 1);
}

#[test]
fn counts_with_movements() {
    let buf = MockBuffer::new("0123456789\nabcdefghij\nABCDEFGHIJ\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let mut cur = Position { line: 0, col: 0 };

    // 3l moves right 3
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('3'));
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    assert_eq!(c, Position { line: 0, col: 3 });
    assert_eq!(cmds.len(), 1);
    cur = c;

    // 2j moves down 2
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('2'));
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('j'));
    assert_eq!(c, Position { line: 2, col: 3 });
    assert_eq!(cmds.len(), 1);
    cur = c;

    // 2h moves left 2
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('2'));
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('h'));
    assert_eq!(c, Position { line: 2, col: 1 });
    assert_eq!(cmds.len(), 1);
}

#[test]
fn count_with_g_motions() {
    let buf = MockBuffer::new("line 1\nline 2\nline 3\nline 4\nline 5\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 0 };

    // 3G goes to line 3 (0-indexed: line 2)
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('3'));
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('G'));
    assert_eq!(c.line, 2);
    assert_eq!(cmds.len(), 1);

    // 2gg goes to line 2 (0-indexed: line 1)
    let (_, _) = eng.handle_event(&buf, &mut clipboard, c, key('2'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, c, key('g'));
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, c, key('g'));
    assert_eq!(c.line, 1);
    assert_eq!(cmds.len(), 1);
}

#[test]
fn insert_mode_transitions() {
    let buf = MockBuffer::new("hello world\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // i enters insert mode at current position
    let cur = Position { line: 0, col: 5 }; // at space
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('i'));
    assert_eq!(c, cur); // no movement
    assert!(cmds.is_empty());
    assert!(matches!(eng.snapshot().mode, Mode::Insert));

    // Esc returns to normal mode
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, c, esc());
    assert_eq!(c, cur);
    assert!(cmds.is_empty());
    assert!(matches!(eng.snapshot().mode, Mode::Normal));

    // a enters insert mode after current position
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('a'));
    assert_eq!(c, Position { line: 0, col: 6 });
    assert_eq!(cmds.len(), 1);
    assert!(matches!(eng.snapshot().mode, Mode::Insert));

    let (_, _) = eng.handle_event(&buf, &mut clipboard, c, esc());

    // I enters insert at beginning of line
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, Position { line: 0, col: 5 }, key('I'));
    assert_eq!(c, Position { line: 0, col: 0 });
    assert_eq!(cmds.len(), 1);
    assert!(matches!(eng.snapshot().mode, Mode::Insert));

    let (_, _) = eng.handle_event(&buf, &mut clipboard, c, esc());

    // A enters insert at end of line
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, Position { line: 0, col: 5 }, key('A'));
    assert_eq!(c, Position { line: 0, col: 11 }); // past 'd', ready to append
    assert_eq!(cmds.len(), 1);
    assert!(matches!(eng.snapshot().mode, Mode::Insert));
}

#[test]
fn insert_mode_text_input() {
    let buf = MockBuffer::new("abc\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Enter insert mode
    let cur = Position { line: 0, col: 1 };
    let (c, _) = eng.handle_event(&buf, &mut clipboard, cur, key('i'));

    // Type 'x'
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, c, InputEvent::ReceivedChar('x'));
    assert_eq!(c, Position { line: 0, col: 2 }); // cursor moves after insertion
    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        Command::InsertText { at, text } => {
            assert_eq!(*at, Position { line: 0, col: 1 });
            assert_eq!(text, "x");
        }
        _ => panic!("Expected InsertText command"),
    }
}

#[test]
fn zero_as_motion_vs_count() {
    let buf = MockBuffer::new("0123456789\n");
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();
    let cur = Position { line: 0, col: 5 };

    // 0 alone is a motion to start of line
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('0'));
    assert_eq!(c, Position { line: 0, col: 0 });
    assert_eq!(cmds.len(), 1);

    // 10l is count 10 with motion l
    let cur = Position { line: 0, col: 0 };
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('1'));
    let (_, _) = eng.handle_event(&buf, &mut clipboard, cur, key('0'));
    let (c, cmds) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    assert_eq!(c, Position { line: 0, col: 10 }); // moves to position 10 (past end)
    assert_eq!(cmds.len(), 1);
}

#[test]
fn edge_cases() {
    let buf = MockBuffer::new("x\n\ny\n"); // line with 'x', empty line, line with 'y'
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Moving on empty line
    let cur = Position { line: 1, col: 0 };
    let (c, _) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    assert_eq!(c, cur); // can't move right on empty line

    // Moving up/down preserves column preference
    let cur = Position { line: 0, col: 0 };
    let (c, _) = eng.handle_event(&buf, &mut clipboard, cur, key('j')); // to empty line
    assert_eq!(c, Position { line: 1, col: 0 });
    let (c, _) = eng.handle_event(&buf, &mut clipboard, c, key('j')); // to 'y' line
    assert_eq!(c, Position { line: 2, col: 0 });
}

#[test]
fn unicode_grapheme_handling() {
    let buf = MockBuffer::new("a👍b\né🇺🇸f\n"); // emoji and flag
    let mut eng = Engine::new();
    let mut clipboard = MockClipboard::new();

    // Start at beginning
    let cur = Position { line: 0, col: 0 };

    // Move right past emoji (single grapheme)
    let (c, _) = eng.handle_event(&buf, &mut clipboard, cur, key('l'));
    assert_eq!(c, Position { line: 0, col: 1 }); // at 👍

    let (c, _) = eng.handle_event(&buf, &mut clipboard, c, key('l'));
    assert_eq!(c, Position { line: 0, col: 2 }); // at 'b'

    // Move to next line with flag emoji
    let (c, _) = eng.handle_event(&buf, &mut clipboard, c, key('j'));
    assert_eq!(c, Position { line: 1, col: 2 }); // at flag

    // $ goes to end (last grapheme 'f')
    let (c, _) = eng.handle_event(&buf, &mut clipboard, c, key('$'));
    assert_eq!(c, Position { line: 1, col: 2 }); // at 'f'
}
