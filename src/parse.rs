use crate::card::*;
use crate::err::{expected, CardErr};
use crate::tokenize::{CardToken, CardTokenizer};
use crate::CardRes;
use std::collections::BTreeMap;
use tokenate::{TErr, Token, TokenRes};

macro_rules! resop {
    ($e:expr) => {
        match $e? {
            Some(s) => s,
            None => return Ok(None),
        }
    };
    ($e:expr,$err:expr) => {
        match $e? {
            Some(s) => s,
            None => return Err(CardErr::Expected($err).eof()),
        }
    };
}

pub type CVec = Vec<(String, CData)>;

#[derive(Debug, PartialEq, Clone)]
pub enum Line {
    DefaultData(Vec<CData>),
    VarDef(String, CData),
    Param(Vec<String>),
    Card {
        num: usize,
        name: String,
        params: Vec<CData>,
    },
    Data(String, usize, usize, CData),
}

pub struct LineParser<'a> {
    tk: CardTokenizer<'a>,
    vars: BTreeMap<String, CData>,
    peek: Option<Token<'a, CardToken>>,
    default: BTreeMap<String, CData>,
    params: Vec<String>,
    res: Option<Card>,
}

impl<'a> LineParser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            tk: CardTokenizer::new(s),
            vars: BTreeMap::new(),
            peek: None,
            default: BTreeMap::new(),
            params: Vec::new(),
            res: None,
        }
    }
    pub fn add_var(&mut self, k: String, v: CData) {
        self.vars.insert(k, v);
    }

    pub fn next_token(&mut self) -> TokenRes<'a, CardToken> {
        match self.peek.take() {
            Some(c) => Ok(Some(c)),
            None => self.tk.next(),
        }
    }

    pub fn consume<T, F: Fn(&CardToken) -> Option<T>>(
        &mut self,
        f: F,
        exp: &'static str,
    ) -> CardRes<T> {
        let t = match self.next_token()? {
            Some(t) => t,
            None => return Err(CardErr::Expected(exp).eof()),
        };
        match f(&t.value) {
            Some(t) => Ok(t),
            None => expected(exp, &t),
        }
    }

    pub fn maybe_consume<T, F: Fn(&CardToken) -> Option<T>>(&mut self, f: F) -> CardRes<Option<T>> {
        let t = resop!(self.peek_token());
        match f(&t.value) {
            Some(v) => {
                self.unpeek();
                Ok(Some(v))
            }
            None => Ok(None),
        }
    }

    pub fn peek_token<'b>(&'b mut self) -> Result<Option<&Token<'a, CardToken>>, TErr> {
        if self.peek.is_none() {
            self.peek = self.tk.next()?;
        }
        match &self.peek {
            Some(t) => Ok(Some(t)),
            None => Ok(None),
        }
    }

    pub fn unpeek(&mut self) {
        self.peek = None;
    }

    pub fn breaks(&mut self) -> Result<(), TErr> {
        while let Some(p) = self.peek_token()? {
            if p.value == CardToken::Break {
                self.unpeek();
            } else {
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn value(&mut self) -> CardRes<CData> {
        let t = resop!(self.next_token(), "Value");
        match &t.value {
            CardToken::DollarVar(v) => match self.vars.get(v) {
                Some(v) => Ok(v.clone()),
                None => expected("Var does not exist", &t),
            },
            CardToken::Number(n) => Ok(CData::N(*n)),
            CardToken::Minus => self
                .consume(|v| v.as_number(), "Number")
                .map(|n| CData::N(-n)),
            _ => expected("A Value", &t),
        }
    }

    pub fn next_line(&mut self) -> CardRes<Option<Line>> {
        self.breaks()?;
        let nt = resop!(self.peek_token());
        match nt.value {
            CardToken::KwParam => {
                //eg: @param size strength type
                self.unpeek();
                let mut pp = Vec::new();
                while let Some(tk) = self.peek_token()? {
                    if let CardToken::Text(t) = &tk.value {
                        pp.push(t.to_string());
                    } else {
                        return Ok(Some(Line::Param(pp)));
                    };
                }
                Ok(Some(Line::Param(pp)))
            }
            CardToken::Dots(pre) => {
                let name = self.consume(CardToken::as_text, "Property Name")?;
                let post = self.maybe_consume(CardToken::as_dots)?.unwrap_or(0);
                self.consume(|v| v.eq_option(&CardToken::Colon), "Colon")?;
                let v = self.value()?;
                Ok(Some(Line::Data(name, pre, post, v)))
            }
            _ => expected("An entry ", nt),
        }
    }

    fn fill_params(&mut self, v: Vec<CData>) -> CardRes<BTreeMap<String, CData>> {
        let mut defdata = BTreeMap::new();
        for (n, p) in v.into_iter().enumerate() {
            defdata.insert(
                self.params
                    .get(n)
                    .ok_or(CardErr::S("Not enough params defined before").at(self.tk.peek_pos()))?
                    .to_string(),
                p,
            );
        }
        Ok(defdata)
    }

    pub fn next_card(&mut self) -> CardRes<Option<Card>> {
        self.breaks();
        loop {
            match resop!(self.next_line()) {
                Line::DefaultData(params) => {
                    self.default = self.fill_params(params)?;
                    if let Some(r) = self.res.take() {
                        return Ok(Some(r));
                    }
                }
                Line::VarDef(name, val) => {
                    self.vars.insert(name, val);
                }
                Line::Param(v) => {
                    self.params = v;
                }
                Line::Card { num, name, params } => {
                    let mut tres = Card {
                        num,
                        name,
                        data: self.fill_params(params)?,
                    };

                    for (k, v) in &self.default {
                        if tres.data.get(k).is_none() {
                            tres.data.insert(k.to_string(), v.clone());
                        }
                    }

                    match self.res.take() {
                        Some(r) => {
                            self.res = Some(tres);
                            return Ok(Some(r));
                        }
                        None => self.res = Some(tres),
                    }
                }
                Line::Data(k, pre, post, val) => match &mut self.res {
                    Some(r) => {
                        r.data.insert(k, val);
                    }
                    None => {
                        self.default.insert(k, val);
                    }
                },
            }
        }
    }

    pub fn parse_cards(&mut self) -> CardRes<Vec<Card>> {
        let mut res = Vec::new();
        while let Some(c) = self.next_card()? {
            res.push(c);
        }
        Ok(res)
    }
}

#[cfg(gods)]
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
    }
}
