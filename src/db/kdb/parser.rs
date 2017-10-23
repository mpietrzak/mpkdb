
//! Parse KDB files.
//! TODO: Don't need Nom I think, KDB format is too small.

use std;
use std::io::BufRead;

use crypto;
use crypto::digest::Digest;
use nom;
use uuid;
use uuid::Uuid;

const PWM_DBSIG_1: u32 = 0x9AA2D903;
const PWM_DBSIG_2: u32 = 0xB54BFB65;
// const PWM_DBSIG_1_KDBX_P: u32 = 0x9AA2D903;
// const PWM_DBSIG_1_KDBX_R: u32 = 0x9AA2D903;
// const PWM_DBVER_DW: u32 = 0x00030004;
const PWM_FLAG_RIJNDAEL: u32 = 2;
const PWM_FLAG_TWOFISH: u32 = 8;

type DateTuple = (u16, u8, u8);
type TimeTuple = (u8, u8, u8);
type DateTimeTuple = (DateTuple, TimeTuple);

/// Header size in file.
const HEADER_SIZE: usize = 4 + 4 + 4 + 4 + 16 + 16 + 4 + 4 + 32 + 32 + 4;

#[derive(Debug)]
pub struct Error {
    pub desc: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "KDB Parse Error: {}", self.desc)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error {
            desc: format!("IO Error: {}", e),
        }
    }
}

impl std::convert::From<uuid::ParseError> for Error {
    fn from(e: uuid::ParseError) -> Error {
        Error {
            desc: format!("UUID Parse Error: {}", e),
        }
    }
}

/// KDB file encryption algo, currently only AES is supported.
#[derive(Debug)]
enum EncryptionAlgorithm {
    AES,
    TwoFish,
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
    /// Hash of the encrypted file contents.
    /// TODO: Actually check the hash.
    contents_hash: [u8; 32],
    /// Used to transform password into decryption key.
    master_seed_2: [u8; 32],
    key_enc_rounds: u32,
}

#[derive(Debug)]
pub struct KdbGroup {
    group_id: u32,
    group_name: String,
    /// Format: 2017-10-23 00:21:00
    created: String,
    modified: String,
    accessed: String,
    expires: String,
}

#[derive(Debug)]
pub struct KdbEntry {
    /// UUID
    entry_id: Uuid,
    group_id: u32,
    title: String,
    url: String,
    username: String,
    password: String,
    notes: String,
    created: String,
    modified: String,
    accessed: String,
    expires: String,
    // binary_description
    // binary_data
}

#[derive(Debug)]
pub struct KdbFile {
    header: KdbHeader,
    groups: Vec<KdbGroup>,
    pub entries: Vec<KdbEntry>,
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
        (
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
        ))
);

/// Turn byte slice into hex string, for debugging purposes.
fn format_bytes(bytes: &[u8]) -> String {
    let mut s: String = String::new();
    for b in bytes {
        s.push_str(&format!("{:0x}", b));
    }
    s
}

/// Hash password string to 32-byte master key used in KDB.
/// I think original KDB uses local windows ANSI encoding for this.
/// But most passwords do not contain "national" characters...
fn password_to_master_key(password: &str) -> [u8; 32] {
    let mut sha = crypto::sha2::Sha256::new();
    let sb: Vec<u8> = password.as_bytes().iter().map(|b| *b).collect();
    sha.input(&sb);
    let mut rb: [u8; 32] = [0; 32];
    sha.result(&mut rb);
    rb
}

