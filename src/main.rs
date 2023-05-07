extern crate docopt;
#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use std::fs::File;
use std::io;
use std::io::{stdin, stdout, Read};
use std::path::Path;

use hexlify::{decode, encode};

const USAGE: &str = "
hexlify

Perform bytes-to-hexstring conversion and vice-versa as implemented
in Python's binascii.{un,}hexlify. Read from stdin if <file> is \"-\"
or not specified. Whitespace is ignored during decoding.

Usage:
  hexlify [options] [<file>]
  hexlify (-h | --help)
  hexlify --version

Options:
  -d --decode          Decode stream.
  -i --ignore-garbage  Ignore non-hex values.
  -h --help            Show this screen.
  --version            Show version.
";

// Get version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Args {
    arg_file: Option<String>,
    flag_decode: bool,
    flag_ignore_garbage: bool,
}

fn run(file: Option<String>, flag_decode: bool, flag_ignore_garbage: bool) -> io::Result<()> {
    // Choose input (file or stdin)
    let mut src: Box<Read> = match file {
        Some(path) => {
            if path != "-" {
                Box::new(File::open(&path)?)
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
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.version(Some(VERSION.into())).deserialize())
        .unwrap_or_else(|e| e.exit());

    // TODO: provide symlinks in package, e.g. via AUR?
    if let Some(name) = std::env::args().next() {
        if let Some(name) = Path::new(&name).file_name() {
            if name == "unhexlify" {
                args.flag_decode = true;
            }
        } else {
            panic!("unknown argument environment")
        }
    } else {
        panic!("unknown argument environment")
    }

    run(args.arg_file, args.flag_decode, args.flag_ignore_garbage).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    })
}
