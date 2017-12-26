extern crate docopt;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

use docopt::Docopt;
use std::fs::File;
use std::io::{Read, Write, stdin, stdout, stderr};

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
        }
    }
}

use errors::*;

const USAGE: &str = "
hexlify

Perform bytes-to-hexstring conversion and vice-versa as implemented
in Python's binascii.{un,}hexlify. Read from stdin if <file> is \"-\"
or not specified. Whitespace is ignored during decoding.

Usage: hexlify [options] [<file>]

Options:
  -d --decode          Decode stream.
  -i --ignore-garbage  Ignore non-hex values.
  -h --help            Show this help screen.
";

const ERR_NOT_IN_HEX: &str = "\
Character does not match hex-alphabet. Only 0-9, a-f and A-F are allowed.
Make sure not to confuse decoding with encoding or use -i to ignore non-hex characters.\
";

const ERR_ODD_CHARS: &str = "\
Input had odd number of characters, please be cautious.\
";

// Get version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Args {
    arg_file: Option<String>,
    flag_decode: bool,
    flag_ignore_garbage: bool,
}

fn to_hex_value(c: u8) -> Result<u8> {
    match c as char {
        '0'...'9' => Ok(c - '0' as u8),
        'a'...'f' => Ok(c - 'a' as u8 + 10),
        'A'...'F' => Ok(c - 'A' as u8 + 10),
        _ => Err(ERR_NOT_IN_HEX.into()),
    }
}

fn pair_to_hex(pair: &[u8; 2]) -> Result<u8> {
    let l = to_hex_value(pair[0])?;
    let r = to_hex_value(pair[1])?;
    Ok(l * 16 + r)
}

fn decode<R: Read, W: Write>(src: &mut R, dst: &mut W, ignore_garbage: bool) -> Result<()> {
    let mut side = 0;
    let mut pair = [0 as u8; 2];

    for byte in src.bytes() {
        let byte = byte?;

        if (byte as char).is_whitespace() {
            continue;
        }

        if to_hex_value(byte).is_err() && ignore_garbage {
            continue;
        }

        pair[side] = byte;

        if side == 0 {
            side = 1;
        } else {
            dst.write(&[pair_to_hex(&pair)?])?;
            side = 0;
        }
    }

    if side == 1 {
        return Err(ERR_ODD_CHARS.into());
    }
    
    Ok(())
}

fn encode<R: Read, W: Write>(src: &mut R, dst: &mut W) -> Result<()> {
    for byte in src.bytes() {
        write!(dst, "{:02X}", byte?)?;
    }
    
    Ok(())
}

fn run(file: Option<String>, flag_decode: bool, flag_ignore_garbage: bool) -> Result<()> {
    // Choose input (file or stdin)
    let mut src: Box<Read> = match file {
        Some(path) => {
            if path != "-" {
                Box::new(File::open(&path).chain_err(|| format!("can't open \"{}\"", path))?)
            } else {
                Box::new(stdin())
            }
        }
        None => Box::new(stdin()),
    };

    // Choose output (stdout)
    let mut dst = stdout();

    if flag_decode {
        decode(&mut src, &mut dst, flag_ignore_garbage)?;
    } else {
        encode(&mut src, &mut dst)?;
    }

    Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.version(Some(VERSION.into())).deserialize())
        .unwrap_or_else(|e| e.exit());

    if let Err(ref e) = run(args.arg_file, args.flag_decode, args.flag_ignore_garbage) {
        writeln!(stderr(), "error: {}", e).unwrap();

        for e in e.iter().skip(1) {
            writeln!(stderr(), "caused by: {}", e).unwrap();
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr(), "backtrace: {:?}", backtrace).unwrap();
        }

        ::std::process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_hex_value() {
        assert_eq!(to_hex_value('0' as u8).unwrap(), 0);
        assert_eq!(to_hex_value('9' as u8).unwrap(), 9);
        assert_eq!(to_hex_value('a' as u8).unwrap(), 10);
        assert_eq!(to_hex_value('f' as u8).unwrap(), 15);
        assert_eq!(to_hex_value('A' as u8).unwrap(), 10);
        assert_eq!(to_hex_value('F' as u8).unwrap(), 15);
    }

    #[test]
    fn test_pair_to_hex() {
        assert_eq!(pair_to_hex(&['0' as u8, '0' as u8]).unwrap(), 0x00);
        assert_eq!(pair_to_hex(&['9' as u8, '9' as u8]).unwrap(), 0x99);
        assert_eq!(pair_to_hex(&['a' as u8, 'a' as u8]).unwrap(), 0xAA);
        assert_eq!(pair_to_hex(&['A' as u8, 'A' as u8]).unwrap(), 0xAA);
        assert_eq!(pair_to_hex(&['f' as u8, 'f' as u8]).unwrap(), 0xFF);
        assert_eq!(pair_to_hex(&['F' as u8, 'F' as u8]).unwrap(), 0xFF);

        assert!(pair_to_hex(&['0' as u8, '/' as u8]).is_err());
        assert!(pair_to_hex(&['0' as u8, ':' as u8]).is_err());

        assert!(pair_to_hex(&['0' as u8, '`' as u8]).is_err());
        assert!(pair_to_hex(&['0' as u8, 'g' as u8]).is_err());

        assert!(pair_to_hex(&['0' as u8, '@' as u8]).is_err());
        assert!(pair_to_hex(&['0' as u8, 'G' as u8]).is_err());
    }
    
    #[test]
    fn test_decode() {
        let samples = vec![
            ("AA", vec![0xAA]),
            ("AABB", vec![0xAA, 0xBB]),
            ("AABBCCD", vec![0xAA, 0xBB, 0xCC]),
            ("AABBCCDD", vec![0xAA, 0xBB, 0xCC, 0xDD]),
            ("00gf", vec![0x00]),
            ("00g  ff", vec![0x00, 0xff]),
            ("---f-..,f aa b", vec![0xff, 0xaa]),
            ("²¹²³fa„“", vec![0xfa]),
            ("¹¸^afg01", vec![0xaf, 0x01]),
            ("´#*-_ff:;:;:1234", vec![0xff, 0x12, 0x34]),
            ("", vec![]),
            ("a1¹²}ff™± ¡¿⅛°±£™⅞`ff¿¡⅛°±£™01`", vec![0xa1, 0xff, 0xff, 0x01]),
        ];
 
        for (mut sample, expected) in samples.into_iter().map(|(s, e)| (std::io::Cursor::new(s), e)) {
            let got = {
                let mut tmp = Vec::new();
                let _ = decode(&mut sample, &mut tmp, true);
                tmp
            };
            
            assert_eq!(expected, got);
        }
    }
    
    #[test]
    fn test_encode() {
        let samples = vec![
            (vec![], ""),
            (vec![0x00], "00"),
            (vec![0x00, 0x99, 0xAA, 0xFF], "0099AAFF"),
            (vec![0x00, 0x99, 0xAA, 0xFF, 0x00, 0x00], "0099AAFF0000"),
        ];
 
        for (mut sample, expected) in samples.into_iter().map(|(s, e)| (std::io::Cursor::new(s), e)) {
            let got = {
                let mut tmp = Vec::new();
                let _ = encode(&mut sample, &mut tmp);
                String::from_utf8(tmp).unwrap()
            };
            
            assert_eq!(expected, got);
        }
    }
}
