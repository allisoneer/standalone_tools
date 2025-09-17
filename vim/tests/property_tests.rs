use proptest::prelude::*;
use vim_mini::types::{Command, Position};
use vim_mini::{Engine, InputEvent, KeyCode, KeyEvent, Modifiers, TextOps};

mod support;
use support::mock_buffer::MockBuffer;

fn key(c: char) -> InputEvent {
    InputEvent::Key(KeyEvent {
        code: KeyCode::Char(c),
        mods: Modifiers::empty(),
    })
}

// Strategy for generating text content with various edge cases
fn text_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty text
        Just("".to_string()),
        // Single line
        "[a-zA-Z0-9 .!?,;:\\-_]{0,50}",
        // Multiple lines with normal text
        "[a-zA-Z0-9 .!?,;:\\-_\n]{0,200}",
        // Text with blank lines (for paragraph tests)
        r"[a-zA-Z0-9 ]{0,20}\n\n[a-zA-Z0-9 ]{0,20}",
        // Unicode text
        "[\u{0020}-\u{007E}\u{00A0}-\u{00FF}\u{4E00}-\u{9FFF}\u{1F600}-\u{1F64F}\n]{0,100}",
        // Lines with only whitespace
        "[ \t]{0,10}\n[ \t]{0,10}\n[a-z]{0,10}",
    ]
}

// Strategy for generating motion characters
fn motion_char_strategy() -> impl Strategy<Value = char> {
    prop_oneof![
        Just('h'),
        Just('j'),
        Just('k'),
        Just('l'),
        Just('0'),
        Just('$'),
        Just('w'),
        Just('b'),
        Just('{'),
        Just('}'),
        Just('g'),
        Just('G'),
    ]
}

// Strategy for generating find/till targets
fn find_char_strategy() -> impl Strategy<Value = char> {
    prop_oneof![
        any::<char>().prop_filter("printable ASCII", |c| c.is_ascii_graphic()),
        prop::char::range('a', 'z'),
        prop::char::range('0', '9'),
    ]
}

