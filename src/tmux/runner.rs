use crate::logger::LogResult;
use anyhow::Context;
use std::{
    io::{BufRead, BufReader, Write},
    process::Stdio,
    sync::mpsc,
    time::Duration,
};

/// Simple wrapper around std::process::Command
#[derive(Debug, Default)]
pub struct Runner {
    v: Vec<String>,
}

/// Tmux handler
#[derive(Debug)]
pub struct Handler {
    input: std::process::ChildStdin,
    output: mpsc::Receiver<Option<String>>,
    errors: mpsc::Receiver<()>,
}

impl Runner {
    /// Creates default runner
    pub fn new() -> Self {
        Self::default()
    }

    /// Append arguments to command
    pub fn args<S: AsRef<str>, I: IntoIterator<Item = S>>(&mut self, args: I) -> &mut Self {
        self.v.extend(args.into_iter().map(|s| s.as_ref().into()));
        self
    }

    /// Executes command, and waits for it to finish
    pub fn output(&self) -> Vec<String> {
        self.run()
            .map(|h| h.wait())
            .with_context(|| format!("Failed to run 'tmux {}'", self.v.join(" ")))
            .unwrap_or_log()
    }

    /// Execute tmux command
    pub fn run(&self) -> anyhow::Result<Handler> {
        let s = self.to_string();
        log::trace!("Running 'tmux {}'", s);
        let mut handle = std::process::Command::new("tmux")
            .args(self.v.iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to run 'tmux {s}'"))?;

        // Spawning two more threads per process is probably an overkill,
        // but it does not really affect performance in this case
        Ok(Handler {
            input: handle.stdin.take().unwrap(),
            output: pipe(handle.stdout.take().unwrap(), |line| Some(line)),
            errors: pipe(handle.stderr.take().unwrap(), |line| match line {
                Some(line) => {
                    log::error!("Tmux error: '{line}'");
                    None
                }
                None => Some(()),
            }),
        })
    }
}

impl ToString for Runner {
    fn to_string(&self) -> String {
        let mut res = String::new();
        for (i, s) in self.v.iter().enumerate() {
            if i != 0 {
                res.push(' ');
            }
            res += s;
        }
        res
    }
}

impl Handler {
    /// Waits for the process to finish and returns its output
    pub fn wait(&self) -> Vec<String> {
        let mut res = Vec::new();
        self.errors.recv().unwrap();
        loop {
            match self.output.recv().unwrap() {
                Some(line) => res.push(line),
                None => return res,
            }
        }
    }

    /// Sends line to stdin of the process
    pub fn send<S: AsRef<str>>(&mut self, line: S) {
        let s = line.as_ref().to_string() + "\n";
        self.input
            .write_all(s.as_bytes())
            .with_context(|| format!("Failed to send '{}'", &s[..s.len() - 1]))
            .unwrap_or_log()
    }

    /// Get line from stdout
    pub fn getline(&self) -> Option<String> {
        match self.output.recv_timeout(Duration::ZERO) {
            Ok(Some(s)) => Some(s),
            _ => None,
        }
    }
}

fn pipe<R, T, F>(r: R, f: F) -> mpsc::Receiver<T>
where
    R: Send + std::io::Read + 'static,
    T: Send + 'static,
    F: Send + Fn(Option<String>) -> Option<T> + 'static,
{
    let (send, recv) = mpsc::channel();
    let stream = BufReader::new(r).lines();
    std::thread::spawn(move || {
        for line in stream {
            match line {
                Ok(line) => {
                    f(Some(line)).map(|s| send.send(s).unwrap());
                }
                Err(err) => log::error!("Tmux output is corrupted: '{err}'"),
            }
        }
        f(None).map(|s| send.send(s).unwrap());
    });
    recv
}
