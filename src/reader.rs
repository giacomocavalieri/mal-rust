use std::iter::Peekable;
use std::str::Chars;

#[derive(Clone, Debug)]
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
    Word(String),
    StringLiteral(String),
}

#[derive(Debug)]
pub enum TokenizerError {
    EndOfInput,
    MissingStringQuote(String),
}

pub struct Tokenizer<I: Iterator<Item = char>> {
    chars: Peekable<I>,
    peeked: Option<Token>,
}

// Oh god what am I doing
macro_rules! token {
    ($self: expr, $token: expr) => {{
        $self.chars.next();
        Ok($token)
    }};
}

const RESERVED_CHARS: [char; 13] = [
    '(', ')', '[', ']', '{', '}', '\'', '@', '^', '`', '"', '~', ';',
];

fn is_word_char(char: &char) -> bool {
    !(RESERVED_CHARS.contains(char) || char.is_whitespace())
}

impl Tokenizer<Chars<'_>> {
    pub fn new(input: &String) -> Tokenizer<Chars<'_>> {
        Tokenizer {
            chars: input.chars().into_iter().peekable(),
            peeked: None,
        }
    }

    pub fn next(&mut self) -> Result<Token, TokenizerError> {
        let mut token = Err(TokenizerError::EndOfInput);
        self.consume_whitespace();
        if let Some(char) = self.chars.peek() {
            token = match char {
                '(' => token!(self, Token::OpenParen),
                ')' => token!(self, Token::CloseParen),
                '[' => token!(self, Token::OpenSquare),
                ']' => token!(self, Token::CloseSquare),
                '{' => token!(self, Token::OpenCurly),
                '}' => token!(self, Token::CloseCurly),
                '\'' => token!(self, Token::Quote),
                '@' => token!(self, Token::Deref),
                '^' => token!(self, Token::Metadata),
                '`' => token!(self, Token::SyntaxQuote),
                '"' => todo!("tokenize string"),
                '~' => {
                    self.chars.next();
                    match self.chars.peek() {
                        Some('@') => token!(self, Token::UnquoteSplice),
                        _ => Ok(Token::Unquote),
                    }
                }
                ';' => {
                    self.consume_line();
                    self.next()
                }
                _ => {
                    let word = self.consume_while(is_word_char);
                    self.consume_whitespace();
                    Ok(Token::Word(word))
                }
            };
        }
        token
    }

    pub fn peek(&mut self) -> Result<Token, TokenizerError> {
        if self.peeked.is_none() {
            let next = self.next()?;
            self.peeked = Some(next.clone());
            Ok(next)
        } else {
            // Oh god maybe this is not the best way
            Ok(self.peeked.as_ref().unwrap().clone())
        }
    }

    pub fn rest(&mut self) -> String {
        self.chars.by_ref().collect()
    }

    pub fn consume_whitespace(&mut self) {
        self.consume_while(|char| char.is_whitespace());
    }

    pub fn consume_line(&mut self) -> String {
        self.consume_until(|char| *char == '\n')
    }

    pub fn consume_while<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
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

    pub fn consume_until<F: Fn(&char) -> bool>(&mut self, predicate: F) -> String {
        let mut consumed_chars = vec![];
        while let Some(char) = self.chars.next() {
            consumed_chars.push(char);
            if predicate(&char) {
                break;
            }
        }
        consumed_chars.into_iter().collect()
    }
}
