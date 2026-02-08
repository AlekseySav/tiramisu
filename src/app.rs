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
    terminal: Option<ratatui::DefaultTerminal>,
    tmux: tmux::Control,
    logger: logger::Logger,
    events: tui::EventHandler,
    screen: tui::Screen,
    prompt: tui::Prompt,
    selected: usize,
    help: bool,
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
            screen: tui::Screen::new(config.theme),
            prompt: tui::Prompt::default(),
            selected: 0,
            help: false,
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
                frame.render_widget(self.prompt.data().to_string(), frame.area());
                frame.set_cursor_position((*self.prompt.cursor() as u16, 0));
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
        match self
            .events
            .read()
            .with_context(|| "Failed to render frame")
            .unwrap_or_log()
        {
            Event::Quit => self.running = false,
            Event::Help => self.help = !self.help,
            Event::Select => todo!(),
            Event::Cursor {
                offset,
                absolute,
                delete,
            } => match delete {
                true => self.prompt.delete(offset, absolute),
                false => self.prompt.set_cursor(offset, absolute),
            },
            Event::Line { offset } => {
                self.selected = self
                    .selected
                    .saturating_add_signed(offset)
                    .min(self.tmux.sessions().len().saturating_sub(1))
            }
            Event::Insert(s) => self.prompt.insert(s),
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
