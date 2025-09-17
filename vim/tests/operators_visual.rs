use vim_mini::{
    Engine, InputEvent, KeyCode, KeyEvent,
    types::{Command, Mode, Position, VisualKind},
};

mod support;
use support::mock_buffer::MockBuffer;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: vim_mini::key::Modifiers::empty(),
    })
}

fn esc() -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Esc,
        mods: vim_mini::key::Modifiers::empty(),
    })
}

#[test]
fn test_dd_deletes_line() {
    let buf = MockBuffer::new("line one\nline two\nline three\n");
    let mut eng = Engine::new();
    let cur = Position { line: 1, col: 0 };

    // dd on second line
    let (_, cmds) = eng.handle_event(&buf, cur, key('d'));
    assert_eq!(cmds.len(), 0); // Operator pending

    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('d'));
    assert_eq!(new_cur.line, 1);
    assert_eq!(new_cur.col, 0);
    assert_eq!(cmds.len(), 1);
    assert!(matches!(cmds[0], Command::Delete { .. }));
}

#[test]
fn test_count_dd_deletes_multiple_lines() {
    let buf = MockBuffer::new("line one\nline two\nline three\nline four\n");
    let mut eng = Engine::new();
    let cur = Position { line: 1, col: 0 };

    // 2dd should delete lines 2 and 3
    eng.handle_event(&buf, cur, key('2'));
    eng.handle_event(&buf, cur, key('d'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('d'));

    assert_eq!(new_cur.line, 1);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.line, 1);
        assert_eq!(range.end.line, 3);
    }
}

#[test]
fn test_x_deletes_character() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('x'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start, cur);
        assert_eq!(range.end.col, 1);
    }
}

#[test]
fn test_count_x_deletes_multiple() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // 3x should delete "hel"
    eng.handle_event(&buf, cur, key('3'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('x'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start, cur);
        assert_eq!(range.end.col, 3);
    }
}

#[test]
fn test_x_at_end_of_line() {
    let buf = MockBuffer::new("hi");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 2 }; // Past last character

    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('x'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 0); // Nothing to delete
}

#[test]
fn test_dh_deletes_left() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 5 };

    eng.handle_event(&buf, cur, key('d'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('h'));
    assert_eq!(new_cur.col, 4);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.col, 4);
        assert_eq!(range.end.col, 5);
    }
}

#[test]
fn test_dl_deletes_right() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    eng.handle_event(&buf, cur, key('d'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('l'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.col, 0);
        assert_eq!(range.end.col, 1);
    }
}

#[test]
fn test_dj_deletes_down() {
    let buf = MockBuffer::new("line one\nline two\nline three");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    eng.handle_event(&buf, cur, key('d'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('j'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.line, 1);
    }
}

#[test]
fn test_d0_deletes_to_line_start() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 5 };

    eng.handle_event(&buf, cur, key('d'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('0'));
    assert_eq!(new_cur.col, 0);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.col, 0);
        assert_eq!(range.end.col, 5);
    }
}

#[test]
fn test_d_dollar_deletes_to_line_end() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 5 };

    eng.handle_event(&buf, cur, key('d'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('$'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 1);
    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.col, 5);
        assert_eq!(range.end.col, 11); // Should include the last character
    }
}

#[test]
fn test_visual_charwise_mode() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    let (_, cmds) = eng.handle_event(&buf, cur, key('v'));
    assert_eq!(cmds.len(), 1);
    if let Command::SetSelection(Some(sel)) = &cmds[0] {
        assert_eq!(sel.start, cur);
        assert_eq!(sel.end, cur);
        assert_eq!(sel.kind, VisualKind::CharWise);
    }
    assert!(matches!(
        eng.snapshot().mode,
        Mode::Visual(VisualKind::CharWise)
    ));
}

#[test]
fn test_visual_linewise_mode() {
    let buf = MockBuffer::new("hello\nworld");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 2 };

    let (_, cmds) = eng.handle_event(&buf, cur, key('V'));
    assert_eq!(cmds.len(), 1);
    if let Command::SetSelection(Some(sel)) = &cmds[0] {
        assert_eq!(sel.start.line, 0);
        assert_eq!(sel.start.col, 0);
        assert_eq!(sel.end.line, 0);
        assert_eq!(sel.kind, VisualKind::LineWise);
    }
}

