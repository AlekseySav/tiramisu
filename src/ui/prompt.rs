use ratatui::{
    layout::Offset,
    style::Stylize,
    text::{Line, Span, ToSpan},
    widgets::Widget,
};
use tui_input::{Input, backend::crossterm::EventHandler};

pub struct Prompt {
    inner: Input,
}

pub struct PromtWidget<'a> {
    ln: Line<'a>,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            inner: Input::default(),
        }
    }

    pub fn cursor(&self) -> usize {
        self.inner.visual_cursor() + 2
    }

    pub fn value(&self) -> &str {
        self.inner.value()
    }

    pub fn handle_event(&mut self, event: &crossterm::event::Event) -> bool {
        self.inner
            .handle_event(event)
            .map_or(false, |r| r.value || r.cursor)
    }
}

impl<'a> PromtWidget<'a> {
    pub fn new(prompt: &'a Prompt, hint: &'a str) -> Self {
        let mut data = Vec::new();
        data.push("> ".to_span().blue().bold());
        data.push(Span::raw(prompt.inner.value()));
        data.push(" ".to_span());
        data.push(" <".to_span().blue().bold());
        data.push(" ".to_span());
        data.push(Span::raw(hint).yellow());

        Self {
            ln: Line::from(data),
        }
    }
}

impl<'a> Widget for PromtWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.width > self.ln.width() as u16 + 4 {
            "─"
                .repeat(area.width as usize - self.ln.width() - 4)
                .to_span()
                .dark_gray()
                .render(area.offset(Offset::new(self.ln.width() as i32 + 2, 0)), buf);
        }
        self.ln.render(area, buf);
    }
}
