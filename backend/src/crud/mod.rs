use std::error::Error;

use common::User;

pub mod csv;

pub trait UserStorage: Send + Clone {
    fn create(&mut self, user: &User) -> Result<(), Box<dyn Error>>;
    fn read_all(&self) -> Result<Vec<User>, Box<dyn Error>>;
}
