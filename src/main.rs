use crate::application::Application;

mod application;
mod config;
mod logger;
mod ui;

pub fn main() {
    let mut app = Application::new().unwrap();
    log::info!("hello");
    while app.running() {
        app.render();
        app.update();
    }
}
