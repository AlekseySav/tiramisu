use anyhow::{Context, Result, anyhow};
use capturing_glob::glob;

use crate::{config, env, logger::LogResult, tmux::Runner};

/// Preprocess all sessions from input configuration
pub fn expand_sessions<'a>(
    it: impl Iterator<Item = &'a config::Session>,
) -> Result<impl Iterator<Item = config::Session>> {
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

fn expand_session(s: &config::Session, entry: &capturing_glob::Entry) -> Result<config::Session> {
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

    Ok(config::Session {
        name: expand(&s.name)?,
        root: entry
            .path()
            .to_str()
            .with_context(|| format!("Invalid session path: '{:?}'", entry.path()))?
            .to_string(),
        window: s
            .window
            .iter()
            .map(|w| -> Result<config::Window> {
                Ok(config::Window {
                    name: expand(&w.name)?,
                    root: expand(&w.root)?,
                    command: expand(&w.command)?,
                    kill: w.kill.iter().map(|s| expand(s)).collect::<Result<_>>()?,
                })
            })
            .collect::<Result<Vec<config::Window>>>()?,
        state: s.state,
        configured: s.configured,
    })
}

/// List created sessions
pub fn list_sessions() -> impl Iterator<Item = config::Session> {
    fn inner(s: &str) -> Result<Option<config::Session>> {
        let mut s = s.split(':');
        let session = s.next().ok_or(anyhow!("'tmux ls' failed"))?;
        let state = s.next().ok_or(anyhow!("'tmux ls' failed"))?;
        let mut res = config::Session::default();
        res.name = session.into();
        res.state = match state {
            "1" => config::SessionState::Attached,
            _ => config::SessionState::Created,
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
pub fn switch_to(_session: &config::Session) -> bool {
    // TODO
    true
}

fn default_true() -> bool {
    true
}
