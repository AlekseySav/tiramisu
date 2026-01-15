mod config;
mod controls;
mod state;
mod tmux;

use indexmap::IndexMap;
use state::State;

use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Offset, Rect, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::controls::EventHandler;
use crate::state::Session;

fn render_items(state: &State, area: Rect) -> Paragraph {
    let mut lines = Vec::new();
    let (selected, _) = state.fzf.selected();

    while lines.len() + state.fzf.iter().len() < area.height as usize {
        lines.push(Line::default());
    }
    for (i, (name, item)) in state.fzf.iter().enumerate().rev() {
        if i > selected + 3 && i >= area.height as usize {
            continue;
        }

        lines.push(Line::from_iter(
            std::iter::empty()
                .chain(if i == selected {
                    std::iter::once(Span::styled("▌", Style::new().magenta()))
                } else {
                    std::iter::once(Span::styled("▎", Style::new().dark_gray()))
                })
                .chain(std::iter::once(if item.attached {
                    Span::styled("◆ ", Style::new().blue())
                } else if item.opened {
                    Span::styled("◇ ", Style::new().blue())
                } else {
                    Span::raw("  ")
                }))
                .chain(
                    name.chars()
                        .map(|(c, matched)| match matched {
                            false => Span::raw(c.to_string()),
                            true => Span::styled(c.to_string(), Color::LightGreen),
                        })
                        .map(|span| {
                            if i == selected {
                                span.bold().italic()
                            } else {
                                span
                            }
                        }),
                ),
        ));
    }

    Paragraph::new(lines)
}

fn render_prompt(state: &State, area: Rect) -> impl Widget {
    Line::from(vec![
        Span::styled("❯ ", Style::new().blue()),
        Span::raw(state.prompt.value()),
        Span::raw(" "),
        Span::styled(" ❮ ", Style::new().blue().bold()),
        Span::styled(
            format!("{}/{} ", state.fzf.iter().len(), state.config.session.len()),
            Style::new().yellow().italic(),
        ),
        Span::styled(
            String::from_iter(
                std::iter::repeat('─').take(area.width as usize - 15 - state.prompt.value().len()),
            ),
            Style::new().dark_gray(),
        ),
    ])
}

fn update(state: &mut State, event: Event) {
    state.fzf.handle_event(&event);
    state.prompt.handle_event(&event);
    state.fzf.update(state.prompt.value());

    match event {
        Event::Key(event) => {
            if event.is_press() || event.is_repeat() {
                match event.code {
                    KeyCode::Esc => state.running = false,
                    KeyCode::Enter => state.running = false,
                    KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.running = false
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let config = Config::new().unwrap();
    let mut sessions = IndexMap::new();
    let (attached, opened) = tmux::list_sessions();

    for s in attached.iter() {
        if !config.session.contains_key(s) || sessions.contains_key(s) {
            continue;
        }
        sessions.insert(
            s.clone(),
            Session {
                path: config.session[s].path.clone(),
                window: config.session[s].window.clone(),
                opened: true,
                attached: true,
            },
        );
    }
    for s in opened.iter() {
        if !config.session.contains_key(s) || sessions.contains_key(s) {
            continue;
        }
        sessions.insert(
            s.clone(),
            Session {
                path: config.session[s].path.clone(),
                window: config.session[s].window.clone(),
                opened: true,
                attached: false,
            },
        );
    }
    for s in config.session.keys() {
        if sessions.contains_key(s) {
            continue;
        }
        sessions.insert(
            s.clone(),
            Session {
                path: config.session[s].path.clone(),
                window: config.session[s].window.clone(),
                opened: false,
                attached: false,
            },
        );
    }

    let mut state = State {
        config: Config::new().unwrap(),
        fzf: controls::Fzf::new(sessions),
        prompt: controls::Input::default(),
        running: true,
    };

    let mut terminal = ratatui::init();
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
                render_prompt(
                    &state,
                    area.offset(Offset {
                        x: 0,
                        y: area.height as i32 - 1,
                    }),
                ),
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
    let session = state.fzf.selected().1;
    println!("{:?}", session);
    Ok(())
}
