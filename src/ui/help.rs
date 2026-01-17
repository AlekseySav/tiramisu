use ratatui::{
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};

pub struct HelpWidget {}

impl HelpWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for HelpWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new(Text::from(
            r"
                 ctrl+? toggle help
                ",
        ))
        .wrap(Wrap { trim: true })
        .render(area, buf);
    }
}
