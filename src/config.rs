use crate::env::{env, expand};
use crokey::KeyCombination;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use std::collections::HashMap;
use toml::{Table, Value};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum SessionState {
    #[default]
    None,
    Created,
    Attached,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Config {
    #[serde(flatten)]
    #[validate]
    pub tmux: Tmux,
    #[serde(flatten)]
    pub logger: Logger,
    #[serde(flatten)]
    pub key_bindings: KeyBindings,
    pub theme: Theme,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Tmux {
    pub name: String,
    #[validate(min_items = 1)]
    pub session: Vec<Session>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logger {
    pub messages: Messages,
    pub logger: Logfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    pub level: log::LevelFilter,
    pub ttl_seconds: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logfile {
    pub level: log::LevelFilter,
    pub path: String,
}

/// Tmux session config
#[derive(Debug, Default, Clone, Serialize, Deserialize, Validate)]
pub struct Session {
    /// Session name
    pub name: String,
    /// Session root path glob pattern
    pub root: String,
    /// List of wintowd
    pub window: Vec<Window>,
    /// Session state
    #[serde(skip_deserializing)]
    pub state: SessionState,
    /// Is session present in configuration?
    #[serde(skip_deserializing)]
    #[serde(default = "default_true")]
    pub configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Window {
    /// Window name
    pub name: String,
    /// Window pwd
    #[serde(default)]
    pub root: String,
    /// Shell command to execute
    #[serde(default)]
    pub command: String,
    /// Safe shutdown command
    #[serde(default)]
    pub kill: Vec<String>,
}

/// Tiramisu-specific events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Quit
    #[default]
    Quit,
    /// Show help
    Help,
    /// Select chosen option
    Select,
    /// Horizontal cursor movement
    Cursor {
        offset: isize,
        #[serde(default)]
        absolute: bool,
        #[serde(default)]
        delete: bool,
    },
    /// Move line cursor
    Line { offset: isize },
    /// Pasted or pressed haracters
    Insert(String),
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct EventConfig {
    #[serde(flatten)]
    pub event: Event,
    pub help: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBindings {
    pub keybindings: HashMap<KeyCombination, EventConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Theme {
    prompt: ratatui::style::Style,
}

const DEFAULT_CONFIG: &str = include_str!("../examples/default.toml");

impl Config {
    pub fn new<S: AsRef<str>>(path: Option<S>) -> anyhow::Result<Self> {
        let path = path.map_or(expand("$TIRAMISU_CONFIG_PATH", env)?, |p| p.as_ref().into());
        let config: Value = toml::from_str(&std::fs::read_to_string(path)?)?;
        let default: Value = toml::from_str(DEFAULT_CONFIG)?;
        let config: Config = merge(Some(config), default).try_into()?;
        config.validate()?;
        Ok(config)
    }
}

fn merge(config: Option<Value>, default: Value) -> Value {
    match config {
        Some(config) => {
            if let Value::Table(default) = default
                && let Value::Table(mut config) = config
            {
                Value::Table(Table::from_iter(
                    default
                        .into_iter()
                        .map(|(k, v)| (k.clone(), merge(config.remove(&k), v))),
                ))
            } else {
                config
            }
        }
        None => default,
    }
}

fn default_true() -> bool {
    true
}
