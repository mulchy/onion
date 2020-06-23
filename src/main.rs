mod ascii85;
use anyhow::Result;
use ascii85::decode;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    let mut f = File::open("input.txt")?;
    let mut buffer = f
        .metadata()
        .map_or_else(|_| Vec::new(), |m| Vec::with_capacity(m.len() as usize));

    f.read_to_end(&mut buffer)?;

    let decoded = decode(buffer.as_slice())?;
    let mut layer1 = File::create("layer1.txt")?;
    layer1.write_all(&decoded)?;

    

    Ok(())
}
