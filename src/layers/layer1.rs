use anyhow::Result;

use super::super::ascii85::decode;

pub fn flip_every_other_bit(n: u8) -> u8 {
    let mask = 0b0101_0101;
    n ^ mask
}

pub fn rotate_right(n: u8) -> u8 {
    let last_bit = n & 1;
    (n >> 1) | (last_bit << 7)
}

#[test]
fn test_flip_every_other_bit() {
    let input = 0b1010_1010;
    let expected = 0b1111_1111;
    assert_eq!(flip_every_other_bit(input), expected);
    assert_eq!(0b0000_0000, flip_every_other_bit(0b0101_0101));
    assert_eq!(0b0101_0101, flip_every_other_bit(0b0000_0000));
}

#[test]
fn test_rotate_right() {
    assert_eq!(0b1000_0000, rotate_right(0b0000_0001));
    assert_eq!(0b0101_0101, rotate_right(0b1010_1010));
    assert_eq!(0b1000_1000, rotate_right(0b0001_0001));
}

pub fn run(input: &[u8]) -> Result<Vec<u8>> {
    let decoded = decode(input)?;
    Ok(decoded
        .iter()
        .map(|&byte| rotate_right(flip_every_other_bit(byte)))
        .collect())
}
