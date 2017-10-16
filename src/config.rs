use std;
use std::fs::File;
use std::io::Read;

use toml;

use errors;
use model::Config;

/// Return default config file path on this system.
pub fn default_config_file_path() -> String {
    String::from("mpkdb.toml")
}

pub fn load_config() -> Result<Config, errors::Error> {
    let path = default_config_file_path();
    let conf: Config = match File::open(&path) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let conf: Config = toml::from_str(&contents)?;
            conf
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!(
                "Config file \"{}\" not found, returning default config",
                path
            );
            Default::default()
        }
        Err(e) => return Err(errors::Error::from(e)),
    };
    Ok(conf)
}
