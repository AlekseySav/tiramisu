use crate::{app::AppState, config, tui::Paragraph};
use derive_new::new;
use ratatui::widgets::Widget;

#[derive(new)]
pub struct Prompt<'a> {
    config: &'a config::Prompt,
    state: &'a AppState,
}

impl Prompt<'_> {
    pub fn cursor(&self) -> usize {
        self.config.prefix.len() + *self.state.prompt().cursor()
    }
}

impl Widget for Prompt<'_> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut p = Paragraph::default();
        p.p(&self.config.prefix);
        let content = config::Widget::new(
            self.state.prompt().data().clone() + " ",
            self.config.content,
        );
        p.p(&content);
        p.p(&self.config.postfix);
        let hint = config::Widget::new(
            format!(
                "{}/{} ",
                self.state.sessions().len(),
                self.state.sessions().len()
            ),
            self.config.hint,
        );
        p.p(&hint);
        let ruler = config::Widget {
            text: self.config.ruler.text.repeat(
                (area.width as usize)
                    .saturating_sub(self.config.prefix.len())
                    .saturating_sub(content.len())
                    .saturating_sub(self.config.postfix.len())
                    .saturating_sub(hint.len())
                    .saturating_sub(1),
            ),
            style: self.config.ruler.style,
        };
        p.p(&ruler);
        p.render(area, buf);
    }
}
