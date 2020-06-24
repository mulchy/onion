use anyhow::{ensure, Result};
use std::io::{Error, ErrorKind::InvalidInput};

fn count_ones(n: u8) -> u8 {
    let mut n = n;
    let mut count = 0;
    for _ in 0..7 {
        n >>= 1;
        if n & 1 == 1 {
            count += 1;
        }
    }
    count
}

fn parity(n: u8) -> bool {
    count_ones(n) % 2 == 1
}

fn get_parity_bit(n: u8) -> bool {
    (n & 1) == 1
}

fn correct_parity(n: u8) -> bool {
    parity(n) == get_parity_bit(n)
}

#[test]
fn test_count_ones() {
    assert_eq!(0, count_ones(0b0000_0000));
    assert_eq!(1, count_ones(0b0000_0010));
    assert_eq!(1, count_ones(0b0100_0000));
    assert_eq!(7, count_ones(0b1111_1110));
}

#[test]
fn test_parity() {
    assert_eq!(0, parity(0b0000_0000) as u8);
    assert_eq!(1, parity(0b0000_0010) as u8);
    assert_eq!(0, parity(0b0010_0010) as u8);
    assert_eq!(1, parity(0b0100_0000) as u8);
    assert_eq!(1, parity(0b1111_1110) as u8);
}

#[test]
fn test_correct_parity() {
    assert!(correct_parity(0b1011_0010));
    assert!(correct_parity(0b0000_0000));
    assert!(correct_parity(0b1111_1111));
}

fn combine(bytes: &[u8]) -> Result<Vec<u8>> {
    let good_bytes: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|&b| correct_parity(b))
        .collect();

    ensure!(
        good_bytes.len() % 8 == 0,
        Error::new(InvalidInput, "input needs to be a multiple of 8 bytes")
    );

    Ok(good_bytes
        .as_slice()
        .chunks(8)
        .map(|bytes| {
            // drop parity bit
            let bytes: Vec<u8> = bytes.iter().map(|b| b >> 1).collect();

            // line up the bits 7 at a time
            let mut temp: u64 = 0;
            for (i, &byte) in bytes.iter().enumerate().take(8) {
                let shift = 7 * (7 - i);
                temp += ((byte as u64) << shift) as u64;
            }

            let mut out: Vec<u8> = Vec::new();
            // read off 7 bytes
            for _ in 0..7 {
                out.insert(0, (temp & 0b1111_1111) as u8); //todo this feels bad
                temp >>= 8;
            }

            out
        })
        .flatten()
        .collect())
}

pub fn run(bytes: &[u8]) -> Result<Vec<u8>> {
    let decoded = super::super::ascii85::decode(bytes)?;
    combine(&decoded)
}