/// Encrypt master key (provided by user) with master key seed (loaded from file).
/// Probably could be cleaned up a litte...
fn transform_master_key(
    master_key: [u8; 32],
    master_seed: [u8; 16],
    master_seed_2: [u8; 32],
    key_enc_rounds: u32,
) -> [u8; 32] {
    // Transform256
    let master_key_left: [u8; 16] = {
        let mut b: [u8; 16] = [0; 16];
        b.copy_from_slice(&master_key[..16]);
        b
    };
    let master_key_right: [u8; 16] = {
        let mut b: [u8; 16] = [0; 16];
        b.copy_from_slice(&master_key[16..]);
        b
    };
    let mut transformed_key_left: [u8; 16] = [0; 16];
    let mut transformed_key_right: [u8; 16] = [0; 16];
    // both "left" and "right" are independently crypted using aes
    // each of them key_enc_rounds-rounds
    transformed_key_left.copy_from_slice(&master_key_left);
    for _ in 0..key_enc_rounds {
        let mut outbuf: [u8; 16] = [0; 16];
        {
            let mut outwrite = crypto::buffer::RefWriteBuffer::new(&mut outbuf);
            let mut reader = crypto::buffer::RefReadBuffer::new(&transformed_key_left);
            let mut encryptor = crypto::aes::ecb_encryptor(
                crypto::aes::KeySize::KeySize256,
                &master_seed_2,
                crypto::blockmodes::NoPadding,
            );
            if let Err(e) = encryptor.encrypt(&mut reader, &mut outwrite, true) {
                panic!("Failed to encrypt master key, error: {:?}", e);
            }
        }
        transformed_key_left.copy_from_slice(&outbuf);
    }
    transformed_key_right.copy_from_slice(&master_key_right);
    for _ in 0..key_enc_rounds {
        let mut outbuf: [u8; 16] = [0; 16];
        {
            let mut outwrite = crypto::buffer::RefWriteBuffer::new(&mut outbuf);
            let mut reader = crypto::buffer::RefReadBuffer::new(&transformed_key_right);
            let mut encryptor = crypto::aes::ecb_encryptor(
                crypto::aes::KeySize::KeySize256,
                &master_seed_2,
                crypto::blockmodes::NoPadding,
            );
            if let Err(e) = encryptor.encrypt(&mut reader, &mut outwrite, true) {
                panic!("Failed to encrypt master key, error: {:?}", e);
            }
        }
        transformed_key_right.copy_from_slice(&outbuf);
    }
    let encrypted_master_key: [u8; 32] = {
        let mut t: [u8; 32] = [0; 32];
        for i in 0..16 {
            t[i] = transformed_key_left[i];
        }
        for i in 0..16 {
            t[i + 16] = transformed_key_right[i];
        }
        t
    };
    // Now sha256 hash it..
    let mut sha = crypto::sha2::Sha256::new();
    sha.input(&encrypted_master_key);
    let mut transformed_master_key: [u8; 32] = [0; 32];
    sha.result(&mut transformed_master_key);
    // And again hash it, but this time with master_seed.
    let mut sha = crypto::sha2::Sha256::new();
    sha.input(&master_seed);
    sha.input(&transformed_master_key);
    sha.result(&mut transformed_master_key);
    transformed_master_key
}

/// Read little endian 16 bit unsigned int.
fn read_u16<R: BufRead>(src: &mut R) -> Result<u16, Error> {
    let mut buf: [u8; 2] = [0; 2];
    src.read_exact(&mut buf)?;
    // Note: Plus is stronger than bit-shift.
    let r = buf[0] as u16 + ((buf[1] as u16) << 8);
    return Ok(r);
}

/// Little endian buf to u32.
fn bytes_to_u32(buf: [u8; 4]) -> u32 {
    let r = (buf[0] as u32) + ((buf[1] as u32) << 8) + ((buf[2] as u32) << 16) + ((buf[3] as u32) << 24);
    r
}

fn slice_to_u32(buf: &[u8]) -> Result<u32, Error> {
    if buf.len() != 4 {
        return Err(Error{desc: format!("Can't convert to u32, expected 4 bytes, got {}", buf.len())});
    }
    let mut arr: [u8; 4] = [0; 4];
    arr.copy_from_slice(buf);
    Ok(bytes_to_u32(arr))
}

/// Read little endian (least significant byte first).
fn read_u32<R: BufRead>(src: &mut R) -> Result<u32, Error> {
    let mut buf: [u8; 4] = [0; 4];
    src.read_exact(&mut buf)?;
    let r = bytes_to_u32(buf);
    return Ok(r);
}

/// Unpack "compressed time".
/// 5 bytes:
/// - year: 14 bit (8 + 6)
/// - month: 4 bit (2 + 2)
/// - day: 5 bit (5)
/// - hour: 5 bit (1 + 4)
/// - min: 6 bit (4 + 2)
/// - sec: 6 bit (6).
/// Consumes the bytes array, because it's too small to bother.
fn parse_datetime(b: [u8; 5]) -> DateTimeTuple {
    let year: u16 = ((b[0] as u16) << 6) + ((b[1] as u16) >> 2);
    let month: u8 = ((b[1] & 0b11) << 2) + (b[2] >> 6);
    let day: u8 = (b[2] >> 1) & 0b00011111;
    let hour: u8 = ((b[2] & 0b1) << 4) + (b[3] >> 4);
    let minute: u8 = ((b[3] & 0b00001111) << 2) + (b[4] >> 6);
    let sec: u8 = b[4] & 0b00111111;
    ((year, month, day), (hour, minute, sec))
}

