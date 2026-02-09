use ratatui::{
    Frame,
    layout::{Layout, Rect},
};

use crate::{app::AppState, config, logger::LogResult};

pub struct Screen {
    config: config::Ui,
}

impl Screen {
    pub fn new(theme: config::Ui) -> Self {
        Self { config: theme }
    }

    pub fn render(&self, frame: &mut Frame, state: &AppState) {
        let cols = Layout::horizontal(self.config.grid.columns.iter()).split(frame.area());
        let rows = Layout::vertical(self.config.grid.rows.iter()).split(frame.area());

        let pa = area(&cols, &rows, &self.config.prompt.pos);
        let pw = super::prompt::Prompt::new(&self.config.prompt, &state);
        frame.set_cursor_position((pw.cursor() as u16 + pa.x, pa.y));
        frame.render_widget(pw, pa);

        frame.render_widget(
            super::messages::Messages::new(&self.config.messages, &state),
            area(&cols, &rows, &self.config.messages.pos),
        );
    }
}

fn area(cols: &[Rect], rows: &[Rect], pos: &config::Position) -> Rect {
    let x0 = cols
        .get(pos.x.0)
        .map(|r| r.x)
        .ok_or(anyhow::anyhow!("Invalid column: {}", pos.x.0))
        .unwrap_or_log();
    let y0 = rows
        .get(pos.y.0)
        .map(|r| r.y)
        .ok_or(anyhow::anyhow!("Invalid row: {}", pos.y.0))
        .unwrap_or_log();
    let x1 = cols
        .get(pos.x.1)
        .map(|r| r.x + r.width)
        .ok_or(anyhow::anyhow!("Invalid column: {}", pos.x.1))
        .unwrap_or_log();
    let y1 = rows
        .get(pos.y.1)
        .map(|r| r.y + r.height)
        .ok_or(anyhow::anyhow!("Invalid row: {}", pos.y.1))
        .unwrap_or_log();
    Rect::new(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
}
