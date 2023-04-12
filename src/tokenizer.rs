use std::iter::Peekable;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenSquare,
    CloseSquare,
    OpenCurly,
    CloseCurly,
    UnquoteSplice, // ~@
    Unquote,       // ~
    Quote,         // '
    Deref,         // @
    Metadata,      // ^
    SyntaxQuote,   // `
    Comment(String),
    Word(String),
    StringLiteral(String),
}

#[derive(Debug)]
pub enum TokenizerError {
    MissingStringQuote,
    UnrecognizedChar(char),
}

pub struct Tokenizer<'a> {
    chars: Peekable<&'a mut dyn Iterator<Item = char>>,
}

const RESERVED_CHARS: [char; 13] = [
    '(', ')', '[', ']', '{', '}', '\'', '@', '^', '`', '"', '~', ';',
];

fn is_word_char(char: &char) -> bool {
    !(RESERVED_CHARS.contains(char) || char.is_whitespace())
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<Token, TokenizerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();
        match self.chars.peek() {
            None => None,
            Some(char) if is_word_char(char) => Some(Ok(self.unsafe_tokenize_word())),
            Some(_) => Some(self.unsafe_tokenize_special()),
        }
    }
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a mut dyn Iterator<Item = char>) -> Tokenizer {
        Tokenizer {
            chars: input.peekable(),
        }
    }

    // Tokenizes a word, it expects at least one word char in the chars stream
    fn unsafe_tokenize_word(&mut self) -> Token {
        let word = self.consume_while(is_word_char);
        Token::Word(word)
    }

    // Tokenizes a special token, it expects at least one char in the chars stream
    fn unsafe_tokenize_special(&mut self) -> Result<Token, TokenizerError> {
        let char = self.chars.next().expect("at least one char");
        match char {
            '(' => Ok(Token::OpenParen),
            ')' => Ok(Token::CloseParen),
            '[' => Ok(Token::OpenSquare),
            ']' => Ok(Token::CloseSquare),
            '{' => Ok(Token::OpenCurly),
            '}' => Ok(Token::CloseCurly),
            '\'' => Ok(Token::Quote),
            '@' => Ok(Token::Deref),
            '^' => Ok(Token::Metadata),
            '`' => Ok(Token::SyntaxQuote),
            '"' => self.tokenize_string(),
            '~' => Ok(self.tokenize_after_tilde()),
            ';' => Ok(Token::Comment(self.consume_line())),
            _ => Err(TokenizerError::UnrecognizedChar(char)),
        }
    }

    fn tokenize_after_tilde(&mut self) -> Token {
        match self.chars.peek() {
            Some('@') => {
                self.chars.next();
                Token::UnquoteSplice
            }
            _ => Token::Unquote,
        }
    }

    fn tokenize_string(&mut self) -> Result<Token, TokenizerError> {
        let mut string_chars = vec![];
        let mut escaping = false;

        loop {
            let char = match self.chars.next() {
                Some(char) => char,
                None => return Err(TokenizerError::MissingStringQuote),
            };
            if char == '"' && !escaping {
                break;
            }
            string_chars.push(char);
            escaping = char == '\\';
        }
        Ok(Token::StringLiteral(string_chars.into_iter().collect()))
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|char| char.is_whitespace());
    }

    fn consume_line(&mut self) -> String {
        self.consume_until(|char| *char != '\n')
    }

    fn consume_while<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
        let mut consumed_chars = vec![];
        loop {
            let char = match self.chars.peek() {
                Some(char) => *char,
                None => break,
            };
            if predicate(&char) {
                consumed_chars.push(char);
                self.chars.next();
            } else {
                break;
            }
        }
        consumed_chars.into_iter().collect()
    }

    fn consume_until<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
        let mut consumed_chars = vec![];
        while let Some(char) = self.chars.next() {
            if predicate(&char) {
                consumed_chars.push(char);
            } else {
                break;
            }
        }
        consumed_chars.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::Token;
    use crate::tokenizer::Tokenizer;

    impl Tokenizer<'_> {
        fn unwrap_next(&mut self) -> Token {
            self.next().unwrap().unwrap()
        }
    }

    #[test]
    fn consume_whitespaces() {
        let mut chars = "  \t\nfoo\n \n\rbar".chars();
        let mut tokenizer = Tokenizer::new(&mut chars);
        tokenizer.consume_whitespace();
        assert_eq!(tokenizer.unwrap_next(), Token::Word("foo".to_string()));
        tokenizer.consume_whitespace();
        assert_eq!(tokenizer.unwrap_next(), Token::Word("bar".to_string()))
    }

    #[test]
    fn consume_line() {
        let mut chars = "foo\nbar".chars();
        let mut tokenizer = Tokenizer::new(&mut chars);
        assert_eq!(tokenizer.consume_line(), "foo");
        assert_eq!(tokenizer.consume_line(), "bar");
    }
}
