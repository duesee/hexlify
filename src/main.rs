use std::{
    fs::File,
    io::{self, Read, stdin, stdout},
    path::Path,
};

use argh::FromArgs;

use hexlify::{decode, encode};

#[derive(FromArgs)]
/// (un)hexlify
///
/// Perform bytes-to-hexstring conversion and vice-versa as implemented
/// in, e.g., Python's binascii.{un,}hexlify. Read from stdin if <file>
/// is not specified. Whitespace is ignored during decoding.
struct Args {
    /// decode stream
    #[argh(switch, short = 'd')]
    decode: bool,

    /// ignore non-hex values
    #[argh(switch, short = 'i')]
    ignore_garbage: bool,

    /// show version
    #[argh(switch)]
    version: bool,

    /// file
    #[argh(positional)]
    file: Option<String>,
}

// Get version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run(file: Option<String>, flag_decode: bool, flag_ignore_garbage: bool) -> io::Result<()> {
    // Choose input (file or stdin)
    let mut src: Box<dyn Read> = match file {
        Some(path) => Box::new(File::open(&path)?),
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
    let mut args: Args = argh::from_env();

    if args.version {
        println!("{VERSION}");
        return;
    }

    // Decode when called as `unhexlify` (e.g., via symlink).
    if let Some(name) = std::env::args().next() {
        if let Some(name) = Path::new(&name).file_name() {
            if name == "unhexlify" {
                args.decode = true;
            }
        } else {
            panic!("unknown argument environment")
        }
    } else {
        panic!("unknown argument environment")
    }

    run(args.file, args.decode, args.ignore_garbage).unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    })
}
