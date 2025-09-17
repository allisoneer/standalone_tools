use vim_mini::types::Position;
use vim_mini::{Engine, InputEvent, KeyCode, KeyEvent};

mod support;
use support::mock_buffer::MockBuffer;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: vim_mini::Modifiers::empty(),
    })
}

#[test]
fn word_forward_basic() {
    let buf = MockBuffer::new("hello world rust\nprogramming is fun");
    let mut eng = Engine::new();
    let mut cur = Position { line: 0, col: 0 };

    // Move to next word "world"
    let (c, _cmds) = eng.handle_event(&buf, cur, key('w'));
    assert_eq!(c, Position { line: 0, col: 6 });
    cur = c;

    // Move to next word "rust"
    let (c, _cmds) = eng.handle_event(&buf, cur, key('w'));
    assert_eq!(c, Position { line: 0, col: 12 });
    cur = c;

    // Move to next line "programming"
    let (c, _cmds) = eng.handle_event(&buf, cur, key('w'));
    assert_eq!(c, Position { line: 1, col: 0 });
}

#[test]
fn word_forward_with_count() {
    let buf = MockBuffer::new("one two three four five");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Move forward 3 words
    let (c, _cmds) = eng.handle_event(&buf, cur, key('3'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 14 }); // at "four"
}

#[test]
fn word_backward_basic() {
    let buf = MockBuffer::new("hello world rust\nprogramming is fun");
    let mut eng = Engine::new();
    let cur = Position { line: 1, col: 15 }; // at 'f' in "fun"

    // Move back to "is"
    let (c, _cmds) = eng.handle_event(&buf, cur, key('b'));
    assert_eq!(c, Position { line: 1, col: 12 });

    // Move back to "programming"
    let (c, _cmds) = eng.handle_event(&buf, c, key('b'));
    assert_eq!(c, Position { line: 1, col: 0 });

    // Move back to previous line "rust"
    let (c, _cmds) = eng.handle_event(&buf, c, key('b'));
    assert_eq!(c, Position { line: 0, col: 12 });
}

#[test]
fn word_with_punctuation() {
    let buf = MockBuffer::new("hello, world! test-case");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // 'w' should stop at punctuation
    let (c, _cmds) = eng.handle_event(&buf, cur, key('w'));
    assert_eq!(c, Position { line: 0, col: 7 }); // at "world"

    let (c, _cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 14 }); // at "test"
}

#[test]
fn paragraph_forward() {
    let buf = MockBuffer::new(
        "First paragraph\nstill first\n\nSecond paragraph\nstill second\n\n\nThird",
    );
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Move to start of second paragraph
    let (c, _cmds) = eng.handle_event(&buf, cur, key('}'));
    assert_eq!(c, Position { line: 3, col: 0 });

    // Move to start of third paragraph
    let (c, _cmds) = eng.handle_event(&buf, c, key('}'));
    assert_eq!(c, Position { line: 7, col: 0 });
}

#[test]
fn paragraph_backward() {
    let buf = MockBuffer::new(
        "First paragraph\nstill first\n\nSecond paragraph\nstill second\n\n\nThird",
    );
    let mut eng = Engine::new();
    let cur = Position { line: 7, col: 0 }; // at "Third"

    // Move to start of second paragraph
    let (c, _cmds) = eng.handle_event(&buf, cur, key('{'));
    assert_eq!(c, Position { line: 3, col: 0 });

    // Move to start of first paragraph
    let (c, _cmds) = eng.handle_event(&buf, c, key('{'));
    assert_eq!(c, Position { line: 0, col: 0 });
}

