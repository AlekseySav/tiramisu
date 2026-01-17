use std::{
    collections::VecDeque,
    fs::OpenOptions,
    sync::{Arc, Mutex},
};

use fern::Dispatch;

use crate::{config, ui};

/// Logger
/// Logs are stored in file and localy, so that they can appear in popups
pub struct Logger {
    logs: Arc<Mutex<VecDeque<ui::Message>>>,
    ttl: chrono::Duration,
}

impl Logger {
    /// Creates new logger
    pub fn new(config: &config::Logger) -> std::io::Result<Self> {
        let logger = Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
            ttl: config.message_ttl,
        };
        let sender = logger.logs.clone();

        std::fs::create_dir_all(config.log_path.parent().unwrap())?;
        Dispatch::new()
            // log to file
            .chain({
                Dispatch::new()
                    .format(|out, message, record| {
                        out.finish(format_args!(
                            "{} [{}] {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            record.level(),
                            message
                        ))
                    })
                    .level(config.level)
                    .chain(
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(true)
                            .open(&config.log_path)?,
                    )
            })
            // log to local queue
            .chain({
                Dispatch::new()
                    .level(config.level)
                    .format(|out, message, _| out.finish(format_args!("{}", message)))
                    .chain(fern::Output::call(move |record| {
                        sender.lock().unwrap().push_back(ui::Message::new(
                            record.level(),
                            record.args().to_string(),
                            chrono::Local::now(),
                        ));
                    }))
            })
            .apply()
            .unwrap();

        Ok(logger)
    }

    /// List all messages, that hasn't expired yet
    pub fn messages(&mut self) -> Vec<ui::Message> {
        let now = chrono::Local::now();
        let mut logs = self.logs.lock().unwrap();
        while logs.front().is_some_and(|m| now - m.time() >= self.ttl) {
            logs.pop_front();
        }

        logs.iter().cloned().collect()
    }
}
