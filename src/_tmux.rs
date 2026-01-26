use anyhow::Result;
use runner::{Handle, Runner};
use tokio::{
    sync::watch::{self, Receiver, Sender},
    task::JoinHandle,
};

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

/// Tmux control session
/// Stores actual data about all configured sessions
#[derive(Debug)]
pub struct Tmux {
    sessions: Vec<Session>,
    client: Option<String>,
    control: TmuxControl,
    updated: Receiver<bool>,
}

impl Tmux {
    /// Creates new tmux control session
    pub async fn new() -> Result<Self> {
        Runner::default().add(["-V"]).run()?.output().await?;
        let client = match std::env::var("TMUX") {
            Ok(_) => Runner::default()
                .add(["display-message", "-p", "#{client_tty}"])
                .run()?
                .output()
                .await?
                .get(0)
                .cloned(),
            Err(_) => None,
        };
        log::debug!("tmux client {client:?}");
        let (control, updated) = TmuxControl::new().await?;

        Ok(Self {
            sessions: Vec::new(),
            client,
            control,
            updated,
        })
    }

    /// Waits until tmux control session ends
    pub async fn wait(mut self) -> Result<()> {
        Runner::default()
            .add(["kill-session"])
            .run_in(&mut self.control)
            .await?;
        self.update().await?;
        self.control.wait().await
    }

    /// Register new session `s`
    pub fn add(&mut self, s: Session) {
        self.sessions.push(s);
    }

    /// Lists all active sessions
    pub fn sessions(&self) -> impl Iterator<Item = &Session> {
        self.sessions.iter()
    }

    /// Switches to new session
    pub async fn switch(&mut self, session: &Session) -> Result<()> {
        if !session.created {
            create_session(session).await?;
        }
        match &self.client {
            Some(client) => Runner::default()
                .add(["switch-client", "-c", client, "-t", &session.name])
                .run()?,
            None => Runner::detached()
                .add(["attach", "-t", &session.name])
                .run()?,
        }
        .wait()
        .await?;
        Ok(())
    }

    /// Actualize sessions info
    pub async fn update(&mut self) -> Result<()> {
        loop {
            match self.control.next_line().await? {
                Some(s) => {
                    log::trace!("received event: {s}");
                    if s == "%sessions-changed" {
                        self.poll_sessions().await?;
                    }
                }
                None => return Ok(()),
            }
        }
    }

    async fn poll_sessions(&mut self) -> Result<()> {
        let res = Runner::default()
            .add(["ls", "-F", "#{session_name}:#{session_attached}"])
            .run()?
            .output()
            .await?;
        // TODO update

        Ok(())
    }
}

struct TmuxControl {
    control: Handle,
    updated: Sender<bool>,
}

impl TmuxControl {
    /// Creates new tmux control session
    async fn new(
        updated: Sender<bool>,
        shutdown: Receiver<bool>,
    ) -> Result<JoinHandle<Result<()>>> {
        let (sx, rx) = watch::channel(false);
        Ok((
            Self {
                control: Runner::default().add(["-C", "new"]).run()?,
                updated: sx,
            },
            rx,
        ))
    }

    /// Waits until tmux control session ends
    async fn wait(mut self) -> Result<()> {
        Runner::default()
            .add(["kill-session"])
            .run_in(&mut self.control)
            .await?;
        self.update().await?;
        self.control.wait().await
    }

    /// Actualize sessions info
    async fn update(&mut self) -> Result<()> {
        loop {
            match self.control.next_line().await? {
                Some(s) => {
                    log::trace!("received event: {s}");
                    if s == "%sessions-changed" {
                        self.updated.send(true)?;
                    }
                }
                None => return Ok(()),
            }
        }
    }
}

