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
    Data(String, Vec<CDPathNode>, CData),
}

pub struct LineParser<'a> {
    tk: CardTokenizer<'a>,
    vars: BTreeMap<String, CData>,
    peek: Option<Token<'a, CardToken>>,
    default: BTreeMap<String, CData>,
    params: Vec<String>,
    curr_card: Option<Card>,
}

impl<'a> LineParser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            tk: CardTokenizer::new(s),
            vars: BTreeMap::new(),
            peek: None,
            default: BTreeMap::new(),
            params: Vec::new(),
            curr_card: None,
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

    pub fn cdata_path(&mut self) -> CardRes<Vec<CDPathNode>> {
        let mut res = Vec::new();
        loop {
            match resop!(self.peek_token(), "Path or Value").value {
                CardToken::Star => {
                    self.unpeek();
                    res.push(CDPathNode::Append);
                }
                CardToken::Dot => {
                    self.unpeek();
                    res.push(CDPathNode::DigLast);
                }
                _ => return Ok(res),
            }
        }
    }

    pub fn peek_value<'b>(&'b mut self) -> Result<Option<&'b CardToken>, TErr> {
        Ok(Some(&resop!(self.peek_token()).value))
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

    pub fn values(&mut self, list: bool) -> CardRes<Vec<CData>> {
        let mut res = Vec::new();
        loop {
            let pk = match self.peek_token()? {
                None => return Ok(res),
                Some(p) => p,
            };
            match pk.value {
                CardToken::DollarVar(_)
                | CardToken::Number(_)
                | CardToken::Minus
                | CardToken::Text(_)
                | CardToken::SquareOpen => res.push(self.value()?),
                CardToken::Comma => {
                    self.unpeek();
                }
                CardToken::Break | CardToken::Colon => match list {
                    true => self.unpeek(),
                    false => return Ok(res),
                },
                CardToken::SquareClose => match list {
                    true => return Ok(res),
                    false => return Err(CardErr::Expected("Value").got(pk)),
                },
                _ => return Err(CardErr::Expected("Value").got(pk)),
            }
        }
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
            CardToken::Text(tx) => Ok(CData::S(tx.clone())),
            CardToken::SquareOpen => {
                let v = self.values(true)?;
                self.consume(|t| t.eq_option(&CardToken::SquareClose), "Close List")?;
                Ok(CData::L(v))
            }
            _ => expected("A Value", &t),
        }
    }

    pub fn next_line(&mut self) -> CardRes<Option<Line>> {
        self.breaks()?;
        let nt = resop!(self.peek_token()).clone();
        match nt.value {
            CardToken::Number(num) => {
                self.unpeek();
                self.consume(|t| t.eq_option(&CardToken::Star), "Star")?;
                let name = self.consume(|t| t.as_text(), "Card Name")?;
                let params = self.values(false)?;
                self.maybe_consume(|t| t.eq_option(&CardToken::Colon))?;
                Ok(Some(Line::Card {
                    name,
                    num: num as usize,
                    params,
                }))
            }
            CardToken::Text(name) => {
                self.unpeek();
                let params = self.values(false)?;
                self.maybe_consume(|t| t.eq_option(&CardToken::Colon))?;
                Ok(Some(Line::Card {
                    name: name.clone(),
                    num: 1,
                    params,
                }))
            }
            CardToken::KwParam => {
                //eg: @param size strength type
                self.unpeek();
                let mut pp = Vec::new();
                while let Some(tk) = self.peek_token()? {
                    if let CardToken::Text(t) = &tk.value {
                        pp.push(t.to_string());
                        self.unpeek();
                    } else {
                        return Ok(Some(Line::Param(pp)));
                    };
                }
                Ok(Some(Line::Param(pp)))
            }
            CardToken::KwDef => {
                self.unpeek();
                let v = self.values(false)?;
                self.maybe_consume(|t| t.eq_option(&CardToken::Colon))?;
                Ok(Some(Line::DefaultData(v)))
            }
            CardToken::KwConst => {
                self.unpeek();
                let name = self.consume(|t| t.as_text(), "Var Name")?;
                self.maybe_consume(|t| t.eq_option(&CardToken::Colon))?;
                let v = self.value()?;
                Ok(Some(Line::VarDef(name, v)))
            }
            CardToken::Dot => {
                self.unpeek();
                let name = self.consume(CardToken::as_text, "Property Name")?;
                let path = self.cdata_path()?;
                //let post = self.maybe_consume(CardToken::as_dots)?.unwrap_or(0);
                self.consume(|v| v.eq_option(&CardToken::Colon), "Colon")?;
                let v = self.value()?;
                Ok(Some(Line::Data(name, path, v)))
            }
            _ => expected("An entry ", &nt),
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
        self.breaks()?;
        loop {
            let ln = match self.next_line()? {
                Some(ln) => ln,
                None => match self.curr_card.take() {
                    Some(mut curr) => {
                        curr.fill_defaults(&self.default);
                        return Ok(Some(curr));
                    }
                    None => return Ok(None),
                },
            };
            match ln {
                Line::DefaultData(params) => {
                    if let Some(curr) = &mut self.curr_card {
                        curr.fill_defaults(&self.default);
                    }

                    self.default = self.fill_params(params)?;
                    if let Some(tres) = self.curr_card.take() {
                        return Ok(Some(tres));
                    }
                }
                Line::VarDef(name, val) => {
                    self.vars.insert(name, val);
                }
                Line::Param(v) => {
                    self.params = v;
                }
                Line::Card { num, name, params } => {
                    let tres = self.curr_card.take();

                    self.curr_card = Some(Card {
                        num,
                        name,
                        data: self.fill_params(params)?,
                    });

                    if let Some(mut curr) = tres {
                        curr.fill_defaults(&self.default);
                        return Ok(Some(curr));
                    }
                }
                Line::Data(k, path, val) => {
                    let tree = match &mut self.curr_card {
                        Some(r) => &mut r.data,
                        None => &mut self.default,
                    };

                    match tree.get_mut(&k) {
                        Some(c) => {
                            c.add_at_path(val, &path)
                                .map_err(|e| e.at(self.tk.peek_pos()))?;
                        }
                        None => {
                            let v = CData::build_from_path(val, &path);
                            tree.insert(k, v);
                        }
                    }
                }
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