fn parse_datetime_slice(b: &[u8]) -> Result<DateTimeTuple, Error> {
    if b.len() != 5 {
        return Err(Error{desc: format!("Failed to parse date-time: expected 5 bytes, got {}", b.len())});
    }
    // Again: copying but it's just too small to bother with refs.
    let mut byte_arr: [u8; 5] = [0; 5];
    byte_arr.copy_from_slice(b);
    Ok(parse_datetime(byte_arr))
}

/// Currently date/time is just a string, formatted this way: "2017-10-23 18:18:12".
fn format_date_time(dt: &DateTimeTuple) -> String {
    let d = dt.0;
    let t = dt.1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        d.1,
        d.1,
        d.2,
        t.0,
        t.1,
        t.2)
}

/// Read group from stream.
fn read_group<R: BufRead>(src: &mut R) -> Result<KdbGroup, Error> {
    let mut group_id = 0;
    let mut group_name = String::new();
    let mut created = String::new();
    let mut modified = String::new();
    let mut accessed = String::new();
    let mut expires = String::new();
    loop {
        let field_type: u16 = read_u16(src)?;
        let field_size: u32 = read_u32(src)?;
        let mut field_data: Vec<u8> = vec![0; field_size as usize];
        src.read_exact(&mut field_data)?;
        match field_type {
            // End of record
            0xFFFF => {
                break;
            }
            // Ext data
            0 => {
                // TODO: handle ext data
            }
            // Group ID
            0x0001 => {
                if field_size != 4 {
                    return Err(Error {
                        desc: format!(
                            "Invalid field size for Group ID: {}, expected 4 bytes",
                            field_size
                        ),
                    });
                }
                let mut bytes: [u8; 4] = [0; 4];
                for i in 0..4 {
                    bytes[i] = field_data[i];
                }
                group_id = bytes_to_u32(bytes);
            }
            // Group Name
            0x0002 => {
                group_name = String::from(String::from_utf8_lossy(&field_data));
            }
            // Creation Time
            0x0003 => {
                created = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x0004 => {
                modified = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x0005 => {
                accessed = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x0006 => {
                expires = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x0007 => {
                // TODO: icon
            }
            0x0008 => {
                // TODO: level
            }
            0x0009 => {
                // TODO: flags
            }
            _ => {
                return Err(Error {
                    desc: format!("Unknown field type: {} ({:0x})", field_type, field_type),
                });
            }
        }
    }
    Ok(KdbGroup {
        group_id: group_id,
        group_name: group_name,
        created: created,
        modified: modified,
        accessed: accessed,
        expires: expires,
    })
}

/// Read entry from stream.
fn read_entry<R: BufRead>(src: &mut R) -> Result<KdbEntry, Error> {
    let mut entry_id: Uuid = Uuid::nil();
    let mut group_id: u32 = 0;
    let mut title: String = String::new();
    let mut url: String = String::new();
    let mut username: String = String::new();
    let mut password: String = String::new();
    let mut notes: String = String::new();
    let mut created: String = String::new();
    let mut modified: String = String::new();
    let mut accessed: String = String::new();
    let mut expires: String = String::new();
    loop {
        let field_type: u16 = read_u16(src)?;
        let field_size: u32 = read_u32(src)?;
        let mut field_data: Vec<u8> = vec![0; field_size as usize];
        src.read_exact(&mut field_data)?;
        match field_type {
            0xFFFF => break,
            0x0000 => {
                // TODO: Handle ext data.
            }
            0x0001 => {
                // Entry ID
                entry_id = Uuid::from_bytes(&field_data)?;
            }
            0x0002 => {
                // Group ID
                group_id = slice_to_u32(&field_data)?;
            }
            0x0003 => {
                // TODO: Icon
            }
            0x0004 => {
                title = String::from(String::from_utf8_lossy(&field_data[0..field_data.len()-1]));
            }
            0x0005 => {
                url = String::from(String::from_utf8_lossy(&field_data[0..field_data.len()-1]));
            }
            0x0006 => {
                username = String::from(String::from_utf8_lossy(&field_data[0..field_data.len()-1]));
            }
            0x0007 => {
                password = String::from(String::from_utf8_lossy(&field_data[0..field_data.len()-1]));
            }
            0x0008 => {
                notes = String::from(String::from_utf8_lossy(&field_data[0..field_data.len()-1]));
            }
            0x0009 => {
                created = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x000a => {
                modified = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x000b => {
                accessed = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x000c => {
                expires = format_date_time(&parse_datetime_slice(&field_data)?);
            }
            0x000d => {
                // TODO: Binary Description
            }
            0x000e => {
                // TODO: Binary Data
            }
            _ => {
                return Err(Error {
                    desc: format!("Unknown field type: {} ({:0x})", field_type, field_type),
                });
            }
        }
    }
    Ok(KdbEntry {
        entry_id: entry_id,
        group_id: group_id,
        title: title,
        url: url,
        username: username,
        password: password,
        created: created,
        modified: modified,
        accessed: accessed,
        expires: expires,
        notes: notes,
    })
}

/// Parse KDB file.
/// TODO: Split into smaller functions.
pub fn parse_kdb_file(bytes: &[u8], password: &str) -> Result<KdbFile, Error> {
    let r = kdb_header(bytes);
    match r {
        nom::IResult::Done(_, header) => {
            // first check the "signature"
            if header.signature_1 != PWM_DBSIG_1 || header.signature_2 != PWM_DBSIG_2 {
                return Err(Error {
                    desc: format!(
                        "Invalid file signature: {:x} {:x} (expected {:x} {:x})",
                        header.signature_1,
                        header.signature_2,
                        PWM_DBSIG_1,
                        PWM_DBSIG_2
                    ),
                });
            }
            // KDB has three versions, and we support newest one for now...
            // 0x00020000 -> v2
            // 0x00020001 -> v2
            // 0x00030004 -> current
            // 0x00010002 -> v1
            // ...
            let file_ver_major = header.version >> 16;
            let file_ver_minor = header.version & 0x0000FFFF;
            if file_ver_major < 3 {
                return Err(Error {
                    desc: format!(
                        "Unsupported DB version {}.{}",
                        file_ver_major,
                        file_ver_minor
                    ),
                });
            }
            let _enc_algo = {
                if header.flags & PWM_FLAG_RIJNDAEL != 0 {
                    EncryptionAlgorithm::AES
                } else if header.flags & PWM_FLAG_TWOFISH != 0 {
                    EncryptionAlgorithm::TwoFish
                } else {
                    return Err(Error {
                        desc: format!("Unknown encryption algorithm, flags: {:x}", header.flags),
                    });
                }
            };
            let master_key = password_to_master_key(password);
            let transformed_master_key = transform_master_key(
                master_key,
                header.master_seed,
                header.master_seed_2,
                header.key_enc_rounds,
            );
            let mut decryptor = crypto::aes::cbc_decryptor(
                crypto::aes::KeySize::KeySize256,
                &transformed_master_key,
                &header.enc_iv,
                crypto::blockmodes::NoPadding,
            );
            let out: Vec<u8> = {
                let mut mout: Vec<u8> = Vec::new();
                mout.resize(bytes.len() - HEADER_SIZE, 0);
                {
                    let mut decryptor_input = crypto::buffer::RefReadBuffer::new(&bytes[HEADER_SIZE..]);
                    let mut decryptor_output = crypto::buffer::RefWriteBuffer::new(&mut mout);
                    if let Err(e) =  decryptor.decrypt(&mut decryptor_input, &mut decryptor_output, true) {
                        return Err(Error {
                            desc: format!("Decrypt error: {:?}", e),
                        });
                    }
                }
                mout
            };
            // TODO: Check hash.
            let (groups, entries) = {
                let mut curs = std::io::Cursor::new(&out);
                let mut groups = Vec::new();
                for i in 0..header.group_count {
                    debug!("Reading group {}", i);
                    let group = read_group(&mut curs)?;
                    debug!("Group {}: {:?}", i, group);
                    groups.push(group);
                }
                let entries = {
                    let mut entries = Vec::new();
                    for i in 0..header.entry_count {
                        debug!("Reading entry {}", i);
                        entries.push(read_entry(&mut curs)?);
                    }
                    entries
                };
                (groups, entries)
            };
            let f = KdbFile {
                header: header,
                entries: entries,
                groups: groups,
            };
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
