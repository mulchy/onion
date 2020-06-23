mod ascii85;
mod layers;
use anyhow::{Result, bail};
use std::fs::File;
use std::io::prelude::*;

use layers::*;

pub fn read() -> Result<Vec<u8>> {
    let mut f = File::open("input.txt")?;
    let mut buffer = f
        .metadata()
        .map_or_else(|_| Vec::new(), |m| Vec::with_capacity(m.len() as usize));

    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<()> {
    let layer1_input = read()?;
    let layer1 = layer1::run(&layer1_input)?;
    let mut layer1_out = File::create("layer1.txt")?;
    layer1_out.write_all(&layer1)?;



    let layer2input = find_input( &String::from_utf8(layer1)?);
    if layer2input.is_none() {
        bail!("Layer 1 is not valid utf-8");
    }
    let layer2 = layer2::run(layer2input.unwrap().as_slice())?;
    let mut layer2_out = File::create("layer2.txt")?;
    layer2_out.write_all(&layer2)?;


    
    let layer3input = find_input( &String::from_utf8(layer2)?);
    if layer3input.is_none() {
        bail!("Layer 2 is not valid utf-8");
    }
    let layer3 = layer3::run(layer3input.unwrap().as_slice())?;
    let mut layer3_out = File::create("layer3.txt")?;
    layer3_out.write_all(&layer3)?;

    Ok(())
}