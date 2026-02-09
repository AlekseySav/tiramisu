use getset::Getters;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default, Getters)]
pub struct Input {
    #[getset(get = "pub")]
    /// Get prompt data
    data: String,
    /// Get cursor position (position to be displayed, not position within string)
    #[getset(get = "pub")]
    cursor: usize,
}

impl Input {
    /// Insert data at cursor position
    pub fn insert<S: AsRef<str>>(&mut self, s: S) {
        let mut data = graphemes(&self.data);
        for c in graphemes(s.as_ref()) {
            data.insert(self.cursor, c);
            self.cursor += 1;
        }
        self.data = data.join("");
        self.set_cursor(0, false);
    }

    /// Move cursor
    pub fn set_cursor(&mut self, offset: isize, absolute: bool) {
        self.cursor = self.get_offset(&graphemes(&self.data), offset, absolute);
    }

    /// Delete from cursor till specified position
    pub fn delete(&mut self, offset: isize, absolute: bool) {
        let data = graphemes(&self.data);
        let n = self.get_offset(&data, offset, absolute);
        let (start, end) = (self.cursor.min(n), self.cursor.max(n));
        self.data = data
            .into_iter()
            .enumerate()
            .filter(|(i, _)| *i < start || *i >= end)
            .fold(String::new(), |a, (_, b)| a + b);
        self.set_cursor(start as isize, true);
    }

    fn get_offset(&self, g: &Vec<&str>, offset: isize, absolute: bool) -> usize {
        if absolute {
            if offset < 0 { g.len() } else { offset as usize }
        } else {
            self.cursor.saturating_add_signed(offset).min(g.len())
        }
    }

    fn len(&self) -> usize {
        graphemes(&self.data).len()
    }
}

fn graphemes(s: &str) -> Vec<&str> {
    UnicodeSegmentation::graphemes(s, true).collect()
}
