use std::{error::Error, fs::File, io::{BufRead, BufReader, Read, Write}};

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
        let mut file = File::open(&self.filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let mut file = File::create(&self.filename)?;
        file.write_all(format!("{}\n", user.fullname).as_bytes())?;
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
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
