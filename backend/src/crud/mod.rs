#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "sqlite")]
pub mod sqlite;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrudError {
    UnknownError,
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
    fn create(&mut self, item: &T) -> Result<(), CrudError>;
    fn read_all(&self) -> Result<Vec<T>, CrudError>;
}
