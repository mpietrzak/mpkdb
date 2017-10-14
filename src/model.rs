
//! Some global helper stucts.

#[derive(Deserialize)]
pub struct Config {
    pub last_file: Option<String>
}

impl Default for Config {
    fn default() -> Config {
        Config {
            last_file: None,
        }
    }
}
