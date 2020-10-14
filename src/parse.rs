use gobble::*;
use serde::ser::{Serialize, SerializeSeq, Serializer};

pub type CVec = Vec<(String, CData)>;

#[derive(Debug, PartialEq, Clone)]
pub enum Entry {
    Def(CVec),
    Var(String, CVec),
    Card {
        num: usize,
        name: String,
        params: Vec<CData>,
        parent: Option<String>,
        data: CVec,
    },
    Param(Vec<String>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CData {
    S(String),
    N(isize),
    R(String),
    L(Vec<CData>),
}

parser! {(NL-> ())
    (maybe('\r'),'\n').ig()
}

parser! { (EmptyLine-> ())
    (ws_(Comment),NL).ig()
}

parser! { (Comment->())
    maybe(("#",not("\r\n").star())).ig()
}

parser! {(SetEnd->())
    (ws__(maybe(":")),Comment,NL,star(EmptyLine)).ig()
}

parser! {(Count->usize)
    maybe(common::UInt.then_ig(ws_("*"))).map(|o| o.unwrap_or(1))
}

parser! { (Dot->())
    or!{
        (" \t".plus(),maybe(".")).ig(),
        ".".ig(),
    }
}

parser! {(PLine->Entry)
    debug(or!(
        (keyword("def"),SetEnd,DataLines).map(|(_,_,d)|Entry::Def(d)),
        (keyword("var"),ws_(str_val()),ws__(":"),NL,DataLines).map(|(_,name,_,_,d)|Entry::Var(name,d)),
        (keyword("param"),star(ws_(str_val())),SetEnd).map(|(_,v,_)|Entry::Param(v)),
        (Count,ws_(str_val()),star(ws_(",").ig_then(ws_(CardData))),maybe(ws_("$".ig_then(str_val()))),SetEnd,DataLines)
            .map(|(num,name,params,parent,_,data)|Entry::Card{num,name,params,parent,data}),
    ),"ENTRY")
}

parser! {(DataLines -> CVec )
    star(DataLine).map(|v|v.into_iter().filter_map(|v|v).collect())
}

parser! { (DataLine -> Option<(String,CData)>)
    or!(
        (Dot,str_val(),ws__(":"),CardData,ws_(NL)).map(|(_,k,_,v,_)|Some((k,v))),
        EmptyLine.map(|_|None),
    )
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

parser! {(CardFile ->Vec<Entry>)
    middle(star(EmptyLine),star(PLine),ws_(eoi))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::Read;
    #[test]
    fn it_works() {
        let mut map0 = BTreeMap::new();
        map0.insert("speak".to_string(), "no".to_string());
        map0.insert("do".to_string(), "yes".to_string());
        let mut f = std::fs::File::open("test_data/cards1.card").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let cf = CardFile.parse_s(&s).unwrap();
        assert_eq!(
            cf[0],
            Entry::Def(vec![
                ("speak".to_string(), CData::S("no".to_string())),
                ("do".to_string(), CData::S("yes".to_string())),
            ])
        );
        //assert_eq!(cf[1], Line::Def);
        /*assert_eq!(
            cf[2],
            Line::Set("speak".to_string(), CData::S("no".to_string()))
        );*/
    }
}
