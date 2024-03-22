use core::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct User {
    pub id: Uuid,
    pub fullname: String,
}

impl User {
    pub fn new(fullname: &str) -> Self {
        User {
            id: Uuid::new_v4(),
            fullname: fullname.to_string(),
        }
    }

    pub fn to_csv(&self) -> String {
        format!("{},{}", self.id.to_string(), self.fullname)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Full name: {}", self.fullname)
    }
}
