
//! The KDB implementation of Database.

use std;
use std::io::Read;

use db::api;
use db::kdb;

#[derive(Debug)]
pub struct KdbDatabase {
    file: kdb::parser::KdbFile,
}

impl api::PasswordDatabase for KdbDatabase {
}

pub fn open(filename: &str) -> Result<KdbDatabase, api::Error> {
    debug!("open: About to open \"{}\"...", filename);
    let file = match std::fs::File::open(filename) {
        Ok(file) => file,
        Err(e) => return Err(api::Error{desc: format!("Error opening file: {}", e)})
    };
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = Vec::new();
    if let Err(e) = reader.read_to_end(&mut buffer) {
        return Err(api::Error{desc: format!("Error reading file: {}", e)});
    }
    let kdb_file = match kdb::parser::parse_kdb_file(&buffer) {
        Ok(f) => f,
        Err(e) => return Err(api::Error{desc: format!("Failed to parse file: {}", e)}),
    };
    Ok(KdbDatabase{
        file: kdb_file
    })
}

