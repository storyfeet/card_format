pub mod card;
pub mod err;
pub mod parse;
pub mod tokenize;
pub use card::{CData, Card};
pub use err::{CardErr, CardRes};

//use failure_derive::*;
//use gobble::traits::*;
//pub use parse::{CData, CVec, Entry};
//use std::collections::BTreeMap;
use std::io::Read;

/*fn c_map(v: CVec) -> BTreeMap<String, CData> {
    v.into_iter().collect()
}*/

pub fn parse_cards(s: &str) -> CardRes<Vec<Card>> {
    let mut p = parse::LineParser::new(s);
    p.parse_cards()
}

pub fn load_cards<R: Read>(r: &mut R) -> CardRes<Vec<Card>> {
    let mut s = String::new();
    r.read_to_string(&mut s).map_err(|_| err::AtErr::FileErr)?;
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
        assert_eq!(cds[0].data.get("speak"), Some(&CData::S("no".to_string())));
        assert_eq!(cds[1].data.get("do"), Some(&CData::S("paint".to_string())));
        assert_eq!(cds[1].data.get("speak"), Some(&CData::S("no".to_string())));
        let pets = cds[2].data.get("pets").unwrap().as_list().unwrap();
        assert_eq!(pets.get(1).unwrap(), &CData::S("cat".to_string()));
    }
    #[test]
    pub fn test_lists_work_both_ways() {
        let mut f = File::open("test_data/cards2_list.crd").unwrap();
        let cds = load_cards(&mut f).unwrap();
        assert_eq!(cds[0].data, cds[1].data);
    }
    #[test]
    pub fn test_maps_work_both_ways() {
        let mut f = File::open("test_data/cards3_maps.crd").unwrap();
        let cds = load_cards(&mut f).unwrap();
        assert_eq!(cds[0].data, cds[1].data);
    }
}
