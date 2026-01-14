mod config;
mod fzf;
mod state;

use state::State;

use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Offset, Rect, Size},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use tmux_interface::{HasSession, StdIO};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::state::Matches;

fn render_items(state: &State, area: Rect) -> Paragraph {
    let mut lines = Vec::new();
    while lines.len() + state.matches.items.len() < area.height as usize {
        lines.push(Line::default());
    }
    for (i, item) in state.matches.items.iter().enumerate().rev() {
        if i > state.matches.selected + 3 && i >= area.height as usize {
            continue;
        }

        lines.push(Line::from_iter(
            std::iter::once(Span::raw(
                if tmux_interface::Tmux::with_command(
                    HasSession::new().target_session(item.as_str()),
                )
                .stderr(Some(StdIO::Null))
                .status()
                .unwrap()
                .success()
                {
                    " A "
                } else {
                    "   "
                },
            ))
            .chain(item.chars().map(|(c, matched)| match matched {
                false => Span::raw(c.to_string()),
                true => Span::styled(c.to_string(), Color::LightGreen),
            }))
            .map(|span| {
                if i == state.matches.selected {
                    span.on_dark_gray()
                } else {
                    span
                }
            }),
        ));
    }

    Paragraph::new(lines)
}

fn render_prompt(state: &State) -> impl Widget {
    Line::from(vec![
        Span::styled("> ", Color::LightBlue),
        Span::raw(state.prompt.value()),
        Span::raw(" "),
        Span::styled(" < ", Color::LightBlue),
        Span::styled(
            format!(
                "{}/{}",
                state.matches.items.len(),
                state.config.session.len()
            ),
            Color::Yellow,
        ),
    ])
}

fn update(state: &mut State, event: Event) {
    state.prompt.handle_event(&event);
    match event {
        Event::Key(event) => {
            if event.is_press() || event.is_repeat() {
                match event.code {
                    KeyCode::Esc => state.running = false,
                    KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.running = false
                    }
                    KeyCode::Enter => state.running = false,
                    KeyCode::Up => {
                        if state.matches.selected + 1 < state.matches.items.len() {
                            state.matches.selected += 1;
                        }
                    }
                    KeyCode::Char('p') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.matches.selected + 1 < state.matches.items.len() {
                            state.matches.selected += 1
                        }
                    }
                    KeyCode::Down => {
                        if state.matches.selected > 0 {
                            state.matches.selected -= 1;
                        }
                    }
                    KeyCode::Char('n') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.matches.selected > 0 {
                            state.matches.selected -= 1
                        }
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }

    state.update_matches();
    if state.matches.selected >= state.matches.items.len() {
        state.matches.selected = if state.matches.items.is_empty() {
            0
        } else {
            state.matches.items.len() - 1
        };
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    let mut state = State {
        config: Config::new().unwrap(),
        prompt: Input::default(),
        matches: Matches {
            items: vec![],
            selected: 0,
        },
        running: true,
    };

    state.update_matches();
    while state.running {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(
                render_items(&state, area.resize(Size::new(area.width, area.height - 1))),
                area.resize(Size {
                    width: area.width,
                    height: area.height - 1,
                }),
            );
            frame.render_widget(
                render_prompt(&state),
                area.offset(Offset {
                    x: 0,
                    y: area.height as i32 - 1,
                }),
            );
            frame.set_cursor_position((
                2 + state.prompt.visual_cursor() as u16,
                area.height as u16 - 1,
            ));
        })?;

        update(&mut state, event::read()?);
    }

    ratatui::restore();
    Ok(())
}
