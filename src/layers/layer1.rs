
use anyhow::Result;
use super::super::ascii85::decode;

pub fn run(bytes: &[u8]) -> Result<Vec<u8>> {
    decode(bytes)
}