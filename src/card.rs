use serde_derive::*;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CData {
    S(String),
    N(isize),
    R(String),
    L(Vec<CData>),
}

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub num: usize,
    pub name: String,
    pub data: BTreeMap<String, CData>,
}

impl Card {
    pub fn new(name: String, num: usize) -> Card {
        Card {
            name,
            num,
            data: BTreeMap::new(),
        }
    }
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
