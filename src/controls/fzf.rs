use super::{EventHandler, StateChanged};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use indexmap::IndexMap;
use nucleo::Utf32String;
use std::collections::HashSet;

/// fzf result
pub struct FzfString<S: AsRef<str> = String> {
    s: S,
    indices: HashSet<u32>,
}

impl<S: AsRef<str>> FzfString<S> {
    // Returns bare string
    pub fn as_str(&self) -> &str {
        self.s.as_ref()
    }

    /// Returns iterator over characters and indicator whether each of them was matched or not
    pub fn chars(&self) -> impl Iterator<Item = (char, bool)> {
        self.s
            .as_ref()
            .char_indices()
            .map(|(i, c)| (c, self.indices.contains(&(i as u32))))
    }
}

pub struct Fzf<V> {
    data: IndexMap<String, V>,
    matches: Vec<FzfString>,
    selected: usize,
}

impl<V> Fzf<V> {
    /// Creates new Fzf instance with data
    pub fn new(data: IndexMap<String, V>) -> Self {
        let mut s = Self {
            data,
            matches: Vec::new(),
            selected: 0,
        };
        s.update("");
        s
    }

    /// Returns list of all matches, sorted respectively to their fzf score
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (&FzfString, &V)> + DoubleEndedIterator<Item = (&FzfString, &V)>
    {
        self.matches
            .iter()
            .map(|s| (s, self.data.get(s.as_str()).unwrap()))
    }

    /// Returns selected element
    pub fn selected(&self) -> (usize, Option<&V>) {
        (
            self.selected,
            self.matches
                .get(self.selected)
                .map_or(None, |s| self.data.get(s.as_str())),
        )
    }

    /// Performs matching with corresponding prompt
    pub fn update(&mut self, prompt: &str) {
        let mut res = Vec::new();

        let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
        for item in self.data.keys() {
            let mut indices = Vec::new();
            let score = matcher.fuzzy_indices(
                Utf32String::from(item.as_ref()).slice(0..item.len()),
                Utf32String::from(prompt).slice(0..prompt.len()),
                &mut indices,
            );

            if score.is_some() {
                res.push((item.clone(), score.unwrap(), indices));
            }
        }

        res.sort_by_key(|(_, score, _)| u16::MAX - score);
        self.matches = res
            .into_iter()
            .map(|(s, _, indices)| FzfString {
                s: s,
                indices: HashSet::from_iter(indices.into_iter()),
            })
            .collect();

        if self.selected >= self.matches.len() && self.matches.len() > 0 {
            self.selected = self.matches.len() - 1;
        }
    }
}

impl<V> EventHandler for Fzf<V> {
    fn handle_event(&mut self, evt: &Event) -> Option<StateChanged> {
        match evt {
            Event::Key(event) => {
                if event.is_press() || event.is_repeat() {
                    match (event.code, event.modifiers) {
                        (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            if self.selected + 1 < self.matches.len() {
                                self.selected += 1;
                                return Some(StateChanged {
                                    value: true,
                                    cursor: false,
                                });
                            }
                        }
                        (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                            if self.selected > 0 {
                                self.selected -= 1;
                                return Some(StateChanged {
                                    value: true,
                                    cursor: false,
                                });
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        None
    }
}
