#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate csv;

use docopt::Docopt;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, stdin, stdout};
use std::ascii::AsciiExt;
use std::fmt;

const USAGE: &'static str = "
hexlify 0.0.1

Performs bytes-to-hex conversion as implemented in Python's binascii.{un,}hexlify. hexlify reads from stdin if <file> is not specified.

Usage: monitor [options] [<file>]

Options:
  -d --decode          Decode stream.
  -i --ignore-garbage  Ignore bad characters.
  -h --help            Show this help screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file:            Option<String>,
    flag_decode:         bool,
    flag_ignore_garbage: bool,
}

#[derive(Debug)]
struct CodingError {
	cause: String
}

impl CodingError {
	fn new(cause: &str) -> CodingError {
		CodingError {
			cause: cause.into()
		}
	}
}

impl fmt::Display for CodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Error for CodingError {
    fn description(&self) -> &str {
		&self.cause
	}

    fn cause(&self) -> Option<&Error> {
		None	
	}
}

fn to_hex_value(c: char) -> Option<u8> {
	if '0' <= c && c <= '9' { return Some(c as u8 - '0' as u8) }
	if 'a' <= c && c <= 'f' { return Some(c as u8 - 'a' as u8 + 10) }
	None
}

fn run(file: Option<String>, decode: bool, ignore_garbage: bool) -> Result<(), Box<Error>> {
    let mut src: Box<Read> = match file {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(stdin()),
    };

	if ignore_garbage {
		unimplemented!();
	}

	if decode {
		loop {
			let mut buffer = [0u8; 2];
			let pair = src.read(&mut buffer);
			let (l, r) = match pair {
				Ok(2) => (buffer[0], buffer[1]),
				Ok(1) => return Err(Box::new(CodingError::new("Input stream must have even number of characters."))),
				Ok(0) => return Ok(()),
				Ok(_) => unreachable!(),			
				Err(err) => return Err(Box::new(err)),
			};

			let l = (l as char).to_ascii_lowercase();
			let r = (r as char).to_ascii_lowercase();
		
			if let Some(l) = to_hex_value(l) {
				if let Some(r) = to_hex_value(r) {
					stdout().write_all(&[l * 16 + r])?;
					continue;
				}
			}

			return Err(Box::new(CodingError::new("Input stream must match hex alphabet, i.e. [a-zA-Z0-9].")));
		}
	} else {
		for byte in src.bytes() {
			match byte {
				Ok(byte) => print!("{:02X}", byte),
				Err(err) => return Err(Box::new(err)),
			}
		};
	}

	Ok(())
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    run(args.arg_file, args.flag_decode, args.flag_ignore_garbage)
		.unwrap_or_else(|e| writeln!(&mut std::io::stderr(), "{}", e).unwrap());
}
