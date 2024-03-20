use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write}, path::Path,
};

use common::User;

use super::UserStorage;

#[derive(Debug, Clone)]
pub struct CsvUserStorage {
    filename: String,
}

impl CsvUserStorage {
    pub fn new(filename: &str) -> Self {
        CsvUserStorage {
            filename: filename.to_string(),
        }
    }
}

impl UserStorage for CsvUserStorage {
    fn create(&mut self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut file = if Path::new(&self.filename).exists() {
            OpenOptions::new().write(true).append(true).open(&self.filename).unwrap()
        } else {
            OpenOptions::new().create_new(true).write(true).open(&self.filename).unwrap()
        };
        if let Err(e) = writeln!(file, "{}", user.fullname) {
            Err(Box::new(e))
        } else {
            Ok(())
        }
    }

    fn read_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
        log::debug!("Reading all Users from '{}'", &self.filename);
        let file = File::open(&self.filename)?;
        let reader = BufReader::new(file);
        let mut users = Vec::new();

        for line in reader.lines() {
            let fullname = line?;
            users.push(User { fullname });
        }
        Ok(users)
    }
}
