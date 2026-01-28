use anyhow::{Context, Result, bail};
use std::{
    collections::HashMap,
    io::BufWriter,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use crate::{logger::LogResult, utils::NonBlockLines};

/// Tmux session config
#[derive(Debug, Default)]
pub struct Session {
    pub name: String,
    pub root: String,
    pub windows: Vec<Window>,
    pub created: bool,
    pub attached: bool,
}

/// Tmux window config
#[derive(Debug, Default)]
pub struct Window {
    pub name: String,
    pub layout: String,
    pub panes: Vec<Pane>,
}

/// Tmux pane config
#[derive(Debug, Default)]
pub struct Pane {
    pub root: String,
    pub command: Option<String>,
}

/// Tmux control session client
/// Tracks data about all sessions
#[derive(Debug)]
pub struct Tmux {
    sessions: HashMap<String, Session>,
    client: Option<String>,
    handle: JoinHandle<()>,
}

impl Tmux {
    /// Creates new tmux control session client
    pub fn new<I: IntoIterator<Item = Session>>(it: I) -> Self {
        Self {
            sessions: HashMap::from_iter(it.into_iter().map(|s| (s.name.clone(), s))),
            client: None, /* TODO */
            update: urx,
            finish: fsx,
            handle: tokio::spawn(async move {
                TmuxControl::new(usx, frx).run().await;
            }),
        }
    }

    /// Returns actual list of sessions
    pub async fn sessions(&mut self) -> impl Iterator<Item = &Session> {
        if self.needs_update() {
            self.update().await;
        }
        self.sessions.values()
    }

    /// Switches client to session `session`
    /// On success, returns `true` and closes session
    pub async fn switch_to(&mut self, session: &Session) -> bool {
        /* TODO */
        true
    }

    /// Closes control session
    pub async fn finish(self) {
        self.finish.send(true);
        self.handle.await.unwrap() /* TODO */
    }

    async fn update(&mut self) {
        /* TODO */
    }

    fn needs_update(&mut self) -> bool {
        let updated = self.update.borrow_and_update();
        updated.has_changed() && *updated == true
    }
}

struct TmuxControl {
    update: watch::Sender<bool>,
    finish: watch::Receiver<bool>,
}

impl TmuxControl {
    fn new(update: watch::Sender<bool>, finish: watch::Receiver<bool>) -> Self {
        Self { update, finish }
    }

    async fn run(mut self) {
        while !*self.finish.borrow() {
            if self.updated().await {
                self.update.send(true);
            }
        }
        self.finish().await;
    }

    async fn finish(mut self) {}

    async fn updated(&mut self) -> bool {}
}

#[derive(Default, Debug)]
struct TmuxRunner {
    args: Vec<String>,
}

#[derive(Default)]
struct TmuxHandle {
    cmd: String,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout: Option<NonBlockLines<ChildStdout>>,
    stderr: Option<NonBlockLines<ChildStdout>>,
    child: Option<Child>,
}

impl TmuxRunner {
    /// Adds parameters to command
    pub fn with<S: AsRef<str>, I: IntoIterator<Item = S>>(it: I) -> Self {
        Self {
            args: Vec::from_iter(it.into_iter().map(|s| s.as_ref().into())),
        }
    }

    /// Adds parameters to command
    pub fn add<S: AsRef<str>, I: IntoIterator<Item = S>>(&mut self, it: I) -> &mut Self {
        self.args.extend(it.into_iter().map(|s| s.as_ref().into()));
        self
    }

    /// Executes command
    pub async fn run(&mut self) -> TmuxHandle {
        self.run_with(
            Command::new("tmux")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped()),
        )
    }

    /// Executes command with inherited stdin/stdout
    pub fn run_detached(&mut self) -> TmuxHandle {
        self.run_with(&mut Command::new("tmux"))
    }

    fn run_with(&mut self, cmd: &mut Command) -> TmuxHandle {
        let command = self.args.join(" ");
        log::debug!("Running \"tmux {}\"", command);
        cmd.args(&self.args).stderr(Stdio::piped());
        cmd.spawn()
            .map(|mut c| TmuxHandle {
                cmd: command.clone(),
                stdin: c.stdin.take().map(|c| BufWriter::new(c)),
                stdout: c.stdout.take().map(|c| NonBlockLines::new(c)),
                stderr: c.stderr.take().map(|c| NonBlockLines::new(c)),
                child: Some(c),
            })
            .with_context(|| format!("Failed to run 'tmux {command}'"))
            .unwrap_or_log()
    }

    // /// Executes command in control session `control`
    // pub async fn run_in(&mut self, control: &mut TmuxHandle) -> Result<()> {
    //     let command = self.args.join(" ");
    //     log::debug!("Sending \"{}\" to \"tmux {}\"", command, control.cmd);

    //     match &mut control.stdin {
    //         Some(w) => w
    //             .write_all(command.as_bytes())
    //             .await
    //             .and(w.write_u8(b'\n').await)
    //             .and(w.flush().await)
    //             .with_context(|| format!("Failed to run 'tmux {command}'"))?,
    //         None => (),
    //     }
    //     Ok(())
    // }
}

fn stderr_handler(stderr: ChildStderr) -> JoinHandle<()> {
    let mut r = BufReader::new(stderr).lines();
    tokio::spawn(async move {
        loop {
            match r.next_line().await {
                Ok(None) => break,
                Ok(Some(s)) => log::warn!("tmux stderr: {}", s),
                Err(err) => log::warn!("tmux stderr is corrupted: {}", err),
            }
        }
    })
}
