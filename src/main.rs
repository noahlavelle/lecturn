#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else
)]
mod document;
mod editor;
mod row;
mod terminal;
mod commands;
mod highlighting;

use termion::color;
pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;
pub use commands::Commands;
pub use highlighting::Type as HighlightType;

pub const ERROR_COLOR: color::Rgb = color::Rgb(197, 15, 31);

fn main() {
    Editor::default().run();
}

/*
TODO (GENERAL IDEAS):
    - Add customisable colors
    - Add customisable syntax highlighting
    - Add more navigation features (goto, etc)
 */