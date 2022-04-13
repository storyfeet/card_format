use crate::tokenize::CardToken;
use thiserror::*;
use tokenate::{Pos, TErr, Token};

pub type CardRes<T> = Result<T, CardErr>;
#[derive(Clone, Debug)]
pub struct GotToken {
    pos: Pos,
    v: CardToken,
}

pub fn expected<T>(exp: &'static str, tk: &Token<CardToken>) -> Result<T, CardErr> {
    Err(CardErr::Expected(exp).got(tk))
}

#[derive(Debug, Error)]
pub enum CardErr {
    #[error("File Error")]
    FileErr,
    #[error("Expected {}",.0)]
    Expected(&'static str),
    #[error("{}",.0)]
    TokenErr(TErr),
    #[error("{} at {:?}" ,.0,.1)]
    At(Box<CardErr>, Pos),
    #[error("{}, got EOF",.0)]
    EOF(Box<CardErr>),
    #[error("{}, got {:?}",.0,.1)]
    Got(Box<CardErr>, GotToken),
}

impl CardErr {
    pub fn at(self, pos: Pos) -> Self {
        Self::At(Box::new(self), pos)
    }
    pub fn got(self, t: &Token<CardToken>) -> Self {
        Self::Got(
            Box::new(self),
            GotToken {
                pos: t.start,
                v: t.value.clone(),
            },
        )
    }
}

impl From<tokenate::TErr> for CardErr {
    fn from(e: TErr) -> Self {
        Self::TokenErr(e)
    }
}
