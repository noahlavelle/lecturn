use crate::{Commands, Document};
use crate::Row;
use crate::Terminal;
use std::env;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::color::Rgb;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(0, 0, 0);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[derive(PartialEq, Eq)]
enum InteractionMode {
    Command,
    Insert,
}

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct StatusMessage {
    text: String,
    time: Instant,
    color: Option<Rgb>,
}
impl StatusMessage {
    pub fn from(message: String, color: Option<Rgb>) -> Self {
        Self {
            time: Instant::now(),
            text: message,
            color,
        }
    }
}

pub struct Editor {
    pub should_quit: bool,
    terminal: Terminal,
    pub(crate) cursor_position: Position,
    offset: Position,
    pub document: Document,
    pub status_message: StatusMessage,
    quit_times: u8,
    interaction_mode: InteractionMode,
    command_handler: Commands,
    just_entered: bool,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::cursor_block();
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = StatusMessage::from("".to_string(), None);
        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status.text = format!("ERR: Could not open file: {}", file_name);
                initial_status.color = Option::from(crate::ERROR_COLOR);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: initial_status,
            quit_times: QUIT_TIMES,
            interaction_mode: InteractionMode::Command,
            command_handler: Commands::default(),
            just_entered: true,
        }
    }

    pub(crate) fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }
    pub fn save(&mut self) -> bool {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ").unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string(), Option::from(crate::ERROR_COLOR));
                return false;
            }
            self.document.file_name = new_name;
        }

        return if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully".to_string(), None);
            true
        } else {
            self.status_message = StatusMessage::from("ERR: could not write to file".to_string(), Option::from(crate::ERROR_COLOR));
            false
        }
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Esc => {
                self.interaction_mode = InteractionMode::Command;
                Terminal::cursor_block();
            }
            Key::Char(c) => {
                if self.interaction_mode == InteractionMode::Command {
                    match pressed_key {
                        Key::Char('i') => {
                            self.just_entered = false;
                            self.interaction_mode = InteractionMode::Insert;
                            Terminal::cursor_bar();
                        }
                        Key::Char(':') => {
                            let command_name = self.prompt(":").unwrap_or(None);
                            if command_name.is_none() {
                                self.status_message = StatusMessage::from("ERR: Command aborted".to_string(), Option::from(crate::ERROR_COLOR));
                                return Ok(());
                            }
                            let command_name = command_name.unwrap();
                            let is_forced = command_name.contains("!");
                            let command = self.command_handler.get_command(&command_name);
                            if command.is_none() {
                                self.status_message = StatusMessage::from("ERR: Invalid command".to_string(), Option::from(crate::ERROR_COLOR));
                            } else {
                                let command_params = command.unwrap().regex.replace(&command_name.to_string(), "").to_string();
                                (command.unwrap().function)(self, command_params.split(" ").collect(), is_forced);
                            }
                        },
                        _ => (),
                    }
                } else {
                    self.document.insert(&self.cursor_position, c);
                    self.move_cursor(Key::Right);
                }
            },
            Key::Delete => self.document.delete(&self.cursor_position),
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new(), None);
        }
        Ok(())
    }
    pub(crate) fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }
    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }
    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        Terminal::set_fg_color(color::Rgb(59, 120, 255));
        print!("~");
        Terminal::reset_fg_color();
        println!("{}\r", welcome_message);
    }
    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row)
    }
    #[allow(clippy::integer_arithmetic, clippy::integer_division)]
    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 && self.just_entered {
                self.draw_welcome_message();
            } else {
                Terminal::set_fg_color(color::Rgb(59, 120, 255));
                println!("~\r");
                Terminal::reset_fg_color();
            }
        }
    }
    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " [+]"
        } else {
            ""
        };
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{}{}",
            file_name,
            modified_indicator
        );

        let position_indicator = format!(
            "{},{}",
            self.cursor_position.y.saturating_add(1),
            self.cursor_position.x.saturating_add(1),
        );
        #[allow(clippy::integer_arithmetic)]
        let len = status.len() + position_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, position_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }
    fn draw_message_bar(&mut self) {
        Terminal::clear_current_line();
        if self.interaction_mode == InteractionMode::Insert {
            self.status_message = StatusMessage::from("-- INSERT --".to_string(), None);
        } else if self.status_message.text == "-- INSERT --".to_string() {
            self.status_message = StatusMessage::from("".to_string(), None);
        }

        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            if !message.color.is_none() {
                Terminal::set_bg_color(message.color.unwrap());
            }
            Terminal::set_fg_color(color::Rgb(255, 255, 255));
            print!("{}", text);
            Terminal::reset_bg_color();
            Terminal::reset_fg_color();
        }
    }
    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result), None);
            self.refresh_screen()?;
            match Terminal::read_key()? {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len().saturating_sub(1));
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
        }
        self.status_message = StatusMessage::from(String::new(), None);
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
