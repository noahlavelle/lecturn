use crate::Position;
use std::io::{self, stdout, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    /// # Errors
    /// Will return `Err` if the `termion` dependency fails
    /// to initiate raw mode
    pub fn default() -> Result<Self, std::io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }
    #[must_use] pub fn size(&self) -> &Size {
        &self.size
    }
    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
    pub fn cursor_position(position: &Position) {
        let &Position { x, y } = position;
        let x: u16 = x.saturating_add(1) as u16;
        let y: u16 = y.saturating_add(1) as u16;
        print!("{}", termion::cursor::Goto(x, y));
    }
    /// # Errors
    /// Will return `Err` if `stdout` fails to flush
    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }
    /// # Errors
    /// Will return `Err` if `stdin` fails to collect key presses
    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }
    pub fn cursor_hide() {
        print!("{}", termion::cursor::Hide);
    }
    pub fn cursor_show() {
        print!("{}", termion::cursor::Show);
    }
    pub fn cursor_bar() {
        print!("{}", termion::cursor::BlinkingBar);
    }
    pub fn cursor_block() {
        print!("{}", termion::cursor::BlinkingBlock);
    }
    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }
    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }
    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }
}
