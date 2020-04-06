pub mod parse;
use gobble::Parser;
use parse::{CData,EType,CExpr};
use std::collections::BTreeMap;
use std::io::Read;

pub enum CardFmtErr {
    FileErr,
    ParseErr,
}

#[derive(Debug)]
pub struct Card {
    num: usize,
    name: String,
    data: BTreeMap<String, CData>,
}

impl Card{
    pub fn fill_
    pub fn from_expr(c:CExpr,num:usize,map:&BTreeMap<String,CData>)->Card{
        let mut rdata = BTreeMap::new():

        let res = Card{
            num,
            name:c.name,
            data:BTreeMap::new(),
            
        }
    }
}

pub fn parse_cards<R: Read>(r: &mut R) -> Result<Vec<Card>, String> {
    let mut default = None;
    let mut vars = BTreeMap::new();
    let mut s = String::new();
    r.read_to_string(&mut s);
    let c_exs = parse::card_file().parse_s(&s);
    for (et, c) in c_exs {
        match et {
            EType::Var=>vars.push(
        }
    }
}
