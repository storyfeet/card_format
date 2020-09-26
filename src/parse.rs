use gobble::*;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::collections::BTreeMap;

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
    L(Vec<CData>),
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
            CData::L(l) => {
                let mut ser = sr.serialize_seq(Some(l.len()))?;
                for e in l {
                    ser.serialize_element(e)?;
                }
                ser.end()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CExpr {
    pub name: String,
    pub use_var: Option<String>,
    pub props: BTreeMap<String, CData>,
}

pub fn card_file() -> impl Parser<Out = Vec<(EType, CExpr)>> {
    star_until_ig(c_type_expr(), n_item().then_ig(eoi))
}

pub fn n_item() -> impl Parser<Out = ()> {
    " \n\t;\r".istar()
}

pub fn str_val() -> impl Parser<Out = String> {
    or(common::Quoted, (Alpha, NumDigit, '_').min_n(1))
}

pub fn sl() -> impl Parser<Out = ()> {
    " \t\n".istar()
}

pub fn sl_<P: Parser>(p: P) -> impl Parser<Out = P::Out> {
    wrap(sl(), p)
}

parser! { (CardData -> CData)
    or!(
        common::Int.map(|v| CData::N(v)),
        str_val().map(|s| CData::S(s)),
        ws_("$").ig_then(str_val()).map(|s| CData::R(s)),
        ws_("[").ig_then(sep_until_ig(CardData, sl_(","), ws_("]")))
            .map(|l| CData::L(l))
    )
}

pub fn props() -> impl Parser<Out = (String, CData)> {
    (n_item(), ".", str_val(), ws__(":"), CardData).map(|(_, _, k, _, v)| (k, v))
}

pub fn c_type_expr() -> impl Parser<Out = (EType, CExpr)> {
    n_item().ig_then((
        or!(
            keyword("def").map(|_| EType::Def),
            keyword("var").map(|_| EType::Var),
            maybe(common::UInt.then_ig(ws_("*"))).map(|opt| EType::Card(opt.unwrap_or(1))),
        ),
        ws_(card_expr()),
    ))
}

pub fn card_expr() -> impl Parser<Out = CExpr> {
    str_val()
        .then(maybe(ws__("$").ig_then(str_val())))
        .then_ig(ws__(":"))
        .then(star(props()))
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
