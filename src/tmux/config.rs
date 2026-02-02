use serde::{Deserialize, Serialize};
use serde_valid::Validate;

/// Tmux session config
#[derive(Debug, Default, Clone, Serialize, Deserialize, Validate)]
pub struct Session {
    /// Session name
    pub name: String,
    /// Session root path glob pattern
    pub root: String,
    /// List of wintowd
    #[validate(min_items = 1)]
    pub window: Vec<Window>,
    /// session state
    #[serde(skip)]
    pub state: State,
    /// is session present in configuration?
    #[serde(skip)]
    pub configured: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    #[default]
    None,
    Created,
    Attached,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Window {
    /// Window name
    pub name: String,
    /// Window pwd
    #[serde(default)]
    pub root: Option<String>,
    /// Shell command to execute
    #[serde(default)]
    pub command: Option<String>,
    /// Safe shutdown command
    #[serde(default)]
    pub safe_kill: Option<Vec<String>>,
}
