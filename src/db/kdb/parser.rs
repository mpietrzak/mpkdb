
//! Parse KDB files using Nom.

use std;

use nom;

const PWM_DBSIG_1: u32 = 0x9AA2D903;
const PWM_DBSIG_2: u32 = 0xB54BFB65;
const PWM_DBSIG_1_KDBX_P: u32 = 0x9AA2D903;
const PWM_DBSIG_1_KDBX_R: u32 = 0x9AA2D903;
const PWM_DBVER_DW: u32 = 0x00030004;

/*
55:#define PWM_DBSIG_1_KDBX_P 0x9AA2D903
56:#define PWM_DBSIG_2_KDBX_P 0xB54BFB66
57:#define PWM_DBSIG_1_KDBX_R 0x9AA2D903
58:#define PWM_DBSIG_2_KDBX_R 0xB54BFB67
*/

#[derive(Debug)]
pub struct Error {
    pub desc: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.desc
    }
}

/// KDB password file header structure.
/// Based on PwStructs.h.
#[derive(Debug)]
pub struct KdbHeader {
    signature_1: u32,
    signature_2: u32,
    flags: u32,
    version: u32,
}

#[derive(Debug)]
pub struct KdbFile {
    header: KdbHeader,
}

named!(
    kdb_header<KdbHeader>,
    do_parse!(
        signature_1: call!(nom::le_u32) >>
        signature_2: call!(nom::le_u32) >>
        flags: call!(nom::le_u32) >>
        version: call!(nom::le_u32) >>
        ({
            debug!("signature_1: {:x}, signature_2: {:x}, flags: {:x}, version: {:x}",
                   signature_1,
                   signature_2,
                   flags,
                   version);
            KdbHeader {
                signature_1: signature_1,
                signature_2: signature_2,
                flags: flags,
                version: version,
            }
        }))
);

named!(
    kdb_file<KdbFile>,
    do_parse!(header: kdb_header >> (KdbFile { header: header }))
);


/// Parse Kdb file.
pub fn parse_kdb_file(bytes: &[u8]) -> Result<KdbFile, Error> {
    let r = kdb_file(bytes);
    match r {
        nom::IResult::Done(_, f) => {
            // first check the "signature"
            if f.header.signature_1 != PWM_DBSIG_1 || f.header.signature_2 != PWM_DBSIG_2 {
                return Err(Error {
                    desc: format!(
                        "Invalid file signature: {:x} {:x} (expected {:x} {:x})",
                        f.header.signature_1,
                        f.header.signature_2,
                        PWM_DBSIG_1,
                        PWM_DBSIG_2
                    ),
                });
            }
            Ok(f)
        }
        nom::IResult::Error(e) => {
            error!("Failed to parse: {}", e);
            Err(Error {
                desc: format!("Parse error: {}", e),
            })
        }
        nom::IResult::Incomplete(i) => Err(Error {
            desc: format!("Parse error - incomplete input: {:?}", i),
        }),
    }
}
