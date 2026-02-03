mod app;
mod env;
mod logger;
mod tmux;
mod tui;

use serde::{Deserialize, Serialize};

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write, stdout};

use crate::logger::Logger;

#[derive(Default, Deserialize, Serialize)]
struct Config {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    logger: logger::Config,
    #[serde(default)]
    session: Vec<tmux::Session>,
}

fn main() -> anyhow::Result<()> {
    let config: Config = toml::from_str(&std::fs::read_to_string("./examples/config.toml")?)?;
    let logger = Logger::new(config.logger)?;
    let mut tmux = tmux::Control::start(
        config.name.as_ref().map_or("tiramisu/control", |s| &s),
        tmux::expand_sessions(config.session.iter())?,
    )?;

    tmux.finish();

    // let mut tmux = tmux::Tmux::new().await?;
    // println!("{:?}", tmux);

    // tmux.switch(&Session {
    //     name: "hello".to_string(),
    //     root: "/".to_string(),
    //     windows: Vec::from([Window {
    //         name: "a".to_string(),
    //         layout: "main-vertical".to_string(),
    //         panes: Vec::from([Pane {
    //             root: "/".to_string(),
    //             command: None,
    //         }]),
    //     }]),
    //     created: false,
    //     attached: false,
    // })
    // .await?;

    // tmux.wait().await?;
    Ok(())

    // // Setup terminal
    // setup_terminal()?;

    // // Main loop
    // loop {
    //     // Check for events
    //     if event::poll(std::time::Duration::from_millis(16))? {
    //         match event::read()? {
    //             Event::Key(KeyEvent { code, .. }) => {
    //                 match code {
    //                     KeyCode::Char('q') | KeyCode::Char('Q') => break,
    //                     KeyCode::Char('c') => {
    //                         // Handle 'c' key
    //                         print_to_stdout("You pressed 'c'\r\n")?;
    //                     }
    //                     KeyCode::Enter => {
    //                         print_to_stdout("Enter pressed\r\n")?;
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }

    //     // Your async work here (example: periodic task)
    //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    // }

    // // Cleanup
    // restore_terminal()?;
    // Ok(())
}

fn setup_terminal() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Ok(())
}

fn restore_terminal() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn print_to_stdout(text: &str) -> io::Result<()> {
    let mut stdout = stdout();
    write!(stdout, "{}", text)?;
    stdout.flush()?;
    Ok(())
}
