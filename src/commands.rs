use regex::Regex;
use crate::editor::{Editor, StatusMessage};

pub struct Command {
    pub regex: Regex,
    pub name: String,
    pub description: String,
    pub function: fn(editor: &mut Editor, forced: bool),
}

pub struct Commands {
    pub commands: Vec<Command>,
}
impl Commands {
    pub fn default() -> Self {
        let stock_commands = vec![
            Command {
                regex: Regex::new(r#"\b(q)\b"#).unwrap(),
                name: "exit".to_string(),
                description: "Quits Editor".to_string(),
                function: |mut editor, forced| {
                    if editor.document.is_dirty() && !forced {
                        editor.status_message = StatusMessage::from("There are unsaved changes. Run :q! to force quit".to_string());
                        return;
                    }
                    editor.should_quit = true
                },
            },
             Command {
             regex: Regex::new(r#"\b(w)\b"#).unwrap(),
                name: "save".to_string(),
                description: "Saves current document".to_string(),
                function: |editor, forced| {
                    editor.save();
                },
            },
            Command {
                regex: Regex::new(r#"\b(wq)\b"#).unwrap(),
                name: "save exit".to_string(),
                description: "Saves current document and exits".to_string(),
                function: |editor, forced| {
                    editor.should_quit = editor.save();
                },
            },
            Command {
                regex: Regex::new(r#"/(.*)"#).unwrap(),
                name: "search".to_string(),
                description: "Searches document (top -> bottom)".to_string(),
                function: |editor, forced| {
                    editor.save();
                    editor.should_quit = true;
                },
            },
        ];
        Self {
            commands: stock_commands,
        }
    }
    pub fn get_command(&self, command_name: String) -> Option<&Command> {
        let mut command: Option<&Command> = None;
        for c in &self.commands {
            if c.regex.is_match(&*command_name) {
                command = Option::from(c);
            }
        }
        command
    }
}