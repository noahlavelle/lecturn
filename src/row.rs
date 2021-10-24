use std::cmp;
use unicode_segmentation::UnicodeSegmentation;
use crate::highlighting;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
    highlighting: Vec<highlighting::Type>
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len: slice.graphemes(true).count(),
            highlighting: vec![highlighting::Type::None; slice.len()],
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        #[allow(clippy::integer_arithmetic)]
        for (index, grapheme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self
                    .highlighting
                    .get(index)
                    .unwrap_or(&highlighting::Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let highlight = highlighting_type.to_color();
                    let start_highlight;
                    if *highlighting_type == highlighting::Type::None {
                        start_highlight =
                            format!("{}{}", termion::color::Bg(termion::color::Reset), termion::color::Fg(termion::color::Reset));
                    } else {
                        start_highlight =
                            format!("{}{}", termion::color::Bg(highlight.bg_color), termion::color::Fg(highlight.fg_color));
                    }

                    result.push_str(&start_highlight[..]);
                }
                if c == '\t' {
                    result.push_str("    ");
                } else {
                    result.push(c);
                }
            }
        }
        let end_highlight = format!("{}{}", termion::color::Bg(termion::color::Reset), termion::color::Fg(termion::color::Reset));
        result.push_str(&end_highlight[..]);
        result
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len = length;
        self.string = result;
    }
    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut result: String = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }
    pub fn append(&mut self, new: &Self)  {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }
    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut split_row: String = String::new();
        let mut split_length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                split_length += 1;
                split_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;
        Self {
            string: split_row,
            len: split_length,
            highlighting: vec!(),
        }
    }
    pub fn find(&self, query: &str) -> Option<usize> {
        self.string.find(query)
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
    pub fn add_highlighting(&mut self, highlight_type: highlighting::Type, index: usize) {
        self.highlighting[index] = highlight_type;
    }
    pub fn reset_highlighting(&mut self) {
        self.highlighting = vec![highlighting::Type::None; self.string.len()]
    }
}
