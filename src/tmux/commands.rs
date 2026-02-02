use anyhow::{Result, anyhow};

use crate::{
    logger::LogResult,
    tmux::{Runner, Session, config::State},
};

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
