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

use termion::color;
pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;
pub use commands::Commands;

pub const ERROR_COLOR: color::Rgb = color::Rgb(197, 15, 31);

fn main() {
    Editor::default().run();
}

/*
TODO (GENERAL IDEAS):
    - Add VIM style commands section using the prompt functionality
    - Add customisable colors
    - Add customisable syntax highlighting
    - Add more navigation features (search, goto, etc)
 */

/*
TODO (VIM LIKE COMMANDS):
    - Modular commands system
    - Each script placed in a directory is auto loaded
    - A struct defines the command name and description]
    - There is an execution function that can interface with the document and editor functions
    - Allows for VIM like commands
    - Also load .rs scripts from a folder in home directory to allow for modding
    - .rs scripts can also color certain things by hooking into an syntax highlighter system
 */