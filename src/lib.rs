pub mod card;
pub mod parse;
pub mod tokenize;
use card::Card;
//use failure_derive::*;
use gobble::err::StrungError;
use gobble::traits::*;
pub use parse::{CData, CVec, Entry};
use std::collections::BTreeMap;
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
    #[error("No Card to add {}:{:?} to", .0,.1)]
    AddErr(String, CData),
}

fn c_map(v: CVec) -> BTreeMap<String, CData> {
    v.into_iter().collect()
}

pub fn parse_cards(s: &str) -> Result<Vec<Card>, CardErr> {
    let mut default = BTreeMap::new();
    let mut param_names = Vec::new();
    let mut vars = BTreeMap::new();
    let mut res = Vec::new();

    let c_exs = parse::CardFile
        .parse_s(&s)
        .map_err(|e| CardErr::ParseErr(e.strung()))?;
    for entry in c_exs {
        match entry {
            Entry::Def(data) => default = c_map(data),
            Entry::Var(name, data) => {
                vars.insert(name, c_map(data));
            }
            Entry::Param(v) => {
                param_names = v;
            }
            Entry::Card {
                num,
                name,
                params,
                parent,
                data,
            } => {
                let mut crd = Card::build(name.clone(), num, c_map(data));
                if let Some(vref) = parent {
                    let ndat = vars.get(&vref).ok_or(CardErr::RefErr(name, vref))?;
                    crd.fill_defaults(ndat);
                } else {
                    crd.fill_defaults(&default);
                }
                for (n, p) in params.into_iter().enumerate() {
                    let k = param_names
                        .get(n)
                        .map(|s| s.to_string())
                        .unwrap_or(n.to_string());
                    crd.data.insert(k, p);
                }
                crd.follow_refs(&vars);
                res.push(crd);
            }
        }
    }
    Ok(res)
}

pub fn load_cards<R: Read>(r: &mut R) -> Result<Vec<Card>, CardErr> {
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
