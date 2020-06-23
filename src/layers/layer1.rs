use super::super::ascii85::decode;
use anyhow::Result;

pub fn run(bytes: &[u8]) -> Result<Vec<u8>> {
    decode(bytes)
}
