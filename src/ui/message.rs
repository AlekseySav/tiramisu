use derive_new::new;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

#[derive(Debug, Clone, new)]
pub struct Message {
    level: log::Level,
    message: String,
    time: chrono::DateTime<chrono::Local>,
}

pub struct MessageWidget<'a> {
    data: Vec<Line<'a>>,
}

impl Message {
    pub fn time(&self) -> chrono::DateTime<chrono::Local> {
        self.time
    }
}

impl<'a> MessageWidget<'a> {
    pub fn new<I: Iterator<Item = &'a Message>>(msg: I) -> Self {
        let mut data = Vec::new();
        for m in msg {
            let mut line = Vec::new();
            line.push(match m.level {
                log::Level::Error => Span::styled("error: ", Color::Red),
                log::Level::Warn => Span::styled("warn: ", Color::Yellow),
                log::Level::Info => Span::styled("info: ", Color::Blue),
                log::Level::Debug => Span::styled("debug: ", Color::Gray),
                log::Level::Trace => Span::styled("trace: ", Color::Gray),
            });
            line.push(Span::styled(&m.message, Color::Gray));

            data.push(Line::default());
            data.push(Line::from(line));
        }

        Self { data }
    }
}

impl<'a> Widget for MessageWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new(self.data)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
