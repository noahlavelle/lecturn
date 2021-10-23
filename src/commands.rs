use regex::{Regex};
use crate::editor::{Editor, StatusMessage};
use crate::{Position, Terminal};
use termion::event::Key;

pub struct Command {
    pub regex: Regex,
    pub name: String,
    pub description: String,
    pub function: fn(editor: &mut Editor, params: Vec<&str>, forced: bool),
}

pub struct Commands {
    pub commands: Vec<Command>,
}
impl Commands {
    pub fn default() -> Self {
        let stock_commands = vec![
            Command {
                regex: Regex::new(r#"\b(q)\b"#).unwrap(),
                name: "q".to_string(),
                description: "Quits Editor".to_string(),
                function: |mut editor, _params, forced| {
                    if editor.document.is_dirty() && !forced {
                        editor.status_message = StatusMessage::from("There are unsaved changes. Run :q! to force quit".to_string());
                        return;
                    }
                    editor.should_quit = true
                },
            },
             Command {
             regex: Regex::new(r#"\b(w)\b"#).unwrap(),
                name: "w".to_string(),
                description: "Saves current document".to_string(),
                function: |editor, _params, _forced| {
                    editor.save();
                },
            },
            Command {
                regex: Regex::new(r#"\b(wq)\b"#).unwrap(),
                name: "wq".to_string(),
                description: "Saves current document and exits".to_string(),
                function: |editor, _params, _forced| {
                    editor.should_quit = editor.save();
                },
            },
            Command {
                regex: Regex::new(r#"/"#).unwrap(),
                name: "/".to_string(),
                description: "Searches document (top -> bottom)".to_string(),
                function: |editor, params, _forced| {
                    Commands::search_command(editor, params[0], false);
                },
            },
            Command {
                regex: Regex::new(r#"\?"#).unwrap(),
                name: "?".to_string(),
                description: "Searches document (bottom -> top)".to_string(),
                function: |editor, params, _forced| {
                    Commands::search_command(editor, params[0], true);
                },
            },
        ];
        Self {
            commands: stock_commands,
        }
    }
    pub fn search_command(editor: &mut Editor, query: &str, reverse: bool) {
        let mut positions: Vec<Position> = editor.document.find(query);
        let mut i: i8 = if reverse { positions.len() - 1 } else { 0 } as i8;
        if positions.is_empty() {
            editor.status_message = StatusMessage::from("No results found".to_string());
            return;
        }
        loop {
            editor.status_message = StatusMessage::from(format!("Search Mode - {}/{} (navigate = n / N)", i + 1, positions.len()));
            if let Some(position) = positions.get(i as usize) {
                editor.cursor_position = Position{ x: position.x, y: position.y };
                let _ = editor.refresh_screen();
            }

            match Terminal::read_key().unwrap() {
                Key::Char('n') => {
                    i -= 1;
                }
                Key::Char('N') => {
                    i += 1;
                }
                Key::Char('\n') => break,
                Key::Esc => break,
                _ => (),
            }
            i = i.clamp(0, (positions.len() - 1) as i8)
        }
        editor.status_message = StatusMessage::from("".to_string());
    }
    pub fn get_command(&self, command_name: &String) -> Option<&Command> {
        let mut command: Option<&Command> = None;
        for c in &self.commands {
            if c.regex.is_match(&*command_name) {
                command = Option::from(c);
            }
        }
        command
    }
}