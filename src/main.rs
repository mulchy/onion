mod ascii85;
mod layers;
use anyhow::Result;

use layers::*;

fn main() -> Result<()> {
    let layer0_input = read_initial_input()?;
    let layer1 = layer0::run(&layer0_input)?;
    write_output("layer_1.txt", &layer1)?;

    let layer1input = find_input(&String::from_utf8(layer1)?)?;
    let layer2 = layer1::run(&layer1input)?;
    write_output("layer_2.txt", &layer2)?;

    let layer2input = find_input(&String::from_utf8(layer2)?)?;
    let layer3 = layer2::run(&layer2input)?;
    write_output("layer_3.txt", &layer3)?;

    let layer3input = find_input(&String::from_utf8(layer3)?)?;
    let layer4 = layer3::run(&layer3input)?;
    write_output("layer_4.txt", &layer4[..layer4.len() - 1])?;

    let layer4input = find_input(&String::from_utf8(layer4)?)?;
    let layer5 = layer4::run(&layer4input)?;
    write_output("layer_5.txt", &layer5)?;

    let layer5input = find_input(&String::from_utf8(layer5)?)?;
    let core = layer5::run(&layer5input)?;
    write_output("the_core.txt", &core)?;

    Ok(())
}
