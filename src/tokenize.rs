use tokenate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum CardToken {
    Dots(usize),
    KwParam,
    KwConst,
    KwDef,
    Colon,
    Comma,
    Star,
    Minus,
    SquareOpen,
    SquareClose,
    WiggleOpen,
    WiggleClose,
    Break,
    DollarVar(String),
    Text(String),
    Number(isize),
}

impl CardToken {
    pub fn as_text(&self) -> Option<String> {
        match self {
            Self::Text(t) => Some(t.clone()),
            _ => None,
        }
    }
    pub fn as_number(&self) -> Option<isize> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_dots(&self) -> Option<usize> {
        match self {
            Self::Dots(n) => Some(*n),
            _ => None,
        }
    }

    pub fn eq_option(&self, v: &Self) -> Option<()> {
        match self == v {
            true => Some(()),
            false => None,
        }
    }
}

fn num_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

impl CardToken {
    pub fn keyword(s: &str) -> Option<CardToken> {
        match s {
            "def" => Some(CardToken::KwDef),
            "param" => Some(CardToken::KwParam),
            "const" => Some(CardToken::KwConst),
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

    pub fn peek_pos(&mut self) -> Pos {
        self.tk.peek_pos()
    }

    pub fn qoth(&mut self) -> TokenRes<'a, CardToken> {
        self.tk.start_token();
        self.tk.unpeek();
        let mut s = String::new();
        loop {
            match self.tk.next() {
                Some((_, '"')) => return self.tk.token_res(CardToken::Text(s), true),
                Some((_, c)) => s.push(c),
                None => return self.tk.expected("String to end".to_string()),
            }
        }
    }

    pub fn next(&mut self) -> TokenRes<'a, CardToken> {
        self.tk.skip(" \t\r");
        let pc = match self.tk.peek_char() {
            None => return Ok(None),
            Some(c) => c,
        };
        match pc {
            ':' => self.tk.token_res(CardToken::Colon, true),
            '[' => self.tk.token_res(CardToken::SquareOpen, true),
            ']' => self.tk.token_res(CardToken::SquareClose, true),
            '{' => self.tk.token_res(CardToken::WiggleOpen, true),
            '}' => self.tk.token_res(CardToken::WiggleClose, true),
            '*' => self.tk.token_res(CardToken::Star, true),
            '-' => self.tk.token_res(CardToken::Minus, true),
            ',' => self.tk.token_res(CardToken::Comma, true),
            '\n' | ';' => self.tk.token_res(CardToken::Break, true),
            '.' => self
                .tk
                .take_while(|c| c == '.', |s| Ok(CardToken::Dots(s.len()))),
            '$' => {
                self.tk.unpeek();
                self.tk.take_while(char::is_alphabetic, |s| {
                    Ok(CardToken::DollarVar(s.to_string()))
                })
            }

            '@' => {
                self.tk.unpeek();
                self.tk.take_while(char::is_alphabetic, |s| {
                    CardToken::keyword(s).ok_or("Keyword".to_string())
                })
            }
            '#' => {
                let _ = self
                    .tk
                    .take_while(|c| !";\n".contains(c), |_| Ok(CardToken::Break))?;
                self.tk.consume_as("\n;", CardToken::Break).or(Ok(None))
            }
            '"' => self.qoth(),
            c if c.is_alphabetic() => self
                .tk
                .take_while(char::is_alphabetic, |s| Ok(CardToken::Text(s.to_string()))),
            c if num_digit(c) => self.tk.take_while(num_digit, |s| {
                Ok(CardToken::Number(
                    s.parse().map_err(|_| "Could not make number".to_string())?,
                ))
            }),

            _ => self.tk.expected("Something else".to_string()),
        }
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;
    #[test]
    pub fn test_keywords() {
        let s = "@param @const @poo";
        let mut tk = CardTokenizer::new(s);
        let nx = tk.next().unwrap().unwrap();
        assert_eq!(nx.value, CardToken::KwParam);
        let nx = tk.next().unwrap().unwrap();
        assert_eq!(nx.value, CardToken::KwConst);
        assert!(tk.next().is_err());
    }
}
