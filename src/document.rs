use crate::{Position, Row};
use std::fs;
use std::io::{Write, Error};

pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

impl Document {
    #[must_use]
    pub fn default() -> Self {
        let rows = vec![Row::default()];
        Self {
            rows,
            file_name: None,
            dirty: false,
        }
    }

    /// # Errors
    /// Will return `Err` if fs fails to open the file (invalid permissions / not found)
    pub fn open(filename: &str) -> Result<Self, Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self{
            rows,
            file_name: Some(filename.to_owned()),
            dirty: false,
        })
    }
    #[must_use] pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    pub fn row_mut(&mut self, index: usize) -> Option<&mut Row> {
        self.rows.get_mut(index)
    }
    #[must_use] pub fn is_empty(&self) -> bool {
        if let Some(first_row) = self.row(0) {
            if first_row.is_empty() {
                return true;
            }
        }
        self.rows.len() == 1
    }
    #[must_use] pub fn len(&self) -> usize {
        self.rows.len()
    }
    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            return;
        }
        if at.y == self.rows.len() {
            self.rows.push(Row::default());
            return;
        }
        #[allow(clippy::indexing_slicing)]
        let new_row = self.rows[at.y].split(at.x);
        #[allow(clippy::integer_arithmetic)]
        self.rows.insert(at.y + 1, new_row);
    }
    pub fn insert (&mut self, at: &Position, c: char) {
        if at.y > self.rows.len() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
        }
    }
    #[allow(clippy::integer_arithmetic, clippy::indexing_slicing)]
    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }
    /// # Errors
    /// Will return `Err` if fs cannot write to the document (Missing
    /// permissions / document not found)
    pub fn save(&mut self) -> Result<(), Error> {
        #[allow(clippy::pattern_type_mismatch)]
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }
    #[must_use] pub fn find(&self, query: &str) -> Vec<Position> {
        let mut positions: Vec<Position> = vec!();
        for i in 0..self.len() {
            if let Some(row) = self.row(i) {
                if let Some(position) = row.find(query) {
                    positions.push(Position{x: position, y: i});
                }
            }
        }
        positions
    }
    #[must_use] pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn reset_highlighting(&mut self) {
        for i in 0..self.rows.len() {
            if let Some(row) = self.row_mut(i) {
                row.reset_highlighting();
            }
        }
    }
    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = Option::from(file_name);
    }
}