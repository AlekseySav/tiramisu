use derive_new::new;
use ratatui::widgets::Widget;

use crate::{app::AppState, config, tui::Paragraph};

#[derive(new)]
pub struct Messages<'a> {
    config: &'a config::MessagesUi,
    state: &'a AppState,
}

impl Widget for Messages<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut v = Vec::new();
        for message in self.state.messages().iter().rev() {
            v.push(Vec::new());
            v.push(Vec::new());
            if let Some(m) = self.config.style.get(message.level()) {
                v.last_mut().unwrap().push(m.level.clone());
                v.last_mut()
                    .unwrap()
                    .push(config::Widget::new(message.message().clone(), m.content));
            }
        }
        let mut p = Paragraph::from(v.iter().map(|v| v.iter()));
        p.set_rev(true);
        p.render(area, buf);
    }
}
