
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
    fn get_entry_count(&self) -> u32 {
        let s = self.file.entries.len();
        if s > (u32::max_value() as usize) {
            // Shouldn't happen, we should not support files that big.
            panic!("Too many entries")
        }
        self.file.entries.len() as u32
    }
}

pub fn open(filename: &str, password: &str) -> Result<KdbDatabase, api::Error> {
    debug!("open: About to open \"{}\"...", filename);
    let file = match std::fs::File::open(filename) {
        Ok(file) => file,
        Err(e) => {
            return Err(api::Error {
                desc: format!("Error opening file: {}", e),
            })
        }
    };
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = Vec::new();
    if let Err(e) = reader.read_to_end(&mut buffer) {
        return Err(api::Error {
            desc: format!("Error reading file: {}", e),
        });
    }
    let kdb_file = match kdb::parser::parse_kdb_file(&buffer, password) {
        Ok(f) => f,
        Err(e) => {
            return Err(api::Error {
                desc: format!("Failed to parse file: {}", e),
            })
        }
    };
    Ok(KdbDatabase { file: kdb_file })
}
