#[macro_use]
extern crate serde_derive;
extern crate docopt;

use docopt::Docopt;
use std::{fmt, process};
use std::ascii::AsciiExt;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, stdin, stdout};

const USAGE: &'static str = "
hexlify 0.0.1

Perform bytes-to-hexstring conversion and vice-versa as implemented
in Python's binascii.{un,}hexlify. Read from stdin if <file> is \"-\"
or not specified. Whitespace is ignored during decoding.

Usage: monitor [options] [<file>]

Options:
  -d --decode          Decode stream.
  -i --ignore-garbage  Ignore non-hex values.
  -h --help            Show this help screen.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file:            Option<String>,
    flag_decode:         bool,
	flag_ignore_garbage: bool,
}

#[derive(Debug)]
struct CliError {
	cause: String
}

impl CliError {
	fn new(cause: &str) -> CliError {
		CliError { cause: cause.into() }
	}
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Error for CliError {
    fn description(&self) -> &str {
		&self.cause
	}

    fn cause(&self) -> Option<&Error> {
		None	
	}
}

fn to_hex_value(c: char) -> Result<u8, Box<Error>> {
	let c = c.to_ascii_lowercase();
	if '0' <= c && c <= '9' { return Ok(c as u8 - '0' as u8) }
	if 'a' <= c && c <= 'f' { return Ok(c as u8 - 'a' as u8 + 10) }
	Err(Box::new(CliError::new("Each character must match hex alphabet, i.e. a-z, A-Z or 0-9. Use -i to ignore non-hex characters.")))
}

fn pair_to_hex(pair: &[char; 2]) -> Result<u8, Box<Error>> {
	let l = to_hex_value(pair[0])?;
	let r = to_hex_value(pair[1])?;
	Ok(l*16 + r)
}

fn run(file: Option<String>, decode: bool, ignore_garbage: bool) -> Result<(), Box<Error>> {
	let src: Box<Read> = match file {
		Some(path) => {
			if path != "-" { 
				Box::new(File::open(path)?)
			} else {
				Box::new(stdin())
			}
		}
		None => Box::new(stdin()),
	};

	if decode {
		let mut side = 0;
		let mut pair = [0 as char; 2];

		for byte in src.bytes() {
			let byte = byte? as char;
			if byte.is_whitespace() { continue }
			if ignore_garbage && to_hex_value(byte).is_err() { continue }

			pair[side] = byte;

			if side == 0 {
				side = 1;
			} else {
				stdout().write(&[pair_to_hex(&pair)?])?;
				side = 0;
			}
		}
		
		if side == 1 {
			return Err(Box::new(CliError::new("Input stream must contain even number of characters. The last incomplete hex-pair was omitted.")))
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

    run(args.arg_file, args.flag_decode, args.flag_ignore_garbage)
		.unwrap_or_else(|e| {
			writeln!(&mut std::io::stderr(), "{}", e).unwrap();
			process::exit(1);
		});
}
