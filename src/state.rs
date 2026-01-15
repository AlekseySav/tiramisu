use crate::{
    config::{Config, Window},
    controls::{Fzf, Input},
};

#[derive(Debug)]
pub struct Session {
    pub path: String,
    pub window: Vec<Window>,
    pub opened: bool,
    pub attached: bool,
}

/// Full tiramisu state
pub struct State {
    pub config: Config,
    pub fzf: Fzf<Session>,
    pub prompt: Input,
    pub running: bool,
}
