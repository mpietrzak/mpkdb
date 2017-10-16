
//! DB Abstraction.

use std;

#[derive(Debug)]
pub struct Error {
    pub desc: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.desc
    }
}

/// What our app can handle as the password db.
pub trait PasswordDatabase: std::fmt::Debug {}
