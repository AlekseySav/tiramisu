use crate::{application::Application, config::Config};

mod application;
mod config;
mod logger;
mod ui;

pub fn main() {
    let config = Config::new("examples/config.toml").unwrap();
    let mut app = Application::new(&config).unwrap();

    if config.show_help {
        log::info!("ctrl+? show help");
    }

    while app.running() {
        app.render();
        app.update();
    }
}
