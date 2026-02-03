use anyhow::{Context, Result};

/// Substitutes enviorment variables
pub fn expand<S: AsRef<str>>(p: S, args: impl Fn(&String) -> Result<String>) -> Result<String> {
    let mut prev = '\0';
    let mut res = String::new();
    let mut varname = String::new();
    let mut queue = &mut res;
    for mut c in p.as_ref().chars() {
        if c.is_alphanumeric() || c == '_' {
            queue.push(c);
            prev = c;
            continue;
        }
        queue = &mut res;
        queue.push_str(&args(&varname)?);
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
    res.push_str(&args(&varname)?);
    Ok(res)
}

/// Only enviorment variables
pub fn env(s: &String) -> Result<String> {
    if s.is_empty() {
        return Ok(String::new());
    }
    std::env::var(s).with_context(|| format!("Failed to substitute variable '${s}'"))
}
