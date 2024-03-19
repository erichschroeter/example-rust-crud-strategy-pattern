use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub fullname: String,
}

impl User {
    pub fn new(fullname: &str) -> Self {
        User {
            fullname: fullname.to_string(),
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Full name: {}", self.fullname)
    }
}
