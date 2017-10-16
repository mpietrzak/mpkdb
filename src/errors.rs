
//! Error helper stuff.

use std;
use std::fmt;

use toml;

/// My error struct.
#[derive(Debug)]
pub struct Error {
    desc: String,
}

pub trait ErrorChain: std::error::Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl Error {
    pub fn new(desc: &str) -> Error {
        Error {
            desc: String::from(desc),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl<E> std::convert::From<E> for Error
where
    E: std::marker::Sized + std::error::Error + ErrorChain,
{
    fn from(e: E) -> Error {
        Error::new(&format!("{}", e))
    }
}

impl ErrorChain for std::io::Error {}

impl ErrorChain for toml::de::Error {}
