
//! Helpers and utils related to logging.

use std::env;

use env_logger::LogBuilder;
use log::LogLevelFilter;
use log::LogRecord;
use time;

/// Does not return error, instead just panics on error,
/// since logging is pretty essential.
pub fn env_logger_init() {
    let format = |record: &LogRecord| {
        let t = time::now();
        let ts = time::strftime("%Y-%m-%d %H:%M:%S", &t).expect("Failed to format time");
        format!(
            "{} {} {} {}",
            ts,
            record.level(),
            record.location().module_path(),
            record.args()
        )
    };
    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Debug);
    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }
    builder.init().expect("Failed to initialize env_logger");
}
