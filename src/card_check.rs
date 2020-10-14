extern crate card_format;
use std::io::{self, Read};

pub fn main() -> anyhow::Result<()> {
    let mut buf = String::new();
    let mut sin = io::stdin();
    sin.read_to_string(&mut buf)?;

    let ar = card_format::parse_cards(&buf)?;

    for (i, c) in ar.iter().enumerate() {
        println!("{} = {:?}", i, c);
    }

    Ok(())
}
