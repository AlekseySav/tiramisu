use std::mem::replace;

use ratatui::{
    text::{Line, Span, Text},
    widgets::Widget,
};

/// Somewhat simplified somewhat fancier version of ratatui::widgets::Paragraph
/// Supports vertical scrolling, line reversing, which is useful in this case, as usually fuzzy
/// finders show matches from bottom to top
pub struct Paragraph<'a> {
    content: Vec<Line<'a>>,
    scroll: usize,
    rev: bool,
}

/// Simplified way of creating Paragraph
pub struct ParagraphBuilder<'a> {
    data: Vec<Line<'a>>,
    last: Vec<Span<'a>>,
    scroll: usize,
    rev: bool,
}

impl<'a, T: Into<Text<'a>>> From<T> for Paragraph<'a> {
    fn from(value: T) -> Self {
        Self {
            content: value.into().lines,
            scroll: 0,
            rev: false,
        }
    }
}

impl<'a> Paragraph<'a> {
    pub fn scroll(mut self, y: usize) -> Self {
        self.scroll = y;
        self
    }

    pub fn rev(mut self, yes: bool) -> Self {
        self.rev = yes;
        self
    }
}

impl<'a> ParagraphBuilder<'a> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            last: Vec::new(),
            scroll: 0,
            rev: false,
        }
    }

    pub fn p(&mut self, s: Span<'a>) {
        self.last.push(s);
    }

    pub fn br(&mut self) {
        self.data.push(replace(&mut self.last, Vec::new()).into());
    }

    pub fn rev(mut self) -> Self {
        self.rev = true;
        self
    }

    pub fn scroll(mut self, y: usize) -> Self {
        self.scroll = y;
        self
    }

    pub fn line_mut(&mut self, i: usize) -> Option<&mut Line<'a>> {
        self.data.get_mut(i)
    }

    pub fn finish(mut self) -> Paragraph<'a> {
        if !self.last.is_empty() {
            self.br();
        }
        Paragraph::from(self.data).scroll(self.scroll).rev(self.rev)
    }
}

impl<'a> Widget for Paragraph<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let i = usize::max(self.scroll, area.height as usize) - area.height as usize;
        let n = usize::min(self.content.len(), area.height as usize);

        for (i, line) in self.content.iter().skip(i).take(n).enumerate() {
            let mut area = area;
            if self.rev {
                area.y = area.height - 1 - i as u16;
            } else {
                area.y += i as u16;
            }
            line.render(area, buf);
        }
    }
}
