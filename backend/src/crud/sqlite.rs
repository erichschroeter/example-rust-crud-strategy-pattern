use common::User;
use rusqlite::Connection;

use super::{Crud, CrudError};

struct SqliteUser {
    _id: u64,
    user: User,
}

#[derive(Debug, Clone)]
pub struct SqliteUserStore {
    filename: String,
}

impl SqliteUserStore {
    pub fn new(filename: &str) -> Self {
        SqliteUserStore {
            filename: filename.to_string(),
        }
    }
}

impl Crud<User> for SqliteUserStore {
    fn create(&mut self, user: &User) -> Result<(), CrudError> {
        let connection = Connection::open(&self.filename).unwrap();
        let query = "
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                fullname TEXT
            );
        ";
        connection.execute(query, ()).unwrap();
        let query = "INSERT INTO users (fullname) VALUES (?1);";
        connection.execute(query, [&user.fullname]).unwrap();
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<User>, CrudError> {
        log::debug!("Reading all Users from '{}'", &self.filename);
        if let Ok(connection) = Connection::open(&self.filename) {
            let query = "SELECT id, fullname FROM users";
            if let Ok(mut stmt) = connection.prepare(query) {
                let mut rows = stmt.query([]).unwrap();
                let mut users = Vec::new();
                while let Some(row) = rows.next()? {
                    let fullname: String = row.get(1).unwrap();
                    users.push(SqliteUser {
                        _id: row.get(0)?,
                        user: User::new(fullname.as_str()),
                    });
                }
                Ok(users.iter().map(|u| u.user.to_owned()).collect())
            } else {
                log::warn!("Error encountered preparing statement");
                Ok(Vec::new())
            }
        } else {
            log::warn!("Error opening SQLite connection");
            Ok(Vec::new())
        }
    }
}
