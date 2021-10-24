use regex::{Regex};
use crate::editor::{Editor, StatusMessage};
use crate::{HighlightType, Position, Terminal};
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
                        editor.status_message = StatusMessage::from("There are unsaved changes. Run :q! to force quit".to_string(), Option::from(crate::ERROR_COLOR));
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
                    Commands::search_command(editor, &params.join(" "), false, false);
                },
            },
            Command {
                regex: Regex::new(r#"\?"#).unwrap(),
                name: "?".to_string(),
                description: "Searches document (bottom -> top)".to_string(),
                function: |editor, params, _forced| {
                    Commands::search_command(editor, &params.join(" "), true, false);
                },
            },
        ];
        Self {
            commands: stock_commands,
        }
    }
    pub fn search_command(editor: &mut Editor, query: &str, reverse: bool, live_update: bool) {
        let positions: Vec<Position> = editor.document.find(query);
        let mut i: i8 = if reverse { positions.len() - 1 } else { 0 } as i8;
        let mut direction_just_jumped = 1;
        if positions.is_empty() {
            if live_update {
                editor.status_message = StatusMessage::from( format!("/{} - No results found", query), Option::from(crate::ERROR_COLOR));
            } else {
                editor.status_message = StatusMessage::from("No results found".to_string(), Option::from(crate::ERROR_COLOR));
            }

            return;
        }

        loop {
            if !live_update {
                editor.status_message = StatusMessage::from(format!("Search Mode - {}/{} (navigate = n / N)", i + 1, &positions.len()), None);
            }
            if let Some(position) = positions.get(i as usize) {
                let mut y: usize;
                if direction_just_jumped == 1 {
                    y = position.y.saturating_add((editor.terminal.size().height / 2) as usize);
                } else {
                    y = position.y.saturating_sub((editor.terminal.size().height / 2) as usize);
                }
                y = y.clamp(0, editor.document.len());
                editor.cursor_position = Position{ x: position.x, y };
                editor.scroll();
                editor.cursor_position = Position{ x: position.x, y: position.y };

                for p in &positions {
                    let row = editor.document.row_mut(p.y).unwrap();
                    for c in (p.x)..(p.x + query.len()) {
                        if p.y == position.y as usize {
                            row.add_highlighting(HighlightType::SearchSelected, c);
                        } else {
                            row.add_highlighting(HighlightType::Search, c);
                        }

                    }
                }
            }

            if live_update {
                return;
            }

            let _ = editor.refresh_screen();
            editor.document.reset_highlighting();

            match Terminal::read_key().unwrap() {
                Key::Char('n') => {
                    i -= 1;
                    direction_just_jumped = -1;
                }
                Key::Char('N') => {
                    i += 1;
                    direction_just_jumped = 1;
                }
                Key::Char('\n') => break,
                Key::Esc => break,
                _ => (),
            }
            i = i.clamp(0, positions.len() as i8 - 1);
        }
        editor.status_message = StatusMessage::from("".to_string(), None);
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