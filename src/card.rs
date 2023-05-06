use crate::err::CardErr;
use serde_derive::*;
use serde::Serializer as SS;
use std::collections::BTreeMap;
use std::fmt::{self, Display};
use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap};

#[derive(Clone, Debug, PartialEq)]
pub enum CDPathNode {
    DigLast,
    Append,
    AtKey(String),
}
#[derive(Debug, PartialEq, Clone)]
pub enum CData {
    S(String),
    N(isize),
    L(Vec<CData>),
    M(BTreeMap<String, CData>),
}

impl serde::Serialize for CData {
    fn serialize<S:SS>(&self,ser: S) -> Result<<S as SS>::Ok,<S as SS>::Error> {
        match self {
            CData::S(s) => ser.serialize_str(s),
            CData::N(n) => ser.serialize_i64(*n as i64),
            CData::L(l) => {
                let mut seq = ser.serialize_seq(Some(l.len()))?;
                for e in l {
                    seq.serialize_element(e);
                }
                seq.end()
            }
            CData::M(m) => {
                let mut map = ser.serialize_map(Some(m.len()))?;
                for (k,v) in m {
                    map.serialize_entry(k,v);
                }
                map.end()
            }

        }
    }
}

impl Display for CData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CData::S(s) => write!(f, r#""{}""#, s),
            CData::N(n) => write!(f, "{}", n),
            CData::L(l) => {
                let mut pre = "[";
                for item in l {
                    write!(f, "{}{}", pre, item)?;
                    pre = ",";
                }
                write!(f, "]")
            }
            CData::M(m) => {
                let mut pre = "{";
                for (k, v) in m {
                    write!(f, "{}{}:{}", pre, k, v)?;
                    pre = ",";
                }
                write!(f, "{}", "}")
            }
        }
    }
}

impl CData {
    pub fn wrap(mut self, w: usize) -> Self {
        for _ in 0..w {
            self = CData::L(vec![self]);
        }
        self
    }

    pub fn add_at_path(&mut self, c: CData, path: &[CDPathNode]) -> Result<(), CardErr> {
        match (self, path.get(0)) {
            (CData::L(l), Some(CDPathNode::DigLast)) => match l.last_mut() {
                Some(ls) => return ls.add_at_path(c, &path[1..]),
                None => l.push(Self::build_from_path(c, &path[1..])),
            },
            (CData::L(l), _) => l.push(Self::build_from_path(c, &path[1..])),
            (CData::M(m), Some(CDPathNode::AtKey(k))) => match m.get_mut(k) {
                Some(v) => v.add_at_path(c, &path[1..])?,
                None => {
                    m.insert(k.clone(), CData::build_from_path(c, &path[1..]));
                }
            },
            (_, _) => return Err(CardErr::S("Could not add child at path")),
        }
        Ok(())
    }

    pub fn build_from_path(c: CData, path: &[CDPathNode]) -> CData {
        match path.get(0) {
            Some(CDPathNode::AtKey(k)) => {
                let mut mp = BTreeMap::new();
                mp.insert(k.clone(), CData::build_from_path(c, &path[1..]));
                CData::M(mp)
            }
            Some(_) => CData::L(vec![CData::build_from_path(c, &path[1..])]),
            None => c,
        }
    }

    pub fn add_child(&mut self, c: CData, depth: usize) -> Result<(), CardErr> {
        match self {
            CData::L(l) => {
                if depth <= 0 {
                    l.push(c);
                    return Ok(());
                }
                match l.last_mut() {
                    None => l.push(c.wrap(depth - 1)),
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
    pub fn flatten(mut self)->CData{
        self.data.insert("name".to_string(), CData::S(self.name));
        self.data.insert("num".to_string(), CData::N(self.num as isize));
        CData::M(self.data)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}*{} : ", self.num, self.name)?;
        for (k, v) in &self.data {
            writeln!(f, ".{}:{}", k, v)?;
        }
        Ok(())
    }
}
