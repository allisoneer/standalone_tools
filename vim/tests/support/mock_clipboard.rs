use vim_mini::traits::Clipboard;

#[derive(Default, Debug, Clone)]
pub struct MockClipboard {
    content: Option<String>,
}

impl MockClipboard {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Clipboard for MockClipboard {
    fn get(&mut self) -> Option<String> {
        self.content.clone()
    }

    fn set(&mut self, text: String) {
        self.content = Some(text);
    }
}
