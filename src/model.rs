
//! Some global helper stucts.

use std::sync::Arc;
use std::sync::RwLock;

use db::api::PasswordDatabase;

#[derive(Deserialize)]
pub struct Config {
    pub last_file: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { last_file: None }
    }
}

/// Main "global" app state.
pub struct State {
    pub db: Option<Arc<RwLock<PasswordDatabase>>>,
}
