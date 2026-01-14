use nucleo::Utf32String;
use std::collections::HashSet;

/// fzf result
pub struct FzfString<S: AsRef<str> = String> {
    s: S,
    indices: HashSet<u32>,
}

impl<S: AsRef<str>> FzfString<S> {
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

/// Simple wrapper around nucleo
pub fn fzf<S: AsRef<str>>(
    items: impl Iterator<Item = S>,
    prompt: &str,
) -> impl Iterator<Item = FzfString<S>> {
    let mut res = Vec::new();

    let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
    for item in items {
        let mut indices = Vec::new();
        let score = matcher.fuzzy_indices(
            Utf32String::from(item.as_ref()).slice(0..item.as_ref().len()),
            Utf32String::from(prompt).slice(0..prompt.len()),
            &mut indices,
        );

        if score.is_some() {
            res.push((item, score.unwrap(), indices));
        }
    }

    res.sort_by_key(|(_, score, _)| u16::MAX - score);
    res.into_iter().map(|(s, _, indices)| FzfString {
        s: s,
        indices: HashSet::from_iter(indices.into_iter()),
    })
}
