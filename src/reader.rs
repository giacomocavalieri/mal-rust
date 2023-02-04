use regex::Regex;

struct Reader {
    tokens: Vec<String>,
    current_position: usize,
}

enum ReaderError {
    NoMoreTokens,
}

impl Reader {
    fn peek(&self) -> Result<&String, ReaderError> {
        self.tokens
            .get(self.current_position)
            .ok_or(ReaderError::NoMoreTokens)
    }

    fn next(&mut self) -> Result<&String, ReaderError> {
        self.current_position += 1;
        self.tokens
            .get(self.current_position - 1)
            .ok_or(ReaderError::NoMoreTokens)
    }
}

fn tokenize(string: &str) -> Vec<String> {
    // There is a way to avoid recompiling the regex every time, I don't care for now
    let regex =
        Regex::new(r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"###)
            .unwrap();

    let mut tokens = vec![];
    for capture in regex.captures_iter(string) {
        let token = &capture[1];
        if !token.starts_with(";") {
            // Maybe I should return the string reference? Does it really matter?
            tokens.push(String::from(token));
        }
    }
    tokens
}
