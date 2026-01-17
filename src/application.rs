use std::collections::HashMap;

use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
};

use crate::{config, logger::Logger, tmux, ui};

pub struct Application {
    config: Vec<config::Session>,
    terminal: DefaultTerminal,
    logger: Logger,
    list: ui::SessionList,
    prompt: ui::Prompt,
    selected: Option<(ui::MatchedString, ui::Session)>,
    help: bool,
    running: bool,
}

impl Application {
    pub fn new(config: config::Config) -> std::io::Result<Self> {
        let mut app = Self {
            config: config.session,
            terminal: ratatui::init(),
            logger: Logger::new(&config.logger)?,
            list: ui::SessionList::new(),
            prompt: ui::Prompt::new(),
            selected: None,
            help: false,
            running: true,
        };
        app.refresh();

        if config.show_help {
            log::info!("ctrl+?/ctrl+7 show help");
        }

        Ok(app)
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn render(&mut self) {
        self.terminal
            .draw(|frame| {
                let area = frame.area();

                // display fzf & prompt
                let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
                let [list_area, prompt_area] = layout.areas(area);

                let hint = format!("{}/{}", self.list.matched_items().len(), self.list.len());
                frame.render_widget(ui::PromtWidget::new(&self.prompt, &hint), prompt_area);
                frame.set_cursor_position((
                    area.x + self.prompt.cursor() as u16,
                    area.y + area.height,
                ));
                frame.render_widget(ui::SessionListWidget::new(&self.list), list_area);

                // display help
                if self.help {
                    frame.render_widget(ui::HelpWidget::new(), area);
                }

                // display messages
                let layout = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Percentage(30),
                    Constraint::Length(1),
                ]);
                let [_, msg_area, _] = layout.areas(area);
                frame.render_widget(
                    ui::MessageWidget::new(self.logger.messages().iter()),
                    msg_area,
                );
            })
            .unwrap();

        if self.help {
            self.terminal.hide_cursor().unwrap();
        } else {
            self.terminal.show_cursor().unwrap();
        }
    }

    pub fn update(&mut self) {
        while event::poll(std::time::Duration::from_millis(10)).unwrap() {
            let e = event::read().unwrap();

            if self.prompt.handle_event(&e).value {
                self.list.prompt(self.prompt.value());
                return;
            }
            if self.list.handle_event(&e) {
                return;
            }
            match e.as_key_press_event() {
                Some(e) => match e.code {
                    KeyCode::Esc => self.running = false,
                    KeyCode::Char('c') if e.modifiers == KeyModifiers::CONTROL => {
                        self.running = false
                    }
                    KeyCode::Char('7') if e.modifiers == KeyModifiers::CONTROL => {
                        self.help = !self.help
                    }
                    KeyCode::Char('x') if e.modifiers == KeyModifiers::CONTROL => {
                        match self.list.selected() {
                            Some((name, session)) => tmux::kill(&name.to_string(), &session),
                            _ => (),
                        }
                    }
                    KeyCode::Enter => {
                        self.selected = self.list.selected().map(|(n, s)| (n.clone(), s.clone()))
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        self.refresh();
    }

    pub fn selected(&mut self) -> Option<(ui::MatchedString, ui::Session)> {
        std::mem::replace(&mut self.selected, None)
    }

    pub fn finish(&self) {
        ratatui::restore();
    }

    fn refresh(&mut self) {
        let selected = self.list.get_selected_index();
        self.list = ui::SessionList::new();
        let (attached, opened) = tmux::list_sessions();
        let map = HashMap::<String, _>::from_iter(self.config.iter().map(|s| (s.name.clone(), s)));

        for name in attached {
            map.get(&name)
                .inspect(|s| self.list.insert(s, ui::State::Attached));
        }

        for name in opened {
            map.get(&name)
                .inspect(|s| self.list.insert(s, ui::State::Created));
        }

        for session in self.config.iter() {
            self.list.insert(session, ui::State::None);
        }
        self.list.set_selected(selected);
        self.list.prompt(self.prompt.value());
    }
}
