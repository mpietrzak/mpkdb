
//! Parse KDB files using Nom.

use std;

use nom;

const PWM_DBSIG_1: u32 = 0x9AA2D903;
const PWM_DBSIG_2: u32 = 0xB54BFB65;
// const PWM_DBSIG_1_KDBX_P: u32 = 0x9AA2D903;
// const PWM_DBSIG_1_KDBX_R: u32 = 0x9AA2D903;
// const PWM_DBVER_DW: u32 = 0x00030004;
const PWM_FLAG_RIJNDAEL: u32 = 2;
const PWM_FLAG_TWOFISH: u32 = 8;

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

#[derive(Debug)]
enum EncryptionAlgorithm {
    AES,
    TwoFish
}

/// KDB password file header structure.
/// Based on PwStructs.h.
#[derive(Debug)]
pub struct KdbHeader {
    signature_1: u32,
    signature_2: u32,
    flags: u32,
    version: u32,
    master_seed: [u8; 16],
    enc_iv: [u8; 16],
    group_count: u32,
    entry_count: u32,
    contents_hash: [u8; 32],
    master_seed_2: [u8; 32],
    key_enc_rounds: u32,
}

#[derive(Debug)]
pub struct KdbFile {
    header: KdbHeader,
}

/// Silly helper.
named!(
    take_u8_16<[u8; 16]>,
    map!(take!(16), |s| {
        let mut a: [u8; 16] = [0; 16];
        a.copy_from_slice(s);
        a
    })
);

/// Twice as silly helper.
named!(
    take_u8_32<[u8; 32]>,
    map!(take!(32), |s| {
        let mut a: [u8; 32] = [0; 32];
        a.copy_from_slice(s);
        a
    })
);


named!(
    kdb_header<KdbHeader>,
    do_parse!(
        signature_1: call!(nom::le_u32) >>
        signature_2: call!(nom::le_u32) >>
        flags: call!(nom::le_u32) >>
        version: call!(nom::le_u32) >>
        master_seed: take_u8_16 >>
        enc_iv: take_u8_16 >>
        group_count: call!(nom::le_u32) >>
        entry_count: call!(nom::le_u32) >>
        contents_hash: take_u8_32 >>
        master_seed_2: take_u8_32 >>
        key_enc_rounds: call!(nom::le_u32) >>
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
                master_seed: master_seed,
                enc_iv: enc_iv,
                group_count: group_count,
                entry_count: entry_count,
                contents_hash: contents_hash,
                master_seed_2: master_seed_2,
                key_enc_rounds: key_enc_rounds
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
            // kdb has three versions, and we support newest one for now...
            // 0x00020000 -> v2
            // 0x00020001 -> v2
            // 0x00030004 -> current
            // 0x00010002 -> v1
            // ...
            let file_ver_major = f.header.version >> 16;
            let file_ver_minor = f.header.version & 0x0000FFFF;
            debug!(
                "Major version: {}, minor version: {}",
                file_ver_major,
                file_ver_minor
            );
            if file_ver_major < 3 {
                return Err(Error {
                    desc: format!(
                        "Unsupported DB version {}.{}",
                        file_ver_major,
                        file_ver_minor
                    ),
                });
            }
            debug!(
                "Group count: {}, entry count: {}",
                f.header.group_count,
                f.header.entry_count
            );
            let enc_algo = {
                if f.header.flags & PWM_FLAG_RIJNDAEL != 0 {
                    EncryptionAlgorithm::AES
                } else if f.header.flags & PWM_FLAG_TWOFISH != 0 {
                    EncryptionAlgorithm::TwoFish
                } else {
                    return Err(Error {
                        desc: format!("Unknown encryption algorithm, flags: {:x}", f.header.flags),
                    });
                }
            };
            debug!("Encryption algorithm: {:?}", enc_algo);
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
