use derive_new::new;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

#[derive(Debug, new)]
pub struct Message {
    level: log::Level,
    message: String,
}

pub struct MessageWidget<'a> {
    level: Span<'a>,
    message: Span<'a>,
}

impl<'a> MessageWidget<'a> {
    pub fn new(msg: &'a Message) -> Self {
        Self {
            level: match msg.level {
                log::Level::Error => Span::styled("error: ", Color::Red),
                log::Level::Warn => Span::styled("warn: ", Color::Yellow),
                log::Level::Info => Span::styled("info: ", Color::Blue),
                log::Level::Debug => Span::raw("debug: "),
                log::Level::Trace => Span::raw("trace: "),
            },
            message: Span::raw(&msg.message),
        }
    }
}

impl<'a> Widget for MessageWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new(Line::from_iter(
            std::iter::once(self.level).chain(std::iter::once(self.message)),
        ))
        .wrap(Wrap { trim: true })
        .render(area, buf);
    }
}
