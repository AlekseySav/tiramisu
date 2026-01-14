use std::{collections::HashMap, env};

use anyhow::Result;
use capturing_glob::glob;
use serde::Deserialize;
use serde_valid::Validate;

const CONFIG_PATH: &str = "examples/config.toml";

#[derive(Debug)]
pub struct Config {
    pub session: HashMap<String, Session>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct Session {
    #[validate(pattern = "^[^.:]*$")]
    pub path: String,
    #[validate(pattern = "^[^.:]*$")]
    pub name: String,
    pub window: Vec<Window>,
}

#[derive(Deserialize, Validate, Clone, Debug)]
pub struct Window {
    #[validate(pattern = "^[^.:]*$")]
    pub name: String,
    pub cmd: String,
}

#[derive(Deserialize, Validate, Debug)]
struct RawConfig {
    pub session: Vec<Session>,
}

impl Config {
    /// Reads and preprocesses configuration
    pub fn new() -> Result<Self> {
        let config: RawConfig = toml::from_str(&std::fs::read_to_string(CONFIG_PATH)?)?;
        config.validate()?;
        Self::from_raw(&config)
    }

    // This code is cursed
    fn from_raw(raw: &RawConfig) -> Result<Self> {
        let mut sessions: HashMap<String, Session> = HashMap::new();
        for session in raw.session.iter() {
            for entry in glob(&replace_env(&session.path, |_| None))? {
                match entry {
                    Ok(e) if e.path().is_dir() => sessions.insert(
                        replace_env(&session.name, |n| {
                            e.group(n).map(|s| s.to_str().unwrap_or_default().into())
                        }),
                        Session {
                            path: e.path().display().to_string(),
                            name: replace_env(&session.name, |n| {
                                e.group(n).map(|s| s.to_str().unwrap_or_default().into())
                            }),
                            window: session
                                .window
                                .iter()
                                .map(|w| Window {
                                    name: replace_env(&w.name, |n| {
                                        e.group(n).map(|s| s.to_str().unwrap_or_default().into())
                                    }),
                                    cmd: replace_env(&w.cmd, |n| {
                                        e.group(n).map(|s| s.to_str().unwrap_or_default().into())
                                    }),
                                })
                                .collect(),
                        },
                    ),
                    Err(e) => return Err(e.into()),
                    _ => continue,
                };
            }
        }

        Ok(Self { session: sessions })
    }
}

fn get_var<F: Fn(usize) -> Option<String>>(name: &str, argv: &F) -> String {
    match str::parse::<usize>(name) {
        Ok(n) => argv(n).unwrap_or_default(),
        Err(_) => env::var(name).unwrap_or_default(),
    }
}

fn replace_env<F: Fn(usize) -> Option<String>>(p: &str, argv: F) -> String {
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
        queue.push_str(&get_var(&varname, &argv));
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
    res.push_str(&get_var(&varname, &argv));
    res
}

#[cfg(test)]
mod test {
    use super::*;

    fn argv(i: usize) -> Option<String> {
        match i {
            0 => Some("123".into()),
            1 => Some("456".into()),
            _ => None,
        }
    }

    #[test]
    fn test_replace_env() {
        temp_env::with_vars([("a", Some("hello")), ("bebebe", Some("lalala"))], || {
            assert_eq!(replace_env("$aa$aaa$bebeb$bebebeb$", argv), "");
            assert_eq!(replace_env("\\$\\$asda\\$\\\\a\\", argv), "$$asda$\\a\\");
            assert_eq!(replace_env("$a", argv), "hello");
            assert_eq!(replace_env("$bebebe", argv), "lalala");
            assert_eq!(replace_env("qwe$bebebe!r", argv), "qwelalala!r");

            assert_eq!(
                replace_env("$a$bebebe$a$bebebe", argv),
                "hellolalalahellolalala"
            );
            assert_eq!(replace_env("\\$$a", argv), "$hello");
            assert_eq!(replace_env("\\$a", argv), "$a");
            assert_eq!(replace_env("\\\\$a", argv), "\\hello");

            assert_eq!(replace_env("$0", argv), "123");
            assert_eq!(replace_env("$1", argv), "456");
        })
    }
}