async fn create_session(session: &Session) -> Result<()> {
    let mut r = Runner::default();
    r.add([
        "new-session",
        "-d",
        "-s",
        &session.name,
        "-c",
        &session.root,
    ]);
    for (i, window) in session.windows.iter().enumerate() {
        if i != 0 {
            r.add(["new-window", "-t", &session.name]);
        }
        r.add(["-n", &window.name]);
        let target = format!("{}:{}", session.name, i);
        for (n, pane) in window.panes.iter().enumerate() {
            if n != 0 {
                r.add(["split-window", "-t", &target]);
            }
            match &pane.command {
                Some(c) => r.add(["-c", &pane.root, c, ";"]),
                None => r.add(["-c", &pane.root, ";"]),
            };
        }
        r.add(["select-layout", "-t", &target, &window.layout]);
        r.add([";"]);
    }
    Ok(r.run()?.wait().await?)
}

mod runner {
    use anyhow::{Context, Result, bail};
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Lines};
    use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
    use tokio::task::JoinHandle;

    #[derive(Debug)]
    pub struct Handle {
        cmd: String,
        stdin: Option<BufWriter<ChildStdin>>,
        stdout: Option<Lines<BufReader<ChildStdout>>>,
        stderr: Option<JoinHandle<()>>,
        child: Child,
    }

    #[derive(Debug, Default)]
    pub struct Runner {
        args: Vec<String>,
        detached: bool,
    }

    impl Handle {
        /// Waits until child process is done
        pub async fn wait(mut self) -> Result<()> {
            match self.stdin.take() {
                Some(c) => drop(c),
                None => (),
            }

            let status = self.child.wait().await?;
            if !status.success() {
                match status.code() {
                    Some(code) => bail!("'tmux {}' exited with status code: {}", self.cmd, code),
                    None => bail!("'tmux {}' process terminated by signal", self.cmd),
                }
            }

            match self.stderr {
                Some(c) => c
                    .await
                    .with_context(|| format!("Failed to run 'tmux {}'", self.cmd))?,
                None => (),
            }

            Ok(())
        }

        /// Returns next line of stdout
        pub async fn next_line(&mut self) -> Result<Option<String>> {
            match self.stdout.as_mut() {
                Some(c) => c
                    .next_line()
                    .await
                    .with_context(|| format!("Failed to run 'tmux {}'", self.cmd)),
                None => Ok(None),
            }
        }

        /// Waits for the process to finish and collects its output
        pub async fn output(mut self) -> Result<Vec<String>> {
            let mut res = Vec::new();
            loop {
                match self.next_line().await? {
                    Some(s) => {
                        res.push(s);
                    }
                    None => break,
                }
            }
            self.wait().await?;
            Ok(res)
        }
    }

    impl Runner {
        /// Adds parameters to command
        pub fn add<S: AsRef<str>, I: IntoIterator<Item = S>>(&mut self, it: I) -> &mut Self {
            for s in it {
                self.args.push(s.as_ref().to_string());
            }
            self
        }

        /// Specifies that `self` should inherit stdin/stdout
        pub fn detached() -> Self {
            Self {
                args: Vec::new(),
                detached: true,
            }
        }

        /// Executes command
        pub fn run(&mut self) -> Result<Handle> {
            let command = self.args.join(" ");
            log::debug!("Running \"tmux {}\"", command);

            let mut cmd = Command::new("tmux");
            cmd.args(&self.args).stderr(Stdio::piped());
            if !self.detached {
                cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
            }
            cmd.spawn()
                .map(|mut c| Handle {
                    cmd: command.clone(),
                    stdin: c.stdin.take().map(|c| BufWriter::new(c)),
                    stdout: c.stdout.take().map(|c| BufReader::new(c).lines()),
                    stderr: c.stderr.take().map(|c| stderr_handler(c)),
                    child: c,
                })
                .with_context(|| format!("Failed to run 'tmux {command}'"))
        }

        /// Executes command in control session `control`
        pub async fn run_in(&mut self, control: &mut Handle) -> Result<()> {
            let command = self.args.join(" ");
            log::debug!("Sending \"{}\" to \"tmux {}\"", command, control.cmd);

            match &mut control.stdin {
                Some(w) => w
                    .write_all(command.as_bytes())
                    .await
                    .and(w.write_u8(b'\n').await)
                    .and(w.flush().await)
                    .with_context(|| format!("Failed to run 'tmux {command}'"))?,
                None => (),
            }
            Ok(())
        }
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
}
