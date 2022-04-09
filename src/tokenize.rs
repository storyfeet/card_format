use tokenate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum CardToken {
    Dot,
    DotDot,
    KwParam,
    KwVar,
    KwDef,
    Dollar,
    Colon,
    Star,
    SquareOpen,
    SquareClose,
    WiggleOpen,
    WiggleClose,
    Break,
    Ident(String),
    Number(i64),
}

impl CardToken {
    pub fn keyword(s: &str) -> Option<CardToken> {
        match s {
            "def" => Some(CardToken::KwDef),
            "param" => Some(CardToken::KwParam),
            "var" => Some(CardToken::KwVar),
            _ => None,
        }
    }
}
pub struct CardTokenizer<'a> {
    tk: InnerTokenizer<'a>,
}

impl<'a> CardTokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            tk: InnerTokenizer::new(s),
        }
    }

    pub fn next(&mut self) -> TokenRes<CardToken> {
        self.tk.skip(" \t\r");
        let pc = match self.tk.peek_char() {
            None => return Ok(None),
            Some(c) => c,
        };
        match pc {
            '[' => self.tk.token_res(CardToken::SquareOpen, true),
            ']' => self.tk.token_res(CardToken::SquareClose, true),
            '{' => self.tk.token_res(CardToken::WiggleOpen, true),
            '}' => self.tk.token_res(CardToken::WiggleClose, true),
            '*' => self.tk.token_res(CardToken::Star, true),
            '$' => self.tk.token_res(CardToken::Dollar, true),
            '.' => self.tk.follow_or('.', CardToken::DotDot, CardToken::Dot),
            '\n' | ';' => self.tk.token_res(CardToken::Break, true),
            '@' => {
                self.tk.unpeek();
                self.tk.take_while(char::is_alphabetic, |s| {
                    CardToken::keyword(s).ok_or("Keyword".to_string())
                })
            }
            c if c.is_alphabetic() => self
                .tk
                .take_while(char::is_alphabetic, |s| Ok(CardToken::Ident(s.to_string()))),

            _ => self.tk.expected("Something else".to_string()),
        }
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;
    #[test]
    pub fn test_keywords() {
        let s = "@param @var @poo";
        let mut tk = CardTokenizer::new(s);
        let nx = tk.next().unwrap().unwrap();
        assert_eq!(nx.value, CardToken::KwParam);
        let nx = tk.next().unwrap().unwrap();
        assert_eq!(nx.value, CardToken::KwVar);
        assert!(tk.next().is_err());
    }
}
