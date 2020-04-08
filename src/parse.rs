use gobble::*;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EType {
    Def,
    Var,
    Card(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CData {
    S(String),
    N(isize),
    R(String),
}

impl Serialize for CData {
    fn serialize<S>(&self, sr: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CData::S(s) => sr.serialize_str(s),
            CData::N(i) => sr.serialize_i64(*i as i64),
            CData::R(r) => sr.serialize_str(r),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CExpr {
    pub name: String,
    pub use_var: Option<String>,
    pub props: BTreeMap<String, CData>,
}

pub fn card_file() -> impl Parser<Vec<(EType, CExpr)>> {
    repeat(c_type_expr(), 0).then_ig(n_item(0)).then_ig(eoi)
}

pub fn uint() -> impl Parser<usize> {
    read_fs(is_num, 1).try_map(|v| usize::from_str(&v).map_err(|_| ECode::SMess("Num Too long")))
}

pub fn int() -> impl Parser<isize> {
    ws(0).ig_then(maybe(tag("-"))).then(uint()).map(|(o, v)| {
        if o.is_some() {
            -(v as isize)
        } else {
            v as isize
        }
    })
}

pub fn n_item(n: usize) -> impl Parser<()> {
    take(
        |c| match c {
            ' ' | '\n' | '\t' | ';' | 'r' => true,
            _ => false,
        },
        n,
    )
}

pub fn str_val() -> impl Parser<String> {
    common_str().or(read_fs(is_alpha_num, 1))
}

pub fn c_data() -> impl Parser<CData> {
    (int().map(|v| CData::N(v)))
        .or(str_val().map(|s| CData::S(s)))
        .or(s_tag("$").ig_then(str_val()).map(|s| CData::R(s)))
}

pub fn props() -> impl Parser<(String, CData)> {
    n_item(0)
        .ig_then(s_tag("."))
        .ig_then(read_fs(is_alpha_num, 1))
        .then_ig(s_tag(":"))
        .then(c_data())
}

pub fn c_type_expr() -> impl Parser<(EType, CExpr)> {
    n_item(0)
        .ig_then(
            (s_tag("def").map(|_| EType::Def))
                .or(s_tag("var").map(|_| EType::Var))
                .or(maybe(uint().then_ig(s_tag("*"))).map(|opn| EType::Card(opn.unwrap_or(1)))),
        )
        .then(card_expr())
}

pub fn card_expr() -> impl Parser<CExpr> {
    read_fs(is_alpha_num, 1)
        .then(maybe(s_tag("$").ig_then(str_val())))
        .then_ig(s_tag(":"))
        .then(repeat(props(), 0))
        .map(|((name, use_var), vals)| {
            let mut props = BTreeMap::new();
            for (dname, cdat) in vals {
                props.insert(dname, cdat);
            }
            CExpr {
                name,
                use_var,
                props,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    #[test]
    fn it_works() {
        let mut f = std::fs::File::open("test_data/cards1.card").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let cf = card_file().parse_s(&s).unwrap();
        assert_eq!(cf[0].1.name, "green");
        assert_eq!(cf[1].1.name, "help");
        assert_eq!(cf[0].0, EType::Def);
        assert_eq!(cf[1].0, EType::Card(4));
    }
}
