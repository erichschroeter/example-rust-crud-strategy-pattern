use std::env;

pub mod index;
pub mod user;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
