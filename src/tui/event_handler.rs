use crate::config::{Event, EventConfig, KeyBindings};
use crossterm::event::{self, KeyCode, KeyModifiers};
use std::collections::HashMap;

pub struct EventHandler {
    config: HashMap<(KeyCode, KeyModifiers), (String, EventConfig)>,
}

impl EventHandler {
    pub fn new(config: KeyBindings) -> Self {
        Self {
            config: HashMap::from_iter(
                config
                    .keybindings
                    .into_iter()
                    .map(|(k, v)| ((*k.codes.first(), k.modifiers), (k.to_string(), v))),
            ),
        }
    }

    pub fn read(&self) -> anyhow::Result<Event> {
        loop {
            let event = event::read()?;
            if let event::Event::Paste(s) = event {
                return Ok(Event::Insert(s));
            }
            if let event::Event::Key(event) = event {
                if !event.is_press() && !event.is_repeat() {
                    continue;
                }
                if let Some((_, event)) = self.config.get(&(event.code, event.modifiers)) {
                    return Ok(event.event.clone());
                }
                if let KeyCode::Char(c) = event.code {
                    return Ok(Event::Insert(c.to_string()));
                }
            }
        }
    }

    pub fn keybindings(&self) -> impl Iterator<Item = (&String, &String)> {
        self.config.values().map(|(key, event)| (key, &event.help))
    }
}
