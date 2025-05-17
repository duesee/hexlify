use std::io::{self, Read, Write};

const ERR_NOT_IN_HEX: &str = "\
Character does not match hex-alphabet. Only 0-9, a-f and A-F are allowed.    
Make sure not to confuse decoding with encoding or use -i to ignore non-hex characters.\
";

const ERR_ODD_CHARS: &str = "\
Input had odd number of characters, please be cautious.\
";

fn to_hex_value(c: u8) -> Option<u8> {
    match c as char {
        '0'..='9' => Some(c - '0' as u8),
        'a'..='f' => Some(c - 'a' as u8 + 10),
        'A'..='F' => Some(c - 'A' as u8 + 10),
        _ => None,
    }
}

fn pair_to_hex(pair: &[u8; 2]) -> Option<u8> {
    if let (Some(l), Some(r)) = (to_hex_value(pair[0]), to_hex_value(pair[1])) {
        Some(l * 16 + r)
    } else {
        None
    }
}

pub fn decode<R: Read, W: Write>(src: &mut R, dst: &mut W, ignore_garbage: bool) -> io::Result<()> {
    let mut side = 0;
    let mut pair = [0 as u8; 2];

    for byte in src.bytes() {
        let byte = byte?;

        if (byte as char).is_whitespace() {
            continue;
        }

        if to_hex_value(byte).is_none() {
            if ignore_garbage {
                continue;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    ERR_NOT_IN_HEX.to_owned(),
                ));
            }
        }

        pair[side] = byte;

        if side == 0 {
            side = 1;
        } else {
            dst.write_all(&[pair_to_hex(&pair).unwrap()])?;
            side = 0;
        }
    }

    if side == 1 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            ERR_ODD_CHARS.to_owned(),
        ));
    }

    Ok(())
}

pub fn encode<R: Read, W: Write>(src: &mut R, dst: &mut W) -> io::Result<()> {
    for byte in src.bytes() {
        write!(dst, "{:02X}", byte?)?;
    }

    Ok(())
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

        assert!(pair_to_hex(&['0' as u8, '/' as u8]).is_none());
        assert!(pair_to_hex(&['0' as u8, ':' as u8]).is_none());

        assert!(pair_to_hex(&['0' as u8, '`' as u8]).is_none());
        assert!(pair_to_hex(&['0' as u8, 'g' as u8]).is_none());

        assert!(pair_to_hex(&['0' as u8, '@' as u8]).is_none());
        assert!(pair_to_hex(&['0' as u8, 'G' as u8]).is_none());
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
            (
                "a1¹²}ff™± ¡¿⅛°±£™⅞`ff¿¡⅛°±£™01`",
                vec![0xa1, 0xff, 0xff, 0x01],
            ),
        ];

        for (mut sample, expected) in samples
            .into_iter()
            .map(|(s, e)| (std::io::Cursor::new(s), e))
        {
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

        for (mut sample, expected) in samples
            .into_iter()
            .map(|(s, e)| (std::io::Cursor::new(s), e))
        {
            let got = {
                let mut tmp = Vec::new();
                let _ = encode(&mut sample, &mut tmp);
                String::from_utf8(tmp).unwrap()
            };

            assert_eq!(expected, got);
        }
    }
}