#[test]
fn test_visual_movement_updates_selection() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Enter visual mode
    eng.handle_event(&buf, cur, key('v'));

    // Move right
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('l'));
    assert_eq!(new_cur.col, 1);
    assert_eq!(cmds.len(), 2); // SetCursor and SetSelection

    let sel_cmd = cmds.iter().find(|c| matches!(c, Command::SetSelection(_)));
    if let Some(Command::SetSelection(Some(sel))) = sel_cmd {
        assert_eq!(sel.start.col, 0);
        assert_eq!(sel.end.col, 1);
    }
}

#[test]
fn test_visual_escape_exits() {
    let buf = MockBuffer::new("hello");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    eng.handle_event(&buf, cur, key('v'));
    let (_, cmds) = eng.handle_event(&buf, cur, esc());
    assert_eq!(cmds.len(), 1);
    assert!(matches!(cmds[0], Command::SetSelection(None)));
    assert_eq!(eng.snapshot().mode, Mode::Normal);
}

#[test]
fn test_visual_v_toggles_off() {
    let buf = MockBuffer::new("hello");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    eng.handle_event(&buf, cur, key('v'));
    let (_, cmds) = eng.handle_event(&buf, cur, key('v'));
    assert_eq!(cmds.len(), 1);
    assert!(matches!(cmds[0], Command::SetSelection(None)));
    assert_eq!(eng.snapshot().mode, Mode::Normal);
}

#[test]
fn test_visual_delete() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Enter visual mode
    eng.handle_event(&buf, cur, key('v'));

    // Move to select "hello"
    let (cur, _) = eng.handle_event(&buf, cur, key('4'));
    let (cur, _) = eng.handle_event(&buf, cur, key('l'));

    // Delete selection
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('d'));
    assert_eq!(new_cur.col, 0);

    // Should have Delete and SetSelection(None) commands
    let del_cmd = cmds.iter().find(|c| matches!(c, Command::Delete { .. }));
    assert!(del_cmd.is_some());

    let sel_cmd = cmds
        .iter()
        .find(|c| matches!(c, Command::SetSelection(None)));
    assert!(sel_cmd.is_some());

    assert_eq!(eng.snapshot().mode, Mode::Normal);
}

#[test]
fn test_visual_line_delete() {
    let buf = MockBuffer::new("line one\nline two\nline three\n");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // Enter visual line mode
    eng.handle_event(&buf, cur, key('V'));

    // Move down to select two lines
    let (cur, _) = eng.handle_event(&buf, cur, key('j'));

    // Delete selection
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('d'));
    assert_eq!(new_cur.line, 0);
    assert_eq!(new_cur.col, 0);

    let del_cmd = cmds.iter().find(|c| matches!(c, Command::Delete { .. }));
    if let Some(Command::Delete { range }) = del_cmd {
        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.line, 2); // Should delete two full lines
    }
}

#[test]
fn test_operator_escape_cancels() {
    let buf = MockBuffer::new("hello");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    eng.handle_event(&buf, cur, key('d'));
    let (_, cmds) = eng.handle_event(&buf, cur, esc());
    assert_eq!(cmds.len(), 0); // No delete should happen
    assert_eq!(eng.snapshot().mode, Mode::Normal);
}

#[test]
fn test_gg_in_visual_mode() {
    let buf = MockBuffer::new("line one\nline two\nline three");
    let mut eng = Engine::new();
    let cur = Position { line: 2, col: 0 };

    // Enter visual mode at last line
    eng.handle_event(&buf, cur, key('v'));

    // gg should move to first line
    eng.handle_event(&buf, cur, key('g'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('g'));
    assert_eq!(new_cur.line, 0);

    // Check selection spans from first to last line
    let sel_cmd = cmds.iter().find(|c| matches!(c, Command::SetSelection(_)));
    if let Some(Command::SetSelection(Some(sel))) = sel_cmd {
        assert_eq!(sel.start.line, 0);
        assert_eq!(sel.end.line, 2);
    }
}

#[test]
fn test_operator_pending_with_count() {
    let buf = MockBuffer::new("hello world");
    let mut eng = Engine::new();
    let cur = Position { line: 0, col: 0 };

    // d3l should delete 3 characters
    eng.handle_event(&buf, cur, key('d'));
    eng.handle_event(&buf, cur, key('3'));
    let (new_cur, cmds) = eng.handle_event(&buf, cur, key('l'));
    assert_eq!(new_cur, cur);
    assert_eq!(cmds.len(), 1);

    if let Command::Delete { range } = &cmds[0] {
        assert_eq!(range.start.col, 0);
        assert_eq!(range.end.col, 3);
    }
}
