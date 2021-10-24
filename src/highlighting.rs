use termion::color::{Rgb};

#[derive(PartialEq, Clone)]
pub enum Type {
    SearchSelected,
    Search,
    None,
}

pub struct Highlight {
    pub bg_color: Rgb,
    pub fg_color: Rgb,
}

impl Type {
    pub fn to_color(&self) -> Highlight {
        match self {
            Type::Search => Highlight { fg_color: Rgb(0, 0, 0), bg_color: Rgb(249, 241, 165) },
            Type::SearchSelected => Highlight { fg_color: Rgb(0, 0, 0), bg_color: Rgb(255, 255, 255) },
            _ => Highlight { fg_color: Rgb(255, 255, 255), bg_color: Rgb(0, 0, 0) },
        }
    }
}