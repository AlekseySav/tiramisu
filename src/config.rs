use crate::paths;

use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use serde_valid::Validate;
use serde_with::{DisplayFromStr, DurationSeconds, serde_as};
use std::path::{Path, PathBuf};

#[serde_inline_default]
#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    /// Logger configuration
    pub logger: Logger,

    /// List of sessions
    pub session: Vec<Session>,

    /// Whether to show initial help message
    #[serde_inline_default(true)]
    pub show_help: bool,
}

#[serde_inline_default]
#[serde_as]
#[derive(Debug, Deserialize, Validate)]
pub struct Logger {
    /// Log level filter (off error warn info debug trace)
    #[serde_as(as = "DisplayFromStr")]
    pub level: log::LevelFilter,

    /// How long to keep messages popup (in seconds)
    #[serde_as(as = "DurationSeconds<f64>")]
    #[serde_inline_default(chrono::Duration::milliseconds(5000))]
    pub message_ttl: chrono::Duration,

    /// Log path
    #[serde(default = "paths::logs")]
    pub log_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Session {
    /// Session root dir, may be glob
    pub root: PathBuf,

    /// Session name
    pub name: String,

    /// List of windows
    #[validate(min_items = 1)]
    pub window: Vec<Window>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct Window {
    /// Window name
    pub name: String,

    /// Window startup command
    #[serde(default)]
    pub command: String,

    /// Command to send to safely kill window
    #[serde(default)]
    pub kill: Vec<String>,
}

impl Config {
    /// Reads and preprocesses configuration
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
        config.validate()?;
        config.expand()?;
        Ok(config)
    }

    // This code is cursed
    fn expand(&mut self) -> anyhow::Result<()> {
        let mut sessions: Vec<Session> = Vec::new();
        for session in self.session.iter() {
            for entry in capturing_glob::glob(&replace_env(session.root.to_str().unwrap(), None))? {
                match entry {
                    Ok(e) if e.path().is_dir() => sessions.push(Session {
                        root: e.path().to_path_buf(),
                        name: replace_env(&session.name, Some(&e)),
                        window: session
                            .window
                            .iter()
                            .map(|w| Window {
                                name: replace_env(&w.name, Some(&e)),
                                command: replace_env(&w.command, Some(&e)),
                                kill: w.kill.clone(),
                            })
                            .collect(),
                    }),
                    Err(e) => return Err(e.into()),
                    _ => continue,
                }
            }
        }
        self.session = sessions;

        Ok(())
    }
}

fn get_var(name: &str, e: Option<&capturing_glob::Entry>) -> String {
    match str::parse::<usize>(name) {
        Ok(n) => e.map_or(String::new(), |e| {
            e.group(n)
                .map_or(String::new(), |s| s.to_str().unwrap_or_default().into())
        }),
        Err(_) => std::env::var(name).unwrap_or_default(),
    }
}

fn replace_env(p: &str, e: Option<&capturing_glob::Entry>) -> String {
    let mut prev = '\0';
    let mut res = String::new();
    let mut varname = String::new();
    let mut queue = &mut res;
    for mut c in p.chars() {
        if c.is_alphanumeric() || c == '_' {
            queue.push(c);
            prev = c;
            continue;
        }
        queue = &mut res;
        queue.push_str(&get_var(&varname, e));
        varname.clear();
        match (prev, c) {
            ('\\', '\\') => c = '\0',
            ('\\', '$') => {
                queue.pop();
                queue.push('$');
            }
            (_, '$') => queue = &mut varname,
            (_, c) => queue.push(c),
        }
        prev = c;
    }
    res.push_str(&get_var(&varname, e));
    res
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     fn argv(i: usize) -> Option<String> {
//         match i {
//             0 => Some("123".into()),
//             1 => Some("456".into()),
//             _ => None,
//         }
//     }
//
//     #[test]
//     fn test_replace_env() {
//         temp_env::with_vars([("a", Some("hello")), ("bebebe", Some("lalala"))], || {
//             assert_eq!(replace_env("$aa$aaa$bebeb$bebebeb$", argv), "");
//             assert_eq!(replace_env("\\$\\$asda\\$\\\\a\\", argv), "$$asda$\\a\\");
//             assert_eq!(replace_env("$a", argv), "hello");
//             assert_eq!(replace_env("$bebebe", argv), "lalala");
//             assert_eq!(replace_env("qwe$bebebe!r", argv), "qwelalala!r");
//
//             assert_eq!(
//                 replace_env("$a$bebebe$a$bebebe", argv),
//                 "hellolalalahellolalala"
//             );
//             assert_eq!(replace_env("\\$$a", argv), "$hello");
//             assert_eq!(replace_env("\\$a", argv), "$a");
//             assert_eq!(replace_env("\\\\$a", argv), "\\hello");
//
//             assert_eq!(replace_env("$0", argv), "123");
//             assert_eq!(replace_env("$1", argv), "456");
//         })
//     }
// }
