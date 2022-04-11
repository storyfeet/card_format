pub mod card;
pub mod parse;
pub mod tokenize;
use card::Card;
//use failure_derive::*;
use gobble::err::StrungError;
//use gobble::traits::*;
//pub use parse::{CData, CVec, Entry};
//use std::collections::BTreeMap;
use std::io::Read;
use thiserror::*;

#[derive(Debug, Error)]
pub enum CardErr {
    #[error("File Error")]
    FileErr,
    #[error("Parse Error: {}", .0)]
    ParseErr(StrungError),
    #[error("Error referencing {} from {}", .0, .1)]
    RefErr(String, String),
    // #[error("No Card to add {}:{:?} to", .0,.1)]
    //  AddErr(String, CData),
}

/*fn c_map(v: CVec) -> BTreeMap<String, CData> {
    v.into_iter().collect()
}*/

pub fn parse_cards(s: &str) -> anyhow::Result<Vec<Card>> {
    let mut p = parse::LineParser::new(s);
    p.parse_cards()
}

pub fn load_cards<R: Read>(r: &mut R) -> anyhow::Result<Vec<Card>> {
    let mut s = String::new();
    r.read_to_string(&mut s).map_err(|_| CardErr::FileErr)?;
    parse_cards(&s)
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::fs::File;
    #[test]
    pub fn test_can_load_cards() {
        let mut f = File::open("test_data/cards1.card").unwrap();
        let cds = load_cards(&mut f).unwrap();
        assert_eq!(cds[1].data.get("do"), Some(&CData::S("paint".to_string())));
        assert_eq!(
            cds[1].data.get("speak"),
            Some(&CData::S("mauve".to_string()))
        );
        assert_eq!(cds[0].data.get("speak"), Some(&CData::S("no".to_string())));
    }
}
