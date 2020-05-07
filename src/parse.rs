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

pub fn card_file() -> impl Parser<Vec<(EType, CExpr)>> {
    repeat_until_ig(c_type_expr(), n_item(0).then_ig(eoi))
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

pub fn sl() -> impl Parser<()> {
    repeat(" ".or("\t").or("\n"), 0).map(|_| ())
}

pub fn sl_tag(t: &'static str) -> impl Parser<&'static str> {
    sl().ig_then(tag(t)).then_ig(sl())
}

pub fn c_data<'a>(it: &LCChars<'a>) -> ParseRes<'a, CData> {
    let p = (common_int.map(|v| CData::N(v)))
        .or(str_val().map(|s| CData::S(s)))
        .or(s_tag("$").ig_then(str_val()).map(|s| CData::R(s)))
        .or(s_tag("[")
            .ig_then(sep_until(c_data, sl_tag(","), s_tag("]")))
            .map(|l| CData::L(l)));
    p.parse(it)
}

pub fn props() -> impl Parser<(String, CData)> {
    (n_item(0), s_("."))
        .ig_then(str_val())
        .then_ig(s_(":"))
        .then(c_data)
}

pub fn c_type_expr() -> impl Parser<(EType, CExpr)> {
    n_item(0).ig_then((
        or3(
            keyword("def").map(|_| EType::Def),
            keyword("var").map(|_| EType::Var),
            maybe(common_uint.then_ig(ws_("*"))).map(|opt| EType::Card(opt.unwrap_or(1))),
        ),
        ws_(card_expr()),
    ))
}

pub fn card_expr() -> impl Parser<CExpr> {
    str_val()
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
