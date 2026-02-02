use anyhow::bail;
use regex::Regex;
use std::collections::HashMap;

use crate::tmux::{Handler, Runner, Session, commands::list_sessions};

/// Tmux control session handler
pub struct Control {
    handler: Handler,
    sessions: HashMap<String, Session>,
}

impl Control {
    /// Start new tmux control session
    pub fn start<'a, S, I>(s: S, it: I) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
        I: Iterator<Item = &'a Session>,
    {
        check_version()?;
        let res = Runner::new().args(["-C", "new", "-s", s.as_ref()]).run()?;

        let mut res = Control {
            handler: res,
            sessions: HashMap::from_iter(it.map(|s| {
                let mut s = s.clone();
                s.configured = true;
                (s.name.clone(), s)
            })),
        };
        res.update();
        Ok(res)
    }

    /// Finish tmux session
    pub fn finish(mut self) {
        self.run(&Runner::default().args(["kill-session"]));
        self.handler.wait();
    }

    /// Executes tmux command within control session
    pub fn run(&mut self, command: &Runner) {
        log::trace!("Running 'tmux {}' in control session", command.to_string());
        self.handler.send(command.to_string());
    }

    /// List all sessions (created, attached, configured)
    pub fn sessions(&mut self) -> impl Iterator<Item = &Session> {
        if self.needs_update() {
            self.update()
        }
        self.sessions.values()
    }

    fn needs_update(&self) -> bool {
        let mut res = false;
        loop {
            match self.handler.getline() {
                Some(s) if s == "%sessions-changed" => res = true,
                None => break res,
                _ => (),
            }
        }
    }

    fn update(&mut self) {
        self.sessions.retain(|_, s| s.configured);
        for s in list_sessions() {
            self.sessions.entry(s.name.clone()).or_insert(s).state = s.state;
        }
    }
}

fn check_version() -> anyhow::Result<()> {
    let re = Regex::new(r"^tmux\s+([0-9]+)\.([0-9]+)[a-z]?$").unwrap();
    let version = Runner::default().args(["-V"]).run()?.wait().join("");
    match re.captures(&version) {
        None => bail!("tmux version is ill-formed: '{version}'"),
        Some(caps) => {
            let maj: usize = str::parse(&caps[2]).unwrap();
            let min: usize = str::parse(&caps[1]).unwrap();
            if maj < 3 || min < 2 {
                bail!("Minimum tmux version required 3.2");
            }
        }
    }
    Ok(())
}
