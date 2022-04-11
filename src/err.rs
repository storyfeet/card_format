use crate::tokenize::CardToken;
use thiserror::*;
use tokenate::{Pos, TErr, Token};

#[derive(Clone, Debug)]
pub struct GotToken {
    pos: Pos,
    v: CardToken,
}

pub fn expected<T>(exp: &'static str, tk: &Token<CardToken>) -> Result<T, CardErr> {
    Err(CardErr::Expected(
        exp,
        GotToken {
            pos: tk.start,
            v: tk.value,
        },
    ))
}

#[derive(Debug, Error)]
pub enum CardErr {
    #[error("File Error")]
    FileErr,
    #[error("Error referencing {} from {}", .0, .1)]
    RefErr(String, String),
    #[error("Expected {}, got {:?}",.0,.1)]
    Expected(&'static str, GotToken),
    #[error("{}",.0)]
    TokenErr(TErr),
}

impl From<tokenate::TErr> for CardErr {
    fn from(e: TErr) -> Self {
        Self::TokenErr(e)
    }
}
