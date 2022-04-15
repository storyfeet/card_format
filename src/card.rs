use crate::err::CardErr;
use serde_derive::*;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CData {
    S(String),
    N(isize),
    R(String),
    L(Vec<CData>),
}

impl CData {
    pub fn wrap(mut self, w: usize) -> Self {
        for _ in 0..w {
            self = CData::L(vec![self]);
        }
        self
    }

    pub fn add_child(&mut self, c: CData, depth: usize) -> Result<(), CardErr> {
        match self {
            CData::L(l) => {
                if depth <= 0 {
                    l.push(c);
                    return Ok(());
                }
                match l.last_mut() {
                    None => l.push(c.wrap(depth)),
                    Some(ls) => {
                        ls.add_child(c, depth - 1)?;
                    }
                }
            }
            _ => return Err(CardErr::Unset),
        }
        Ok(())
    }

    pub fn as_list(&self) -> Option<&Vec<CData>> {
        match self {
            CData::L(l) => Some(l),
            _ => None,
        }
    }
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
