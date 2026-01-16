use crossterm::event;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
};

use crate::{logger::Logger, ui};

pub struct Application {
    terminal: DefaultTerminal,
    logger: Logger,
    list: ui::List<u8>,
    prompt: ui::Prompt,
    running: bool,
}

impl Application {
    pub fn new() -> std::io::Result<Self> {
        let mut app = Self {
            terminal: ratatui::init(),
            logger: Logger::new(log::LevelFilter::Debug, "hello")?,
            list: ui::List::new(),
            prompt: ui::Prompt::new(),
            running: true,
        };
        app.list.insert("hello", 1);
        app.list.insert("hello you", 1);
        app.terminal.show_cursor().unwrap();

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

                frame.render_widget(ui::ListWidget::new(&self.list), list_area);

                loop {
                    match self.logger.message() {
                        None => break,
                        Some(msg) => frame.render_widget(ui::MessageWidget::new(&msg), area),
                    }
                }
            })
            // TODO properly return error
            .unwrap();
    }

    pub fn update(&mut self) {
        let e = event::read().unwrap();

        if self.prompt.handle_event(&e) {
            self.list.prompt(self.prompt.value());
        } else if !self.list.handle_event(&e) {
            self.running = false;
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
