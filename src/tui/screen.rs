use crate::{config, logger};

pub struct Screen {
    theme: config::Theme,
}

impl Screen {
    pub fn new(theme: config::Theme) -> Self {
        Self { theme }
    }

    pub fn render<'a, S, I, M>(&self, prompt: S, sessions: I, messages: M)
    where
        S: AsRef<str>,
        I: ExactSizeIterator<Item = &'a config::Session>,
        M: Iterator<Item = &'a logger::Message>,
    {
    }
}
