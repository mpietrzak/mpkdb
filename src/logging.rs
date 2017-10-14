
//! Helpers and utils related to logging.

use env_logger;

/// Init logging.
/// Does not return error, instead it exists the app if logging was not initialized.
pub fn init_logger() -> () {
    env_logger::init().expect("Failed to init env_logger");
}

