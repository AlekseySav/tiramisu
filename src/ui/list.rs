use crossterm::event::{Event, KeyCode, KeyModifiers};
use indexmap::IndexMap;
use nucleo::Utf32String;
use ratatui::{
    style::Stylize,
    text::{Span, ToSpan},
    widgets::Widget,
};
use std::collections::HashSet;

use crate::ui::ParagraphBuilder;

/// String whith matched indices
#[derive(Debug)]
pub struct MatchedString {
    s: Utf32String,
    indices: HashSet<u32>,
}

/// Selectable list
pub struct List<T> {
    items: IndexMap<Utf32String, T>,
    matches: Vec<MatchedString>,
    prompt: Utf32String,
    selected: usize,
}

pub struct ListWidget<'a> {
    matches: &'a Vec<MatchedString>,
    selected: usize,
}

impl MatchedString {
    /// Returns iterator over characters and indicator whether each of them was matched or not
    pub fn chars(&self) -> impl Iterator<Item = (char, bool)> {
        self.s
            .slice(..)
            .chars()
            .enumerate()
            .map(|(i, c)| (c, self.indices.contains(&(i as u32))))
    }
}

impl Into<String> for MatchedString {
    fn into(self) -> String {
        self.s.to_string()
    }
}

impl<T> List<T> {
    /// Creates empty list
    pub fn new() -> Self {
        Self {
            items: IndexMap::new(),
            matches: Vec::new(),
            prompt: Utf32String::Ascii(String::new().into_boxed_str()),
            selected: 0,
        }
    }

    /// Inserts new item, order preserved, if key already exists, no insertion occurs
    pub fn insert<S: AsRef<str>>(&mut self, name: S, value: T) {
        let s: Utf32String = name.as_ref().into();
        if !self.items.contains_key(&s) {
            self.items.insert(s, value);
        }
        self.update();
    }

    /// Returns number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Updates prompt
    pub fn prompt<S: AsRef<str>>(&mut self, prompt: S) {
        self.prompt = prompt.as_ref().into();
        self.update();
    }

    /// Returns iterator over all matched items
    pub fn matched_items(&self) -> impl Iterator<Item = (&MatchedString, &T)> + ExactSizeIterator {
        self.matches
            .iter()
            .map(|s| (s, self.items.get(&s.s).unwrap()))
    }

    /// Returns currently selected item
    pub fn selected(&self) -> Option<(&MatchedString, &T)> {
        self.matches
            .get(self.selected)
            .map(|s| (s, self.items.get(&s.s).unwrap()))
    }

    fn update(&mut self) {
        let mut res = Vec::new();

        let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
        for item in self.items.keys() {
            let mut indices = Vec::new();
            let score = matcher.fuzzy_indices(item.slice(..), self.prompt.slice(..), &mut indices);

            if score.is_some() {
                res.push((item.clone(), score.unwrap(), indices));
            }
        }

        res.sort_by_key(|(_, score, _)| u16::MAX - score);
        self.matches = res
            .into_iter()
            .map(|(s, _, indices)| MatchedString {
                s: s,
                indices: HashSet::from_iter(indices.into_iter()),
            })
            .collect();

        if self.selected >= self.matches.len() && self.matches.len() > 0 {
            self.selected = self.matches.len() - 1;
        }
    }

    pub fn handle_event(&mut self, evt: &Event) -> bool {
        match evt {
            Event::Key(event) => {
                if event.is_press() || event.is_repeat() {
                    match (event.code, event.modifiers) {
                        (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            if self.selected + 1 < self.matches.len() {
                                self.selected += 1;
                            }
                            true
                        }
                        (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                            if self.selected > 0 {
                                self.selected -= 1;
                            }
                            true
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl<'a> ListWidget<'a> {
    pub fn new<T>(list: &'a List<T>) -> Self {
        Self {
            matches: &list.matches,
            selected: list.selected,
        }
    }
}

impl<'a> Widget for ListWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut b = ParagraphBuilder::new();

        for (i, line) in self.matches.iter().enumerate() {
            if i == self.selected {
                b.p("▌".to_span().magenta());
            } else {
                b.p("▎".to_span().dark_gray());
            }
            b.p("  ".to_span());
            for (c, matched) in line.chars() {
                if matched {
                    b.p(Span::from(c.to_string()).green());
                } else {
                    b.p(Span::from(c.to_string()));
                }
            }
            b.br();
        }

        b.line_mut(self.selected)
            .map(|line| *line = line.clone().bold().italic());

        b.scroll(self.selected + 3).rev().finish().render(area, buf);

        // while lines.len() + state.fzf.iter().len() < area.height as usize {
        //     lines.push(Line::default());
        // }
        // for (i, (name, item)) in state.fzf.iter().enumerate().rev() {
        //     if i > self.selected + 3 && i >= area.height as usize {
        //         continue;
        //     }

        //     lines.push(Line::from_iter(
        //             .chain(std::iter::once(if item.attached {
        //                 Span::styled("◆ ", Style::new().blue())
        //             } else if item.opened {
        //                 Span::styled("◇ ", Style::new().blue())
        //             } else {
        //                 Span::raw("  ")
        //             }))
        //             .chain(
        //                 name.chars()
        //                     .map(|(c, matched)| match matched {
        //                         false => Span::raw(c.to_string()),
        //                         true => Span::styled(c.to_string(), Color::LightGreen),
        //                     })
        //                     .map(|span| {
        //                         if i == selected {
        //                             span.bold().italic()
        //                         } else {
        //                             span
        //                         }
        //                     }),
        //             ),
        //     ));
        // }
    }
}
