use std::path::PathBuf;

use crate::{application::Application, config::Config};
use clap::Parser;

mod application;
mod config;
mod logger;
mod paths;
mod tmux;
mod ui;

#[derive(Parser)]
#[command(name = "sessionizer", version = "1.0", about = "Tmux session manager")]
struct Args {
    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Keep running after session selection
    #[arg(long)]
    keep: bool,
}

pub fn main() {
    let args = Args::parse();
    let config = Config::new(args.config.unwrap_or(paths::config())).unwrap();
    let mut app = Application::new(&config).unwrap();

    if config.show_help {
        log::info!("ctrl+? show help");
    }

    while app.running() {
        app.render();
        app.update();

        match app.selected() {
            Some((name, session)) => {
                if tmux::open(&name.to_string(), session) && !args.keep {
                    log::trace!("Exiting...");
                    break;
                }
            }
            None => (),
        }
    }

    app.finish();
}
