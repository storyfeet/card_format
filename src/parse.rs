use crate::card::*;
use crate::tokenize::{CardToken, CardTokenizer};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::collections::BTreeMap;
use tokenate::{TErr, Token, TokenRes};

pub type CVec = Vec<(String, CData)>;

#[derive(Debug, PartialEq, Clone)]
pub enum Line {
    Def(CVec),
    Var(String, CVec),
    Param(Vec<String>),
    Card {
        num: usize,
        name: String,
        params: Vec<CData>,
        parent: Option<String>,
        data: CVec,
    },
}

pub struct LineParser<'a> {
    tk: CardTokenizer<'a>,
    vars: BTreeMap<String, CData>,
    peek: Option<Token<'a, CardToken>>,
}

impl<'a> LineParser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            tk: CardTokenizer::new(s),
            vars: BTreeMap::new(),
            peek: None,
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

    pub fn peek_token(&mut self) -> TokenRes<'a, CardToken> {
        if self.peek.is_none() {
            self.peek = self.tk.next()?;
        }
        Ok(self.peek.clone())
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

    pub fn next(&mut self) -> anyhow::Result<Line> {
        self.breaks();
        match self.peek_token()? {}
    }
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
