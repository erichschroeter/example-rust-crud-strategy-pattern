#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub type Result<T> = std::result::Result<T, CrudError>;

// #[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Debug)]
pub enum CrudError {
    UnknownError,
    NotFound,
    IO(std::io::Error),
    #[cfg(feature = "sqlite")]
    SqliteError(rusqlite::Error),
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for CrudError {
    fn from(err: rusqlite::Error) -> Self {
        CrudError::SqliteError(err)
    }
}

impl From<std::io::Error> for CrudError {
    fn from(err: std::io::Error) -> Self {
        CrudError::IO(err)
    }
}

impl std::error::Error for CrudError {}

impl Default for CrudError {
    fn default() -> Self {
        CrudError::UnknownError
    }
}

impl std::fmt::Display for CrudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Generic CRUD error")
    }
}

pub trait Crud<T> {
    fn create(&mut self, item: &T) -> Result<()>;
    fn read_all(&self) -> Result<Vec<T>>;
    fn update(&mut self, item: &T) -> Result<()>;
    fn delete(&mut self, item: &T) -> Result<()>;
}
