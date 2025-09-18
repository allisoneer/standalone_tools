/// Key codes representing individual keys on the keyboard.
///
/// This enum provides a platform-agnostic representation of keys.
/// Hosts should map their platform-specific key events to these codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// A character key. Hosts should normalize to lowercase for consistency.
    /// For example, 'A' should be mapped to 'a' unless SHIFT is held.
    Char(char),
    /// The Escape key, used to exit modes and cancel operations.
    Esc,
    /// The Enter/Return key.
    Enter,
    /// The Backspace key for deleting characters in insert/search modes.
    Backspace,
    // navigation keys if host prefers: Up, Down, Left, Right (optional)
    // but we primarily use Char('h','j','k','l', ...)
}

bitflags::bitflags! {
    /// Keyboard modifier flags.
    ///
    /// These can be combined to represent multiple modifiers held simultaneously.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const SHIFT = 0b0001;
        const CTRL  = 0b0010;
        const ALT   = 0b0100;
        const META  = 0b1000;
    }
}

/// A key press event with optional modifiers.
///
/// This represents a single key press, including any modifier keys held down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key that was pressed.
    pub code: KeyCode,
    /// Modifier keys held during the key press.
    pub mods: Modifiers,
}

/// Input events that can be processed by the vim engine.
///
/// This enum distinguishes between key presses (used for commands)
/// and text input (used in insert/search modes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// A key press event, typically used for commands and navigation.
    Key(KeyEvent),
    /// A character received in text input mode (insert or search).
    /// This allows hosts to handle composed characters and IME input.
    ReceivedChar(char),
}
