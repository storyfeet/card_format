use gobble::*;
use std::collections::BTreeMap;
use std::str::FromStr;

pub enum EType {
    Card,
    Var,
    Def,
}

#[derive(Debug)]
pub enum CData {
    S(String),
    N(isize),
    R(String),
}

#[derive(Debug)]
pub struct CExpr {
    num: usize,
    name: String,
    udef: bool,
    props: BTreeMap<String, CData>,
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

pub fn card_expr() -> impl Parser<CExpr> {
    n_item(0)
        .ig_then(maybe(uint().then_ig(s_tag("*"))))
        .then(read_fs(is_alpha_num, 1))
        .then(maybe(s_tag("$")))
        .then_ig(s_tag(":"))
        .then(repeat(props(), 0))
        .map(|(((num, name), udef), vals)| {
            let mut props = BTreeMap::new();
            for (dname, cdat) in vals {
                props.insert(dname, cdat);
            }
            CExpr {
                num: num.unwrap_or(1),
                name,
                udef: udef.is_some(),
                props,
            }
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
