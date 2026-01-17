use std::collections::HashMap;

use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout, Margin},
};

use crate::{config, logger::Logger, tmux, ui};

pub struct Application {
    terminal: DefaultTerminal,
    logger: Logger,
    list: ui::SessionList,
    prompt: ui::Prompt,
    selected: Option<(ui::MatchedString, ui::Session)>,
    help: bool,
    running: bool,
}

impl Application {
    pub fn new(config: &config::Config) -> std::io::Result<Self> {
        let mut app = Self {
            terminal: ratatui::init(),
            logger: Logger::new(&config.logger)?,
            list: ui::SessionList::new(),
            prompt: ui::Prompt::new(),
            selected: None,
            help: false,
            running: true,
        };
        app.add_sessions(&config.session);

        Ok(app)
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn render(&mut self) {
        self.terminal
            .draw(|frame| {
                let area = frame.area();
                let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]);
                let [list_area, prompt_area] = layout.areas(area);

                let hint = format!("{}/{}", self.list.matched_items().len(), self.list.len());
                frame.render_widget(ui::PromtWidget::new(&self.prompt, &hint), prompt_area);
                frame.set_cursor_position((
                    area.x + self.prompt.cursor() as u16,
                    area.y + area.height,
                ));

                frame.render_widget(ui::SessionListWidget::new(&self.list), list_area);

                let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(30)]);
                let [_, msg_area] = layout.areas(area);
                frame.render_widget(
                    ui::MessageWidget::new(self.logger.messages().iter()),
                    msg_area,
                );

                if self.help {
                    frame.render_widget(ui::HelpWidget::new(), area);
                }
            })
            .unwrap();

        if self.help {
            self.terminal.hide_cursor().unwrap();
        } else {
            self.terminal.show_cursor().unwrap();
        }
    }

    pub fn update(&mut self) {
        while event::poll(std::time::Duration::from_millis(100)).unwrap() {
            let e = event::read().unwrap();
            log::trace!("Received event {:?}", e);

            if self.prompt.handle_event(&e) {
                self.list.prompt(self.prompt.value());
                return;
            }
            if self.list.handle_event(&e) {
                return;
            }
            match e.as_key_press_event() {
                Some(e) => match e.code {
                    KeyCode::Esc => self.running = false,
                    KeyCode::Char('c') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.running = false
                    }
                    KeyCode::Char('7') if e.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.help = !self.help
                    }
                    KeyCode::Enter => {
                        self.selected = self.list.selected().map(|(n, s)| (n.clone(), s.clone()))
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn selected(&self) -> &Option<(ui::MatchedString, ui::Session)> {
        &self.selected
    }

    pub fn finish(&self) {
        ratatui::restore();
    }

    fn add_sessions(&mut self, sessions: &Vec<config::Session>) {
        let (attached, opened) = tmux::list_sessions();
        let map: HashMap<String, &config::Session> =
            HashMap::from_iter(sessions.iter().map(|s| (s.name.clone(), s)));

        for name in attached {
            map.get(&name)
                .inspect(|s| self.list.insert(s, ui::State::Attached));
        }

        for name in opened {
            map.get(&name)
                .inspect(|s| self.list.insert(s, ui::State::Created));
        }

        for session in sessions.iter() {
            self.list.insert(session, ui::State::None);
        }
    }
}
