use std::error::Error;

use common::User;
use rusqlite::Connection;

use super::UserStorage;

struct SqliteUser {
    id: u64,
    user: User,
}

#[derive(Debug, Clone)]
pub struct SqliteUserStorage {
    filename: String,
}

impl SqliteUserStorage {
    pub fn new(filename: &str) -> Self {
        SqliteUserStorage {
            filename: filename.to_string(),
        }
    }
}

impl UserStorage for SqliteUserStorage {
    fn create(&mut self, user: &User) -> Result<(), Box<dyn Error>> {
        let connection = Connection::open(&self.filename).unwrap();
        let query = "
            CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                fullname TEXT
            );
        ";
        connection.execute(query, ()).unwrap();
        let query = "INSERT INTO users (fullname) VALUES (?1);";
        connection.execute(query, [&user.fullname]).unwrap();
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<User>, Box<dyn Error>> {
        log::debug!("Reading all Users from '{}'", &self.filename);
        let connection = Connection::open(&self.filename).unwrap();
        let query = "SELECT id, fullname FROM users";
        let mut stmt = connection.prepare(query).unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut users = Vec::new();
        while let Some(row) = rows.next()? {
            let fullname: String = row.get(1).unwrap();
            users.push(SqliteUser {
                id: row.get(0)?,
                user: User::new(fullname.as_str()),
            });
        }
        Ok(users.iter().map(|u| u.user.to_owned()).collect())
    }
}
