mod app;
pub mod tmux;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write, stdout};

use crate::tmux::{Pane, Session, Window};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or(""),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stderr())
        .apply()?;

    let mut tmux = tmux::Tmux::new().await?;
    println!("{:?}", tmux);

    tmux.switch(&Session {
        name: "hello".to_string(),
        root: "/".to_string(),
        windows: Vec::from([Window {
            name: "a".to_string(),
            layout: "main-vertical".to_string(),
            panes: Vec::from([Pane {
                root: "/".to_string(),
                command: None,
            }]),
        }]),
        created: false,
        attached: false,
    })
    .await?;

    tmux.wait().await?;
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
