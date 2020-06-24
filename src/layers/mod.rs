pub mod layer0;
pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod layer4;
pub mod layer5;

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn find_input(haystack: &str) -> Result<Vec<u8>> {
    let needle = "==[ Payload ]===============================================";
    haystack
        .find(needle)
        .map(|idx| haystack[idx..].trim().as_bytes().to_vec())
        .ok_or_else(|| anyhow!("Couldn't find payload delimeter: {}", needle))
}

pub fn write_output(path: &str, bytes: &[u8]) -> Result<()> {
    let mut output_path = PathBuf::from("out");
    output_path.push(path);
    let mut f = File::create(output_path)?;
    f.write_all(bytes)?;

    Ok(())
}

pub fn read_initial_input() -> Result<Vec<u8>> {
    let mut f = File::open("input.txt")?;
    let mut buffer = f
        .metadata()
        .map_or_else(|_| Vec::new(), |m| Vec::with_capacity(m.len() as usize));

    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}
