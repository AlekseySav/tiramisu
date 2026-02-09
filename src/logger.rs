use anyhow::Context;
use getset::Getters;
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{config, env};

/// Result<T, _> that can be unwrapped with logging error if any
pub trait LogResult<T> {
    /// Unwrap value, and log error if any
    fn unwrap_or_log(self) -> T;
}

/// Logger message
#[derive(Debug, Clone, Getters)]
pub struct Message {
    /// Creation time
    #[getset(get = "pub")]
    time: chrono::DateTime<chrono::Local>,
    /// Message log level
    #[getset(get = "pub")]
    level: log::Level,
    /// Message contents
    #[getset(get = "pub")]
    message: String,
}

/// Logger
/// Logs messages to logfile and keeps them locally for application to get them and display
/// If during destruction some of messages were not processed, they will be printed to stderr
#[derive(Default)]
pub struct Logger {
    queue: Arc<Mutex<VecDeque<Message>>>,
    msg: VecDeque<Message>,
    ttl: chrono::Duration,
}

impl Logger {
    pub fn new(config: config::Logger) -> anyhow::Result<Logger> {
        let logger = Logger {
            queue: Arc::default(),
            msg: VecDeque::default(),
            ttl: chrono::Duration::seconds(config.messages.ttl_seconds),
        };
        let queue = logger.queue.clone();

        let path: PathBuf = env::expand(config.logger.path, env::env)?.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        fern::Dispatch::new()
            .chain(
                fern::Dispatch::new()
                    .level(config.messages.level)
                    .format(|out, message, _| out.finish(format_args!("{}", message)))
                    .chain(fern::Output::call(move |record| {
                        queue.lock().unwrap().push_back(Message {
                            time: chrono::Local::now(),
                            level: record.level(),
                            message: record.args().to_string(),
                        })
                    })),
            )
            .chain(
                fern::Dispatch::new()
                    .level(config.logger.level)
                    .format(|out, message, record| {
                        out.finish(format_args!(
                            "{} [{}] {}: {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            record.level(),
                            record.module_path().unwrap_or(""),
                            message
                        ))
                    })
                    .chain(
                        std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(true)
                            .open(&path)?,
                    ),
            )
            .apply()
            .with_context(|| "Failed to create logger")?;

        Ok(logger)
    }

    /// Collect and return all alive messages
    pub fn messages(&mut self) -> Vec<Message> {
        self.msg.extend(self.queue.lock().unwrap().drain(..));
        let now = chrono::Local::now();
        while self.msg.front().is_some_and(|m| now - m.time() >= self.ttl) {
            self.msg.pop_front();
        }
        self.msg.iter().rev().cloned().collect()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        for m in self.queue.lock().unwrap().iter() {
            eprintln!("{}: {:?}", m.level.to_string(), m.message())
        }
    }
}

impl<T: Default, E: std::fmt::Debug> LogResult<T> for Result<T, E> {
    fn unwrap_or_log(self) -> T {
        match self {
            Ok(v) => v,
            Err(err) => {
                log::error!("{:?}", err);
                T::default()
            }
        }
    }
}

fn logpath() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or(PathBuf::from("."))
        .join("tiramisu")
        .join("tiramisu.log")
}
