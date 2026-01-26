use std::collections::HashMap;

use crossterm::event;

pub enum Event {
    Char { char: char },
    Ctrl { char: char },
    Tmux { p: String },
}

pub struct Keymap {
    subs: HashMap<char, (Box<dyn Fn(&Event)>, String)>,
}

impl Keymap {
    pub fn set<S: AsRef<str>, F: Fn(&Event)>(&mut self, c: char, action: F, help: S) {
        self.subs
            .insert(c, (Box::new(action), help.as_ref().to_string()));
    }
}

pub struct EventDispatcher {
    subs: HashMap<Event, Box<dyn Fn(&Event)>>,
}

impl EventDispatcher {
    pub fn set<S: AsRef<S>, F: Fn(&Event)>(&mut self, key: S, action: F, help: S) {}
}
