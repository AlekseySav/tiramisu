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

#[derive(Getters)]
pub struct App {
    tmux: tmux::Control,
    logger: logger::Logger,
    events: tui::EventHandler,
    terminal: Option<ratatui::DefaultTerminal>,
    running: bool,
}

impl App {
    /// Create application
    pub fn new() -> Result<Self> {
        let args = Args::parse();
        let config = config::Config::new(args.config.as_ref())?;
        Ok(Self {
            logger: logger::Logger::new(config.logger)?,
            tmux: tmux::Control::new(config.tmux)?,
            events: tui::EventHandler::new(config.key_bindings),
            terminal: None,
            running: true,
        })
    }

    /// Is process running
    pub fn running(&self) -> bool {
        self.running
    }

    /// Render frame
    pub fn render(&mut self) {
        if let Err(err) = self
            .terminal()
            .draw(|frame| frame.render_widget("hello", frame.area()))
            .with_context(|| "Failed to render frame")
        {
            log::error!("{:?}", err);
            self.running = false;
        }
    }

    /// Process terminal events
    pub fn update(&mut self) {
        self.terminal();
        match self
            .events
            .read()
            .with_context(|| "Failed to render frame")
            .unwrap_or_log()
        {
            Event::Quit => self.running = false,
            Event::Help => todo!(),
            Event::Select => todo!(),
            Event::Cursor {
                offset,
                absolute,
                delete,
            } => todo!(),
            Event::Line { offset } => todo!(),
            Event::Insert(_) => todo!(),
        }
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
