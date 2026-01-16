use std::{
    collections::VecDeque,
    fs::OpenOptions,
    path::Path,
    sync::{Arc, Mutex},
};

use fern::Dispatch;

use crate::ui;

/// Logger
/// Logs are stored in file and localy, so that they can appear in popups
pub struct Logger {
    pub logs: Arc<Mutex<VecDeque<ui::Message>>>,
}

impl Logger {
    /// Creates new logger
    pub fn new<P: AsRef<Path>>(level: log::LevelFilter, logfile: P) -> std::io::Result<Self> {
        let logger = Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
        };
        let sender = logger.logs.clone();

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
                    .level(level)
                    .chain(
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .append(true)
                            .open(logfile)?,
                    )
            })
            // log to local queue
            .chain({
                Dispatch::new()
                    .format(|out, message, _| out.finish(format_args!("{}", message)))
                    .chain(fern::Output::call(move |record| {
                        sender
                            .lock()
                            .unwrap()
                            .push_back(ui::Message::new(record.level(), record.args().to_string()));
                    }))
            })
            .apply()
            .unwrap();

        Ok(logger)
    }

    pub fn message(&self) -> Option<ui::Message> {
        self.logs.lock().unwrap().pop_front()
    }
}
