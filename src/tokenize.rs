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
    Comma,
    Star,
    SquareOpen,
    SquareClose,
    WiggleOpen,
    WiggleClose,
    Break,
    Text(String),
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
            '[' => self.tk.token_res(CardToken::SquareOpen, true),
            ']' => self.tk.token_res(CardToken::SquareClose, true),
            '{' => self.tk.token_res(CardToken::WiggleOpen, true),
            '}' => self.tk.token_res(CardToken::WiggleClose, true),
            '*' => self.tk.token_res(CardToken::Star, true),
            ',' => self.tk.token_res(CardToken::Comma, true),
            '$' => self.tk.token_res(CardToken::Dollar, true),
            '.' => self.tk.follow_or('.', CardToken::DotDot, CardToken::Dot),
            '\n' | ';' => self.tk.token_res(CardToken::Break, true),
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
