
//! DB Abstraction.

use std;

pub trait Error: std::error::Error {
}

/// What our app can handle as the password db.
pub trait PasswordDatabase {
    fn open(path: &str) -> Error;
    fn close(&self);
}
