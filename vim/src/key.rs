#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char), // lowercase mapped by host or engine (we'll normalize)
    Esc,
    Enter,
    Backspace,
    // navigation keys if host prefers: Up, Down, Left, Right (optional)
    // but we primarily use Char('h','j','k','l', ...)
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const SHIFT = 0b0001;
        const CTRL  = 0b0010;
        const ALT   = 0b0100;
        const META  = 0b1000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub mods: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Key(KeyEvent),
    ReceivedChar(char), // Insert mode text input
}
