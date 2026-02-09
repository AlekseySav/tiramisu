use getset::{Getters, Setters};
use ratatui::{layout::Offset, style::Style, text::Span, widgets::Widget};
use std::mem;
use unicode_segmentation::UnicodeSegmentation;

use crate::config;

/// Generic text widget
#[derive(Default, Getters, Setters)]
pub struct Paragraph<'a> {
    data: Vec<Vec<(&'a str, &'a Style)>>,
    line: Vec<(&'a str, &'a Style)>,
    /// Scroll to line
    #[getset(get = "pub", set = "pub")]
    scroll: usize,
    /// Wrap text
    #[getset(get = "pub", set = "pub")]
    wrap: bool,
    /// Reverse lines
    #[getset(get = "pub", set = "pub")]
    rev: bool,
}

impl<'a, A, I> From<I> for Paragraph<'a>
where
    A: Iterator<Item = &'a config::Widget>,
    I: Iterator<Item = A>,
{
    fn from(value: I) -> Self {
        let mut p = Paragraph::default();
        for line in value {
            for c in line {
                p.p(c);
            }
            p.br();
        }
        p
    }
}

impl<'a> Paragraph<'a> {
    /// Add styled text
    pub fn p(&mut self, w: &'a config::Widget) -> &mut Self {
        for c in UnicodeSegmentation::graphemes(w.text.as_str(), true) {
            self.line.push((c, &w.style));
        }
        self
    }

    /// Linebreak
    pub fn br(&mut self) -> &mut Self {
        self.data.push(mem::replace(&mut self.line, Vec::new()));
        self
    }
}

impl<'a> Widget for Paragraph<'a> {
    fn render(mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let (w, h) = (area.width as usize, area.height as usize);

        if !self.line.is_empty() {
            self.br();
        }
        if self.rev {
            self.data.reverse();
            self.scroll = self.data.len() - self.scroll;
        }

        let mut lines = Vec::new();
        let mut scroll = 0;

        for (i, line) in self.data.iter().enumerate() {
            let mut n = line.len();
            if self.wrap {
                n = n.min(w);
            }
            for i in 0..=n.saturating_sub(1) / w {
                lines.push(&line[i * w..n.min((i + 1) * w)]);
            }
            if self.scroll == i + 1 {
                scroll = lines.len();
            }
        }

        if self.rev {
            scroll = lines.len() - scroll;
        }
        scroll = (scroll + 1).saturating_sub(h);
        if self.rev {
            scroll = (lines.len() - scroll).saturating_sub(h);
        }

        let dy = h.saturating_sub(lines.len());

        for (mut y, line) in lines.into_iter().skip(scroll).enumerate() {
            if self.rev {
                y += dy;
            }
            for (x, &(str, style)) in line.into_iter().enumerate() {
                Span::styled(str, style.clone())
                    .render(area.offset(Offset::new(x as i32, y as i32)), buf);
            }
        }
    }
}
