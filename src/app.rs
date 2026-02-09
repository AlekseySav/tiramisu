use std::time::Duration;

use crate::{
    config::{self, Event},
    logger::{self, LogResult},
    tmux, tui,
};
use anyhow::{Context, Result};
use clap::Parser;
use getset::Getters;

#[derive(Parser)]
#[command(name = "tiramisu", version, about = "Tmux session manager")]
struct Args {
    /// Path to config file
    #[arg(long)]
    config: Option<String>,

    /// Print logs and quit
    #[arg(long)]
    logs: bool,
}

#[derive(Debug, Default, Getters)]
pub struct AppState {
    #[getset(get = "pub")]
    prompt: tui::Input,
    #[getset(get = "pub")]
    messages: Vec<logger::Message>,
    #[getset(get = "pub")]
    sessions: Vec<config::Session>,
    #[getset(get = "pub")]
    selected: usize,
    #[getset(get = "pub")]
    help: bool,
}

#[derive(Getters)]
pub struct App {
    terminal: Option<ratatui::DefaultTerminal>,
    tmux: tmux::Control,
    logger: logger::Logger,
    events: tui::EventHandler,
    screen: tui::Screen,
    state: AppState,
    running: bool,
}

impl App {
    /// Create application
    pub fn new() -> Result<Self> {
        let args = Args::parse();
        let config = config::Config::new(args.config.as_ref())?;
        Ok(Self {
            terminal: None,
            logger: logger::Logger::new(config.logger)?,
            tmux: tmux::Control::new(config.tmux)?,
            events: tui::EventHandler::new(config.key_bindings),
            screen: tui::Screen::new(config.ui),
            state: AppState::default(),
            running: true,
        })
    }

    /// Is process running
    pub fn running(&self) -> bool {
        self.running
    }

    /// Render frame
    pub fn render(&mut self) {
        self.terminal();
        if let Err(err) = self
            .terminal
            .as_mut()
            .unwrap() // TODO: refactor self.terminal() or this
            .draw(|frame| {
                self.screen.render(frame, &self.state);
            })
            .with_context(|| "Failed to render frame")
        {
            log::error!("{:?}", err);
            self.running = false;
        }
    }

    /// Process terminal events
    pub fn update(&mut self) {
        self.terminal();
        loop {
            match self
                .events
                .poll(Duration::from_millis(10)) // TODO: move to config
                .with_context(|| "Failed to render frame")
                .unwrap_or_log()
            {
                Event::None => break,
                Event::Quit => self.running = false,
                Event::Help => self.state.help = !self.state.help,
                Event::Select => todo!(),
                Event::Cursor {
                    offset,
                    absolute,
                    delete,
                } => match delete {
                    true => self.state.prompt.delete(offset, absolute),
                    false => self.state.prompt.set_cursor(offset, absolute),
                },
                Event::Line { offset } => {
                    self.state.selected = self
                        .state
                        .selected
                        .saturating_add_signed(offset)
                        .min(self.tmux.sessions().len().saturating_sub(1));
                }
                Event::Insert(s) => self.state.prompt.insert(s),
            }
        }

        self.state.sessions = self.tmux.sessions().cloned().collect();
        self.state.messages = self.logger.messages();
    }

    /// Stop application
    pub fn finish(mut self) {
        self.restore();
        self.tmux.finish();
    }

    fn restore(&mut self) {
        if self.terminal.is_some() {
            ratatui::restore();
            self.terminal = None;
        }
    }

    fn terminal(&mut self) -> &mut ratatui::DefaultTerminal {
        self.terminal.get_or_insert_with(|| ratatui::init())
    }
}
