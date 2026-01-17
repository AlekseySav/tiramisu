use std::borrow::Cow;

use ratatui::{
    layout::{Constraint, Layout, Margin, Offset, Rect},
    style::{Color, Style},
    symbols::border,
    text::Span,
    widgets::{Block, Widget},
};

pub struct HelpWidget<'a> {
    key_width: u16,
    width: u16,
    height: u16,
    help: Vec<Help<'a>>,
    background: Block<'a>,
}

struct Help<'a> {
    key: Span<'a>,
    help: Span<'a>,
}

impl<'a> Help<'a> {
    fn new<S: Into<Cow<'a, str>>>(key: S, help: S) -> Self {
        Self {
            key: Span::styled(key, Color::Red),
            help: Span::styled(help, Color::Gray),
        }
    }
}

impl<'a> HelpWidget<'a> {
    pub fn new() -> Self {
        let mut v: Vec<Help<'a>> = Vec::new();
        v.push(Help::new("ctrl+?/ctrl+7", "toggle help"));
        v.push(Help::new("ctrl+c/esc", "quit"));
        v.push(Help::new("enter", "switch to selected session"));
        v.push(Help::new("ctrl+x", "kill selected session"));
        v.push(Help::new("ctrl+p/up ctrl+n/down", "move selection"));
        v.push(Help::new("", ""));
        v.push(Help::new("ctrl+h/backspace", "backspace"));
        v.push(Help::new("ctrl+d/delete", "delete"));
        v.push(Help::new("ctrl+b/left, right", "move prompt cursor"));
        v.push(Help::new("ctrl+a", "move prompt cursor to beginning"));
        v.push(Help::new("ctrl+e", "move prompt cursor to end"));
        v.push(Help::new("ctrl+w", "delete from cursor to beginning"));
        v.push(Help::new("ctrl+k", "delete from cursor to end"));
        v.push(Help::new("ctrl+shift+v/cmd+v", "paste"));
        Self {
            key_width: v.iter().map(|s| s.key.width()).max().unwrap() as u16 + 2,
            width: v.iter().map(|s| s.help.width()).max().unwrap() as u16,
            height: v.len() as u16,
            help: v,
            background: Block::bordered()
                .style(Style::default().bg(Color::Black).fg(Color::Black))
                .border_set(border::ROUNDED)
                .border_style(Color::DarkGray),
        }
    }
}

impl<'a> Widget for HelpWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.width < self.width + self.key_width + 6 || area.height < self.height + 2 {
            log::error!("not enough screen space to render help");
            return;
        }
        let [_, vl, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(self.height),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, hl, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.width + self.key_width + 2),
            Constraint::Fill(1),
        ])
        .areas(area);

        let area = Rect::new(hl.x, vl.y, hl.width, vl.height);

        self.background.render(area.outer(Margin::new(2, 1)), buf);
        for (y, help) in self.help.into_iter().enumerate() {
            help.key.render(area.offset(Offset::new(0, y as i32)), buf);
            help.help.render(
                area.offset(Offset::new(self.key_width as i32 + 2, y as i32)),
                buf,
            );
        }
    }
}