proptest! {
    #[test]
    fn motion_never_panics(
        text in text_strategy(),
        motion in motion_char_strategy(),
        count in 0u32..100,
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        // Start from origin
        let start = Position { line: 0, col: 0 };

        // Apply count if > 0
        if count > 0 && count <= 9 {
            for digit in count.to_string().chars() {
                let _ = eng.handle_event(&buf, start, key(digit));
            }
        }

        // Apply motion - should not panic
        let (new_pos, _cmds) = eng.handle_event(&buf, start, key(motion));

        // Verify position is within bounds
        assert!(new_pos.line < buf.line_count() || (new_pos.line == 0 && buf.line_count() == 0));
        if new_pos.line < buf.line_count() {
            assert!(new_pos.col <= buf.line_len(new_pos.line));
        }
    }

    #[test]
    fn motion_from_any_position_never_panics(
        text in text_strategy(),
        start_line in 0u32..50,
        start_col in 0u32..50,
        motion in motion_char_strategy(),
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let start = Position { line: start_line, col: start_col };

        // Motion from potentially invalid position should not panic
        let (new_pos, _cmds) = eng.handle_event(&buf, start, key(motion));

        // Result should be valid
        assert!(new_pos.line < buf.line_count() || (new_pos.line == 0 && buf.line_count() == 0));
    }

    #[test]
    fn find_char_never_panics(
        text in text_strategy(),
        target in find_char_strategy(),
        before in any::<bool>(),
        count in 1u32..10,
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let start = Position { line: 0, col: 0 };

        // Set count
        if count > 1 {
            let _ = eng.handle_event(&buf, start, key(char::from_digit(count, 10).unwrap_or('1')));
        }

        // f or t
        let find_key = if before { 't' } else { 'f' };
        let _ = eng.handle_event(&buf, start, key(find_key));

        // Target character - should not panic even if not found
        let (new_pos, _cmds) = eng.handle_event(&buf, start, key(target));

        // Position should be valid
        assert!(new_pos.line < buf.line_count() || (new_pos.line == 0 && buf.line_count() == 0));
    }

    #[test]
    fn delete_motion_produces_valid_ranges(
        text in text_strategy(),
        motion in motion_char_strategy(),
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let start = Position { line: 0, col: 0 };

        // Enter delete operator
        let _ = eng.handle_event(&buf, start, key('d'));

        // Apply motion
        let (_pos, cmds) = eng.handle_event(&buf, start, key(motion));

        // Check all delete commands have valid ranges
        for cmd in cmds {
            if let Command::Delete { range } = cmd {
                // Start should be <= end
                assert!(range.start.line <= range.end.line ||
                       (range.start.line == range.end.line && range.start.col <= range.end.col));

                // Both positions should be valid (or at end for deletion)
                let max_line = buf.line_count();
                assert!(range.start.line <= max_line);
                assert!(range.end.line <= max_line);
            }
        }
    }

    #[test]
    fn word_motion_handles_unicode(
        prefix in "[a-z]{0,10}",
        emoji in "[\u{1F600}-\u{1F64F}]{1,3}",
        suffix in "[a-z]{0,10}",
    ) {
        let text = format!("{} {} {}", prefix, emoji, suffix);
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let start = Position { line: 0, col: 0 };

        // Move word forward - should handle emoji correctly
        let (pos1, _) = eng.handle_event(&buf, start, key('w'));

        // Move word forward again
        let (pos2, _) = eng.handle_event(&buf, pos1, key('w'));

        // Both positions should be valid
        assert!(pos1.line < buf.line_count());
        assert!(pos2.line < buf.line_count());
        assert!(pos1.col <= buf.line_len(pos1.line));
        assert!(pos2.col <= buf.line_len(pos2.line));
    }

    #[test]
    fn paragraph_motion_with_many_blanks(
        blank_lines in 0usize..10,
        text_lines in 0usize..5,
    ) {
        let mut lines = vec!["First paragraph"];
        lines.extend(vec![""; blank_lines]);
        lines.extend(vec!["Second paragraph"; text_lines.max(1)]);

        let text = lines.join("\n");
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let start = Position { line: 0, col: 0 };

        // Move to next paragraph - should not panic with any number of blanks
        let (new_pos, _) = eng.handle_event(&buf, start, key('}'));

        assert!(new_pos.line < buf.line_count());
    }

    #[test]
    fn large_counts_dont_panic(
        text in "[a-z \n]{10,100}",
        motion in motion_char_strategy(),
        count in 10u32..9999,
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let start = Position { line: 0, col: 0 };

        // Apply large count (only use first 2 digits to avoid overflow)
        let count_str = count.to_string();
        for digit in count_str.chars().take(2) {
            let _ = eng.handle_event(&buf, start, key(digit));
        }

        // Motion with large count should clamp, not panic
        let (new_pos, _) = eng.handle_event(&buf, start, key(motion));

        assert!(new_pos.line < buf.line_count() || (new_pos.line == 0 && buf.line_count() == 0));
    }

    #[test]
    fn visual_mode_motions_never_panic(
        text in text_strategy(),
        motions in prop::collection::vec(motion_char_strategy(), 1..5),
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let mut pos = Position { line: 0, col: 0 };

        // Enter visual mode
        let (p, _) = eng.handle_event(&buf, pos, key('v'));
        pos = p;

        // Apply series of motions
        for motion in motions {
            let (p, cmds) = eng.handle_event(&buf, pos, key(motion));
            pos = p;

            // Check selection is valid
            for cmd in cmds {
                if let Command::SetSelection(Some(sel)) = cmd {
                    assert!(sel.start.line < buf.line_count() || buf.line_count() == 0);
                    assert!(sel.end.line < buf.line_count() || buf.line_count() == 0);
                }
            }
        }
    }

    #[test]
    fn motion_sequence_never_panics(
        text in text_strategy(),
        motions in prop::collection::vec(motion_char_strategy(), 0..10),
    ) {
        let buf = MockBuffer::new(&text);
        let mut eng = Engine::new();

        let mut pos = Position { line: 0, col: 0 };

        // Apply sequence of motions
        for motion in motions {
            let (p, _) = eng.handle_event(&buf, pos, key(motion));
            pos = p;
            assert!(pos.line < buf.line_count() || (pos.line == 0 && buf.line_count() == 0));
        }
    }
}

// Specific edge case tests
#[test]
fn empty_buffer_motions() {
    let buf = MockBuffer::new("");
    let mut eng = Engine::new();
    let pos = Position { line: 0, col: 0 };

    // All motions should handle empty buffer gracefully
    for motion in ['h', 'j', 'k', 'l', 'w', 'b', '{', '}', '0', '$', 'G'] {
        let (new_pos, _) = eng.handle_event(&buf, pos, key(motion));
        assert_eq!(new_pos, Position { line: 0, col: 0 });
    }
}

#[test]
fn single_char_buffer_motions() {
    let buf = MockBuffer::new("x");
    let mut eng = Engine::new();
    let pos = Position { line: 0, col: 0 };

    // Test all motions work on minimal buffer
    let cases = [
        ('h', Position { line: 0, col: 0 }),
        ('l', Position { line: 0, col: 1 }),
        ('w', Position { line: 0, col: 0 }), // 'w' on single char stays at start
        ('b', Position { line: 0, col: 0 }),
        ('$', Position { line: 0, col: 0 }),
    ];

    for (motion, expected) in cases {
        let (new_pos, _) = eng.handle_event(&buf, pos, key(motion));
        assert_eq!(new_pos, expected, "Motion '{}' failed", motion);
    }
}
