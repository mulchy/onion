use anyhow::{ensure, Result};
use std::io::{Error, ErrorKind::InvalidInput};

#[allow(dead_code)]
pub fn encode(_bytes: &[u8]) -> Vec<u8> {
    panic!()
}

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut buffer = trim_start_and_end_delimeters(bytes)?;

    ensure!(
        buffer.iter().all(u8::is_ascii),
        Error::new(InvalidInput, "non-ascii input")
    );
    ensure!(
        buffer.iter().all(|&byte| {
            // let z = byte == 'z' as u8; //todo handle decompressing all zeroes
            let in_range = byte >= 33 && byte <= 117;
            let whitespace = byte == 10 || byte == 13; // lf cr
            in_range || whitespace
        }),
        Error::new(InvalidInput, "Found bytes outside of Ascii85 range.")
    );

    // silently ignore all whitespace in the encoded data
    buffer.retain(|byte| !byte.is_ascii_whitespace());

    let mut decoded: Vec<u8>;
    let iterator = buffer.chunks_exact(5);
    decoded = iterator.clone().map(decode_chunk).flatten().collect();

    // handle the last chunk
    let mut rem = iterator.remainder().to_vec();
    let pad = 5 - rem.len();
    if pad > 0 {
        for _ in 0..pad {
            rem.push(b'u');
        }
        let mut last = decode_chunk(rem.as_mut());
        last = last.into_iter().take(5 - pad).collect();
        decoded.append(&mut last);
    }

    Ok(decoded)
}

fn trim_start_and_end_delimeters(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut start = None;
    let mut end = None;

    // todo slice.windows(2)
    for i in 0..bytes.len() - 1 {
        let first = bytes[i];
        let second = bytes[i + 1];

        if first == 60 && second == 126 {
            start = Some(i + 2); // skip the 2 byte delimeter
        }

        if first == 126 && second == 62 {
            end = Some(i - 1);
        }
    }

    let start =
        start.ok_or_else(|| Error::new(InvalidInput, "missing Ascii85 start delimiter '<~"))?;
    let end = end.ok_or_else(|| Error::new(InvalidInput, "missing Ascii85 end delimeter '~>"))?;

    Ok(bytes[start..end].to_vec())
}

fn decode_chunk(chunk: &[u8]) -> Vec<u8> {
    assert!(chunk.len() == 5); // idris when

    let base: u64 = 85;
    let digit1 = (chunk[0] - 33) as u64;
    let digit2 = (chunk[1] - 33) as u64;
    let digit3 = (chunk[2] - 33) as u64;
    let digit4 = (chunk[3] - 33) as u64;
    let digit5 = (chunk[4] - 33) as u64;

    let input = digit1 * base.pow(4)
        + digit2 * base.pow(3)
        + digit3 * base.pow(2)
        + digit4 * base.pow(1)
        + digit5;

    let first_byte = (input >> 24 & 0xff) as u8;
    let second_byte = (input >> 16 & 0xff) as u8;
    let third_byte = (input >> 8 & 0xff) as u8;
    let fourth_byte = (input & 0xff) as u8;

    [first_byte, second_byte, third_byte, fourth_byte].to_vec()
}

#[test]
fn test_decode() -> Result<()> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut input = File::open("test/encoded.txt")?;
    let mut encoded = Vec::new();
    input.read_to_end(&mut encoded)?;

    let output = decode(encoded.as_mut())?;

    let expected = "Man is distinguished, not only by his reason, but by this singular passion from other animals, which is a lust of the mind, that by a perseverance of delight in the continued and indefatigable generation of knowledge, exceeds the short vehemence of any carnal pleasure.";

    assert_eq!(String::from_utf8(output)?, expected);
    Ok(())
}
