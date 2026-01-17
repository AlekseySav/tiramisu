use std::process::Command;

use crate::ui::{Session, State};

pub struct Tmux {
    args: Vec<String>,
}

impl Tmux {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn command<S: AsRef<str>, I: IntoIterator<Item = S>>(&mut self, it: I) {
        for s in it {
            let s = s.as_ref();
            if s != "" {
                self.args.push(s.to_string());
            }
        }
        self.args.push(";".to_string());
    }

    pub fn run(self) -> Option<String> {
        log::trace!("tmux {:?}", self.args);
        match Command::new("tmux").args(self.args).output() {
            Ok(r) => {
                let stderr = Self::to_string(r.stderr);
                if !stderr.is_empty() {
                    log::warn!("tmux stderr: {}", stderr);
                }
                if !r.status.success() {
                    match r.status.code() {
                        Some(code) => log::error!("Exited with status code: {code}"),
                        None => log::error!("Process terminated by signal"),
                    }
                    return None;
                }
                Some(Self::to_string(r.stdout))
            }
            Err(e) => {
                log::error!("Failed to run tmux: {}", e);
                None
            }
        }
    }

    pub fn attached() -> bool {
        std::env::var("TMUX").is_ok()
    }

    fn to_string(v: Vec<u8>) -> String {
        match String::from_utf8(v) {
            Ok(s) => s,
            Err(e) => {
                log::error!("tmux returned non-utf8: {:?}", e.into_bytes().as_slice());
                String::new()
            }
        }
    }
}

fn create_session(name: &String, session: &Session) -> bool {
    let mut tmux = Tmux::new();
    let w = &session.windows[0];
    let root = session.root.to_str().unwrap();
    tmux.command([
        "new-session",
        "-d",
        "-s",
        name,
        "-c",
        root,
        "-n",
        &w.name,
        &w.command,
    ]);
    for w in session.windows.iter().skip(1) {
        tmux.command([
            "new-window",
            "-t",
            name,
            "-c",
            root,
            "-n",
            &w.name,
            &w.command,
        ]);
    }
    tmux.run().is_some()
}

pub fn open(name: &String, session: &Session) -> bool {
    if session.state == State::None {
        if !create_session(name, session) {
            return false;
        }
    }

    let mut tmux = Tmux::new();
    if Tmux::attached() {
        tmux.command(["switch-client", "-t", name]);
    } else {
        tmux.command(["attach", "-t", name]);
    }

    tmux.run().is_some()
}

pub fn list_sessions() -> (Vec<String>, Vec<String>) {
    let mut tmux = Tmux::new();
    tmux.command(["ls", "-F", "#{session_name} #{session_attached}"]);
    let res = tmux.run().unwrap_or_default();
    let mut r: (Vec<String>, Vec<String>) = (vec![], vec![]);
    for (name, attached) in res
        .lines()
        .map(|s| s.split(' ').collect())
        .map(|s: Vec<&str>| (String::from(s[0]), s[1].parse::<usize>().unwrap() == 1))
    {
        if attached {
            r.0.push(name);
        } else {
            r.1.push(name);
        }
    }

    r
}
