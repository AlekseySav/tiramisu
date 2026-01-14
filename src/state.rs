use tui_input::Input;

use crate::{
    config::Config,
    fzf::{self, FzfString},
};

/// Items
pub struct Matches {
    pub items: Vec<FzfString>,
    pub selected: usize,
}

/// Full tiramisu state
pub struct State {
    pub config: Config,
    pub prompt: Input,
    pub matches: Matches,
    pub running: bool,
}

impl State {
    pub fn update_matches(&mut self) {
        self.matches.items =
            fzf::fzf(self.config.session.keys().cloned(), self.prompt.value()).collect();
    }
}
