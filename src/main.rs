mod app;
mod config;
mod env;
mod logger;
mod tmux;
mod tui;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    env::init();
    let mut app = app::App::new().with_context(|| "Failed to start application")?;
    while app.running() {
        app.render();
        app.update();
    }
    app.finish();
    Ok(())
}
