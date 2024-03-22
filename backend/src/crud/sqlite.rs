use common::User;
use rusqlite::Connection;
use uuid::Uuid;

use super::{Crud, CrudError};

const SQL_CREATE_USER_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                fullname TEXT
            );
        ";
const SQL_INSERT_USER: &str = "INSERT INTO users (id, fullname) VALUES (?1, ?2);";
// const SQL_SELECT_ALL_USERS: &str = "SELECT id, fullname FROM users";
const SQL_SELECT_USERS_BY_ID: &str = "SELECT id, fullname FROM users WHERE id = ?1";
const SQL_UPDATE_USER_BY_ID: &str = "UPDATE users SET fullname = ?1 WHERE id = ?2";
const SQL_DELETE_USER_BY_ID: &str = "DELETE FROM users WHERE id = ?1";

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
        connection.execute(SQL_CREATE_USER_TABLE, ()).unwrap();
        connection.execute(SQL_INSERT_USER, [&user.id.to_string(), &user.fullname]).unwrap();
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<User>, CrudError> {
        log::debug!("Reading all Users from '{}'", &self.filename);
        if let Ok(connection) = Connection::open(&self.filename) {
            let query = "SELECT id, fullname FROM users";
            if let Ok(mut stmt) = connection.prepare(query) {
                let user_iter = stmt.query_map([], |row| {
                    let uuid_str: String = row.get(0)?;
                    Ok(User {
                        id: Uuid::parse_str(&uuid_str).expect("Failed to parse UUID"),
                        fullname: row.get(1)?,
                    })
                })?;
                let users = user_iter.map(|r| r.expect("Failed to SELECT User")).collect();
                Ok(users)
            } else {
                log::warn!("Error encountered preparing statement");
                Ok(Vec::new())
            }
        } else {
            log::warn!("Error opening SQLite connection");
            Ok(Vec::new())
        }
    }

    fn update(&mut self, item: &User) -> super::Result<()> {
        log::debug!("Updating User id='{}' from '{}'", &item.id, &self.filename);
        let conn = Connection::open(&self.filename)?;
        let mut stmt = conn.prepare(SQL_UPDATE_USER_BY_ID)?;
        stmt.execute([&item.fullname, &item.id.to_string()])?;
        Ok(())
    }

    fn delete(&mut self, item: &User) -> super::Result<()> {
        log::debug!("Deleting User id='{}' from '{}'", &item.id, &self.filename);
        let conn = Connection::open(&self.filename)?;
        let mut stmt = conn.prepare(SQL_DELETE_USER_BY_ID)?;
        stmt.execute([&item.id.to_string()])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use uuid::uuid;

    use super::*;

    #[test]
    fn create_creates_file_when_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.sqlite");
        let mut store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        store
            .create(&User::new("Test User"))
            .expect("Failed to create new User");
        assert!(sqlite_path.exists());
    }

    #[test]
    fn read_all_returns_none_when_file_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.sqlite");
        let store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        let users = store.read_all().expect("Failed to read Users");
        assert_eq!(users.len(), 0);
    }

    #[test]
    fn read_all_returns_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.sqlite");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test User")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        let users = store.read_all().expect("Failed to read Users");
        assert_eq!(users.len(), 1);
        assert_eq!(users.get(0).unwrap().fullname, "Test User");
    }

    #[test]
    fn update_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test User")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        let mut user_updated = User::new("Modified User");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update User");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let actual_row: String = stmt.query_row([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()], |row| row.get(1)).expect(&format!("Failed to query {}", SQL_SELECT_USERS_BY_ID));
        assert_eq!(actual_row, "Modified User");
    }

    #[test]
    fn update_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test User 1")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c9").to_string(), "Test User 2")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        let mut user_updated = User::new("Modified User 1");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update User");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let actual_user_1: String = stmt.query_row([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()], |row| row.get(1)).expect(&format!("Failed to query {}", SQL_SELECT_USERS_BY_ID));
        assert_eq!(actual_user_1, "Modified User 1");
    }

    #[test]
    fn delete_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test User 1")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        let mut user = User::new("Test User");
        user.id = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.delete(&user).expect("Failed to delete User");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let mut result = stmt.query([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()]).expect(&format!("Failed to execute {}", SQL_SELECT_USERS_BY_ID));
        assert!(result.next().expect("").is_none())
    }

    #[test]
    fn delete_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("users.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test User 1")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c9").to_string(), "Test User 2")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteUserStore::new(sqlite_path.display().to_string().as_str());
        let mut user = User::new("Test User");
        user.id = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.delete(&user).expect("Failed to delete User");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let mut result = stmt.query([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()]).expect(&format!("Failed to execute {}", SQL_SELECT_USERS_BY_ID));
        assert!(result.next().expect("").is_none())
    }
}
