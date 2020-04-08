pub mod parse;
use failure_derive::*;
use gobble::Parser;
pub use parse::{CData, EType};
use serde_derive::*;
use std::collections::BTreeMap;
use std::io::Read;

#[derive(Debug, Fail)]
pub enum CardErr {
    #[fail(display = "File Error")]
    FileErr,
    #[fail(display = "Parse Error")]
    ParseErr,
    #[fail(display = "Error referencing {} from {}", 0, 1)]
    RefErr(String, String),
}

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub num: usize,
    pub name: String,
    pub data: BTreeMap<String, CData>,
}

impl Card {
    pub fn build(name: String, num: usize, data: BTreeMap<String, CData>) -> Card {
        Card { name, num, data }
    }

    pub fn fill_defaults(&mut self, rmap: &BTreeMap<String, CData>) {
        for (k, v) in rmap {
            if self.data.get(k).is_none() {
                self.data.insert(k.clone(), v.clone());
            }
        }
    }

    pub fn follow_refs(&mut self, rmap: &BTreeMap<String, BTreeMap<String, CData>>) {
        for (k, v) in &mut self.data {
            if let CData::R(r) = v {
                if let Some(mp) = rmap.get(r) {
                    if let Some(nv) = mp.get(k) {
                        *v = nv.clone();
                    }
                }
            }
        }
    }
}

pub fn load_cards<R: Read>(r: &mut R) -> Result<Vec<Card>, CardErr> {
    let mut default = None;
    let mut vars = BTreeMap::new();
    let mut s = String::new();
    let mut res = Vec::new();
    r.read_to_string(&mut s).map_err(|_| CardErr::FileErr)?;
    let c_exs = parse::card_file()
        .parse_s(&s)
        .map_err(|_| CardErr::ParseErr)?;
    for (et, c) in c_exs {
        match et {
            EType::Var => {
                vars.insert(c.name, c.props);
            }
            EType::Def => default = Some(c.props),
            EType::Card(n) => {
                let mut crd = Card::build(c.name.clone(), n, c.props);
                if let Some(vref) = c.use_var {
                    let ndat = vars.get(&vref).ok_or(CardErr::RefErr(c.name, vref))?;
                    crd.fill_defaults(ndat);
                } else if let Some(ref ddat) = default {
                    crd.fill_defaults(ddat);
                }
                crd.follow_refs(&vars);
                res.push(crd)
            }
        }
    }
    Ok(res)
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
