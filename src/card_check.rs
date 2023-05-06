extern crate card_format;
use std::io::{self, Read};
use clap::{Arg,arg,Command,crate_version};
use card_format::card::{Card,CData};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {

    let matches = Command::new("card_check")
        .version(crate_version!())
        .about("A program to check and convert card_format from stdin to stdout")
        .author("Matthew Stoodley (storyfeet)")
        .subcommand(Command::new("json"))
        .get_matches();
  

    let mut buf = String::new();
    let mut sin = io::stdin();
    sin.read_to_string(&mut buf)?;

    let ar = card_format::parse_cards(&buf)?;

    match matches.subcommand() {
        Some(("json",_)) => {
            let mp:Vec<CData> = ar.into_iter().map(Card::flatten).collect();
            
            print!("{}", serde_json::to_string_pretty(&mp)?);
        }

        _ => for (i, c) in ar.iter().enumerate() {
            println!("{} = {}", i, c);
        }
    }



    

    Ok(())
}



