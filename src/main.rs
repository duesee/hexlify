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
hexlify 0.0.1

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
Character does not match hex-alphabet. Only a-z, A-Z and 0-9 are allowed.
Make sure not to confuse decoding with encoding or use -i to ignore non-hex characters.\
";

const ERR_ODD_CHARS: &str = "\
Input had odd number of characters, please be cautious.\
";

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

fn run(file: Option<String>, decode: bool, ignore_garbage: bool) -> Result<()> {
    let src: Box<Read> = match file {
        Some(path) => {
            if path != "-" {
                Box::new(File::open(&path).chain_err(
                    || format!("can't open \"{}\"", path),
                )?)
            } else {
                Box::new(stdin())
            }
        }
        None => Box::new(stdin()),
    };

    if decode {
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
                stdout().write(&[pair_to_hex(&pair)?])?;
                side = 0;
            }
        }

        if side == 1 {
            stdout().flush()?;
            return Err(ERR_ODD_CHARS.into());
        }
    } else {
        for byte in src.bytes() {
            print!("{:02X}", byte?);
        }
    }

    Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
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
    use super::{to_hex_value, pair_to_hex};

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
}
