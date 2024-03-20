use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use common::User;

use super::{Crud, CrudError};

#[derive(Debug, Clone)]
pub struct CsvUserStore {
    filename: String,
}

impl CsvUserStore {
    pub fn new(filename: &str) -> Self {
        CsvUserStore {
            filename: filename.to_string(),
        }
    }
}

impl Crud<User> for CsvUserStore {
    fn create(&mut self, user: &User) -> Result<(), CrudError> {
        let mut file = if Path::new(&self.filename).exists() {
            OpenOptions::new()
                .write(true)
                .append(true)
                .open(&self.filename)
                .unwrap()
        } else {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&self.filename)
                .unwrap()
        };
        if let Err(_) = writeln!(file, "{}", user.fullname) {
            Err(CrudError::UnknownError)
        } else {
            Ok(())
        }
    }

    fn read_all(&self) -> Result<Vec<User>, CrudError> {
        log::debug!("Reading Users from '{}'", &self.filename);
        let file = if Path::new(&self.filename).exists() {
            OpenOptions::new().read(true).open(&self.filename).unwrap()
        } else {
            return Ok(Vec::new());
        };

        let reader = BufReader::new(file);
        let mut users = Vec::new();

        for line in reader.lines() {
            if let Ok(fullname) = line {
                users.push(User { fullname });
            }
        }
        log::debug!("Read {} Users from '{}'", users.len(), &self.filename);
        Ok(users)
    }
}