#[test]
fn find_char_forward() {
    let buf = MockBuffer::new("hello world, this is rust");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Find 'o' (first occurrence)
    let (c, _cmds) = eng.handle_event(&buf, cur, key('f'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('o'));
    assert_eq!(c, Position { line: 0, col: 4 }); // at 'o' in "hello"
}

#[test]
fn find_char_forward_with_count() {
    let buf = MockBuffer::new("hello world, look at those books");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Find 3rd 'o'
    let (c, _cmds) = eng.handle_event(&buf, cur, key('3'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('f'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('o'));
    assert_eq!(c, Position { line: 0, col: 14 }); // at 'o' in "look"
}

#[test]
fn till_char_forward() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Till 'w' (stop before it)
    let (c, _cmds) = eng.handle_event(&buf, cur, key('t'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 5 }); // at space before 'w'
}

#[test]
fn find_char_not_found() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Try to find 'z' which doesn't exist
    let (c, _cmds) = eng.handle_event(&buf, cur, key('f'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('z'));
    assert_eq!(c, cur); // cursor should not move
}

#[test]
fn delete_word() {
    let buf = MockBuffer::new("hello world rust");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Delete word "hello "
    let (c, _cmds) = eng.handle_event(&buf, cur, key('d'));
    let (c, cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 0 });
    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        vim_mini::types::Command::Delete { range } => {
            assert_eq!(range.start, Position { line: 0, col: 0 });
            assert_eq!(range.end, Position { line: 0, col: 6 });
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn delete_paragraph() {
    let buf = MockBuffer::new("First para\n\nSecond para");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Delete to next paragraph
    let (c, _cmds) = eng.handle_event(&buf, cur, key('d'));
    let (c, cmds) = eng.handle_event(&buf, c, key('}'));
    assert_eq!(c, Position { line: 0, col: 0 });
    match &cmds[0] {
        vim_mini::types::Command::Delete { range } => {
            assert_eq!(range.start, Position { line: 0, col: 0 });
            assert_eq!(range.end, Position { line: 2, col: 0 });
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn delete_find() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Delete up to and including 'w'
    let (c, _cmds) = eng.handle_event(&buf, cur, key('d'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('f'));
    let (c, cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 0 });
    match &cmds[0] {
        vim_mini::types::Command::Delete { range } => {
            assert_eq!(range.start, Position { line: 0, col: 0 });
            assert_eq!(range.end, Position { line: 0, col: 7 }); // includes 'w'
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn delete_till() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Delete up to (but not including) 'w'
    let (c, _cmds) = eng.handle_event(&buf, cur, key('d'));
    let (c, _cmds) = eng.handle_event(&buf, c, key('t'));
    let (c, cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 0 });
    match &cmds[0] {
        vim_mini::types::Command::Delete { range } => {
            assert_eq!(range.start, Position { line: 0, col: 0 });
            assert_eq!(range.end, Position { line: 0, col: 5 }); // stops before 'w'
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn visual_word_selection() {
    let buf = MockBuffer::new("hello world rust");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Enter visual mode
    let (c, cmds) = eng.handle_event(&buf, cur, key('v'));
    assert_eq!(cmds.len(), 1);

    // Select to next word
    let (c, cmds) = eng.handle_event(&buf, c, key('w'));
    assert_eq!(c, Position { line: 0, col: 6 });
    assert_eq!(cmds.len(), 2); // SetCursor and SetSelection

    match &cmds[1] {
        vim_mini::types::Command::SetSelection(Some(sel)) => {
            assert_eq!(sel.start, Position { line: 0, col: 0 });
            assert_eq!(sel.end, Position { line: 0, col: 6 });
        }
        _ => panic!("Expected SetSelection command"),
    }
}

#[test]
fn word_motion_at_end_of_buffer() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 6 }; // at 'w'

    // Try to move forward when at last word
    let (c, _cmds) = eng.handle_event(&buf, cur, key('w'));
    assert_eq!(c.line, 0); // should stay on same line
}

#[test]
fn paragraph_motion_with_multiple_blanks() {
    let buf = MockBuffer::new("First\n\n\n\n\nSecond");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Should skip all blank lines
    let (c, _cmds) = eng.handle_event(&buf, cur, key('}'));
    assert_eq!(c, Position { line: 5, col: 0 });
}
