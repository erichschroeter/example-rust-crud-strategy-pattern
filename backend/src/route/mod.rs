use std::env;

pub mod index;
pub mod user;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(feature = "csv")]
pub const BACKEND_STRATEGY: &str = "csv";
#[cfg(feature = "sqlite")]
pub const BACKEND_STRATEGY: &str = "sqlite";
