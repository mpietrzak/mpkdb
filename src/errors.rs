
//! Error helper stuff.

use std;
use std::fmt;

use toml;

/// My error struct.
#[derive(Debug)]
pub struct Error {
    desc: String,
}

/// My mini error chain helper, marker trait.
/// Things that implement error chain can be converted to my error type.
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

/// Generic impl of convert from things marked by my ErrorChain trait.
/// TODO: Investigate error chain crate (again).
/// TODO: Maybe do something with Error::cause (although for now I didn't really need it so... investigate benefits first).
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
