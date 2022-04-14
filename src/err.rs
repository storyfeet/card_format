use crate::tokenize::CardToken;
use thiserror::*;
use tokenate::{Pos, TErr, Token};

pub type CardRes<T> = Result<T, AtErr>;
#[derive(Clone, Debug)]
pub struct GotToken {
    pos: Pos,
    v: CardToken,
}

pub fn expected<T>(exp: &'static str, tk: &Token<CardToken>) -> Result<T, AtErr> {
    Err(CardErr::Expected(exp).got(tk))
}

#[derive(Debug, Error)]
pub enum CardErr {
    #[error("{}",.0)]
    S(&'static str),
    #[error("Expected {}",.0)]
    Expected(&'static str),
    #[error("Cannot set Property")]
    Unset,
    #[error("{}",.0)]
    TokenErr(TErr),
}

impl CardErr {
    pub fn at(self, pos: Pos) -> AtErr {
        AtErr::At(self, pos)
    }
    pub fn got(self, t: &Token<CardToken>) -> AtErr {
        AtErr::Got(
            self,
            GotToken {
                pos: t.start,
                v: t.value.clone(),
            },
        )
    }
    pub fn eof(self) -> AtErr {
        AtErr::EOF(self)
    }
}

#[derive(Debug, Error)]
pub enum AtErr {
    #[error("{} at {:?}" ,.0,.1)]
    At(CardErr, Pos),
    #[error("{}, got EOF",.0)]
    EOF(CardErr),
    #[error("{}, got {:?}",.0,.1)]
    Got(CardErr, GotToken),
    #[error("{}",.0)]
    TokenError(TErr),

    #[error("File Error")]
    FileErr,
}

impl From<tokenate::TErr> for AtErr {
    fn from(e: TErr) -> Self {
        Self::TokenError(e)
    }
}
