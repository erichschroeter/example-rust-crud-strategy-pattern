use common::Account;
use rusqlite::Connection;
use uuid::Uuid;

use super::{Crud, CrudError};

const SQL_CREATE_USER_TABLE: &str = "
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                fullname TEXT
            );
        ";
const SQL_INSERT_USER: &str = "INSERT INTO accounts (id, fullname) VALUES (?1, ?2);";
// const SQL_SELECT_ALL_USERS: &str = "SELECT id, fullname FROM accounts";
const SQL_SELECT_USERS_BY_ID: &str = "SELECT id, fullname FROM accounts WHERE id = ?1";
const SQL_UPDATE_USER_BY_ID: &str = "UPDATE accounts SET fullname = ?1 WHERE id = ?2";
const SQL_DELETE_USER_BY_ID: &str = "DELETE FROM accounts WHERE id = ?1";

#[derive(Debug, Clone)]
pub struct SqliteAccountStore {
    filename: String,
}

impl SqliteAccountStore {
    pub fn new(filename: &str) -> Self {
        SqliteAccountStore {
            filename: filename.to_string(),
        }
    }
}

impl Crud<Account> for SqliteAccountStore {
    fn create(&mut self, account: &Account) -> Result<(), CrudError> {
        let connection = Connection::open(&self.filename).unwrap();
        connection.execute(SQL_CREATE_USER_TABLE, ()).unwrap();
        connection.execute(SQL_INSERT_USER, [&account.id.to_string(), &account.fullname]).unwrap();
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<Account>, CrudError> {
        log::debug!("Reading all Users from '{}'", &self.filename);
        if let Ok(connection) = Connection::open(&self.filename) {
            let query = "SELECT id, fullname FROM accounts";
            if let Ok(mut stmt) = connection.prepare(query) {
                let user_iter = stmt.query_map([], |row| {
                    let uuid_str: String = row.get(0)?;
                    Ok(Account {
                        id: Uuid::parse_str(&uuid_str).expect("Failed to parse UUID"),
                        fullname: row.get(1)?,
                    })
                })?;
                let accounts = user_iter.map(|r| r.expect("Failed to SELECT Account")).collect();
                Ok(accounts)
            } else {
                log::warn!("Error encountered preparing statement");
                Ok(Vec::new())
            }
        } else {
            log::warn!("Error opening SQLite connection");
            Ok(Vec::new())
        }
    }

    fn update(&mut self, item: &Account) -> super::Result<()> {
        log::debug!("Updating Account id='{}' from '{}'", &item.id, &self.filename);
        let conn = Connection::open(&self.filename)?;
        let mut stmt = conn.prepare(SQL_UPDATE_USER_BY_ID)?;
        stmt.execute([&item.fullname, &item.id.to_string()])?;
        Ok(())
    }

    fn delete(&mut self, item: &Account) -> super::Result<()> {
        log::debug!("Deleting Account id='{}' from '{}'", &item.id, &self.filename);
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
        let sqlite_path = dir.path().join("accounts.sqlite");
        let mut store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        store
            .create(&Account::new("Test Account"))
            .expect("Failed to create new Account");
        assert!(sqlite_path.exists());
    }

    #[test]
    fn read_all_returns_none_when_file_not_exist() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("accounts.sqlite");
        let store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        let accounts = store.read_all().expect("Failed to read Users");
        assert_eq!(accounts.len(), 0);
    }

    #[test]
    fn read_all_returns_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("accounts.sqlite");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test Account")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        let accounts = store.read_all().expect("Failed to read Users");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts.get(0).unwrap().fullname, "Test Account");
    }

    #[test]
    fn update_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("accounts.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test Account")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        let mut user_updated = Account::new("Modified Account");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update Account");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let actual_row: String = stmt.query_row([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()], |row| row.get(1)).expect(&format!("Failed to query {}", SQL_SELECT_USERS_BY_ID));
        assert_eq!(actual_row, "Modified Account");
    }

    #[test]
    fn update_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("accounts.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test Account 1")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c9").to_string(), "Test Account 2")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        let mut user_updated = Account::new("Modified Account 1");
        user_updated.id = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.update(&user_updated).expect("Failed to update Account");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let actual_user_1: String = stmt.query_row([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()], |row| row.get(1)).expect(&format!("Failed to query {}", SQL_SELECT_USERS_BY_ID));
        assert_eq!(actual_user_1, "Modified Account 1");
    }

    #[test]
    fn delete_one_of_one() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("accounts.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test Account 1")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        let mut account = Account::new("Test Account");
        account.id = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.delete(&account).expect("Failed to delete Account");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let mut result = stmt.query([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()]).expect(&format!("Failed to execute {}", SQL_SELECT_USERS_BY_ID));
        assert!(result.next().expect("").is_none())
    }

    #[test]
    fn delete_one_of_two() {
        let dir = tempdir().expect("Failed to create temp directory");
        let sqlite_path = dir.path().join("accounts.csv");
        let conn = Connection::open(&sqlite_path).expect(&format!("Failed to open {}", &sqlite_path.display()));
        conn.execute(SQL_CREATE_USER_TABLE, ()).expect(&format!("Failed to execute {}", SQL_CREATE_USER_TABLE));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string(), "Test Account 1")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        conn.execute(SQL_INSERT_USER, (uuid!("67e55044-10b1-426f-9247-bb680e5fe0c9").to_string(), "Test Account 2")).expect(&format!("Failed to execute {}", SQL_INSERT_USER));
        let mut store = SqliteAccountStore::new(sqlite_path.display().to_string().as_str());
        let mut account = Account::new("Test Account");
        account.id = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
        store.delete(&account).expect("Failed to delete Account");
        let mut stmt = conn.prepare(SQL_SELECT_USERS_BY_ID).expect(&format!("Failed to prepare {}", SQL_SELECT_USERS_BY_ID));
        let mut result = stmt.query([uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8").to_string()]).expect(&format!("Failed to execute {}", SQL_SELECT_USERS_BY_ID));
        assert!(result.next().expect("").is_none())
    }
}
