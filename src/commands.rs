use regex::{Regex};
use crate::editor::{Editor, StatusMessage};
use crate::{highlighting, Position, Terminal};
use termion::event::Key;

pub struct Command {
    pub regex: Regex,
    pub name: String,
    pub description: String,
    pub function: fn(editor: &mut Editor, params: Vec<&str>, forced: bool),
}

#[non_exhaustive]
pub struct Commands {
    pub commands: Vec<Command>,
}
impl Commands {
    #[must_use]
    #[allow(clippy::unwrap_used)]
    /// # Panics
    /// Will panic if the regexes fail to be created
    pub fn default() -> Self {
        let stock_commands = vec![
            Command {
                regex: Regex::new(r#"\b(q)\b"#).unwrap(),
                name: "q".to_owned(),
                description: "Quits Editor".to_owned(),
                function: |mut editor, _params, forced| {
                    if editor.document.is_dirty() && !forced {
                        editor.status_message = StatusMessage::from("There are unsaved changes. Run :q! to force quit".to_owned(), Option::from(crate::ERROR_COLOR));
                        return;
                    }
                    editor.should_quit = true;
                },
            },
            Command {
                regex: Regex::new(r#"\b(w)\b"#).unwrap(),
                name: "w".to_owned(),
                description: "Saves current document".to_owned(),
                function: |editor, _params, _forced| {
                    editor.save();
                },
            },
            Command {
                regex: Regex::new(r#"\b(wq)\b"#).unwrap(),
                name: "wq".to_owned(),
                description: "Saves current document and exits".to_owned(),
                function: |editor, _params, _forced| {
                    editor.should_quit = editor.save();
                },
            },
            Command {
                regex: Regex::new(r#"/"#).unwrap(),
                name: "/".to_owned(),
                description: "Searches document (top -> bottom)".to_owned(),
                function: |editor, params, _forced| {
                    Commands::search_command(editor, &params.join(" "), false, false);
                },
            },
            Command {
                regex: Regex::new(r#"\?"#).unwrap(),
                name: "?".to_owned(),
                description: "Searches document (bottom -> top)".to_owned(),
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
        let mut i: usize = if reverse { positions.len().saturating_sub(1) } else { 0 };
        let mut direction_just_jumped: isize = 1;
        if positions.is_empty() {
            if live_update {
                editor.status_message = StatusMessage::from( format!("/{} - No results found", query), Option::from(crate::ERROR_COLOR));
            } else {
                editor.status_message = StatusMessage::from("No results found".to_owned(), Option::from(crate::ERROR_COLOR));
            }

            return;
        }

        loop {
            if !live_update {
                editor.status_message = StatusMessage::from(format!("Search Mode - {}/{} (navigate = n / N)", i.saturating_add(1), &positions.len()), None);
            }
            if let Some(position) = positions.get(i) {
                let mut y;
                #[allow(clippy::integer_division)]
                if direction_just_jumped == 1 {
                    y = position.y.saturating_add(usize::from(editor.terminal.size().height / 2));
                } else {
                    y = position.y.saturating_sub(usize::from(editor.terminal.size().height / 2));
                }
                y = y.clamp(0, editor.document.len());
                editor.cursor_position = Position{ x: position.x, y };
                editor.scroll();
                editor.cursor_position = Position{ x: position.x, y: position.y };

                for p in &positions {
                    if let Some(row) = editor.document.row_mut(p.y) {
                        for c in (p.x)..(p.x.saturating_add(query.len())) {
                            if p.y == position.y {
                                row.add_highlighting(highlighting::Type::SearchSelected, c);
                            } else {
                                row.add_highlighting(highlighting::Type::Search, c);
                            }
                        }
                    }
                }
            }

            if live_update || editor.refresh_screen(true).is_err() {
                return;
            }

            editor.document.reset_highlighting();
            if let Ok(key) = Terminal::read_key() {
                match key {
                    Key::Char('n') => {
                        if i > 0 {
                            i = i.saturating_sub(1);
                        }
                        direction_just_jumped = -1;
                    }
                    Key::Char('N') => {
                        i = i.saturating_add(1);
                        direction_just_jumped = 1;
                    }
                    Key::Char('\n') | Key::Esc => break,
                    _ => (),
                }
                i = i.clamp(0, positions.len().saturating_sub(1));
        };
        }
        editor.status_message = StatusMessage::from("".to_owned(), None);
    }
    #[must_use]
    pub fn get_command(&self, command_name: &str) -> Option<&Command> {
        let mut command: Option<&Command> = None;
        for c in &self.commands {
            if c.regex.is_match(command_name) {
                command = Option::from(c);
            }
        }
        command
    }
}