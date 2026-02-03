use std::path::Path;

use anyhow::{Context, Result, anyhow};
use capturing_glob::glob;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use crate::{env, logger::LogResult, tmux::Runner};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    #[default]
    None,
    Created,
    Attached,
}

/// Tmux session config
#[derive(Debug, Default, Clone, Serialize, Deserialize, Validate, Getters, Setters)]
pub struct Session {
    /// Session name
    #[getset(get = "pub")]
    name: String,
    /// Session root path glob pattern
    root: String,
    /// List of wintowd
    #[validate(min_items = 1)]
    window: Vec<Window>,
    /// Session state
    #[serde(skip)]
    #[getset(get = "pub", set = "pub")]
    state: State,
    /// Is session present in configuration?
    #[serde(skip)]
    #[getset(get = "pub", set = "pub")]
    #[serde(default = "default_true")]
    configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct Window {
    /// Window name
    name: String,
    /// Window pwd
    #[serde(default)]
    root: Option<String>,
    /// Shell command to execute
    #[serde(default)]
    command: Option<String>,
    /// Safe shutdown command
    #[serde(default)]
    safe_kill: Option<Vec<String>>,
}

/// Preprocess all sessions from input configuration
pub fn expand_sessions<'a>(
    it: impl Iterator<Item = &'a Session>,
) -> Result<impl Iterator<Item = Session>> {
    // TODO: probably should log all errors
    let mut res = Vec::new();
    for session in it {
        log::trace!("Processing configuration for session '{}'", session.name);
        let mut any = false;
        for entry in glob(&env::expand(&session.root, env::env)?)? {
            match expand_session(&session, &entry?) {
                Ok(s) => {
                    res.push(s);
                    any = true;
                }
                Err(e) => log::error!("Faild to configure session '{}': {e}", session.name),
            }
        }
        if !any {
            log::warn!("Session root '{}' does not match any path", session.root);
        }
    }

    Ok(res.into_iter())
}

fn expand_session(s: &Session, entry: &capturing_glob::Entry) -> Result<Session> {
    fn args(glob: &capturing_glob::Entry, s: &String) -> Result<String> {
        match s.parse::<usize>() {
            Ok(n) => glob
                .group(n)
                .unwrap_or_default()
                .to_str()
                .map(|s| s.to_string())
                .with_context(|| format!("Failed to substitute capture '${n}'")),
            Err(_) => env::env(s),
        }
    }

    let expand = |s| env::expand(s, |s| args(entry, s));
    let expand_or = |s| -> Result<Option<String>> {
        match s {
            Some(s) => Ok(Some(expand(s)?)),
            None => Ok(None),
        }
    };

    Ok(Session {
        name: expand(&s.name)?,
        root: entry
            .path()
            .to_str()
            .with_context(|| format!("Invalid session path: '{:?}'", entry.path()))?
            .to_string(),
        window: s
            .window
            .iter()
            .map(|w| -> Result<Window> {
                Ok(Window {
                    name: expand(&w.name)?,
                    root: expand_or(w.root.as_ref())?,
                    command: expand_or(w.command.as_ref())?,
                    safe_kill: w.safe_kill.clone(),
                })
            })
            .collect::<Result<Vec<Window>>>()?,
        state: s.state,
        configured: s.configured,
    })
}

/// List created sessions
pub fn list_sessions() -> impl Iterator<Item = Session> {
    fn inner(s: &str) -> Result<Option<Session>> {
        let mut s = s.split(':');
        let session = s.next().ok_or(anyhow!("'tmux ls' failed"))?;
        let state = s.next().ok_or(anyhow!("'tmux ls' failed"))?;
        let mut res = Session::default();
        res.name = session.into();
        res.state = match state {
            "1" => State::Attached,
            _ => State::Created,
        };
        Ok(Some(res))
    }

    Runner::new()
        .args(["ls", "-F", "#{session_name}:#{session_attached}"])
        .output()
        .into_iter()
        .map(|s| inner(&s).unwrap_or_log())
        .filter_map(|s| s)
}

/// Switch to session
pub fn switch_to(session: &Session) -> bool {
    // TODO
    true
}

fn default_true() -> bool {
    true
}
