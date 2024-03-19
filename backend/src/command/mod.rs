pub mod serve;

use std::error::Error;

pub trait FixmeError: Error {}

impl FixmeError for std::io::Error {}

pub trait Command {
    fn execute(&self) -> Result<(), Box<dyn FixmeError>>;
}
