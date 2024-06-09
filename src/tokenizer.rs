#[derive(Debug)]
pub enum JsonToken {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    True,
    False,
    Null,
    Number(f64),
    String(String),
    Eof
}

pub struct Tokenizer<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, start: 0, current: 0, line: 1}
    }

    pub fn next_token(&mut self) -> Result<JsonToken, String> {
        self.skip_whitespace();
        if self.is_at_end() {
            return Ok(JsonToken::Eof);
        }

        self.start = self.current;
        match self.consume() {
            "{" => Ok(JsonToken::LeftBrace),
            "}" => Ok(JsonToken::RightBrace),
            "[" => Ok(JsonToken::LeftBracket),
            "]" => Ok(JsonToken::RightBracket),
            "," => Ok(JsonToken::Comma),
            ":" => Ok(JsonToken::Colon),
            "\"" => self.make_string(),
            x if is_letter(x) => self.make_keyword(),
            x if is_number(x) => self.make_number(),
            x => Err(format!("Unknown character in line: '{}'", x))
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    fn make_string(&mut self) -> Result<JsonToken, String> {
        let mut string = String::new();

        while !self.matches("\"") {
            if self.is_at_end() {
                return Err("Unterminated string".into());
            }
            match self.consume() {
                "\\" => string.push(self.parse_escape()?),
                x => string.push_str(x),
            }
        }

        Ok(JsonToken::String(string))
    }

    fn parse_escape(&mut self) -> Result<char, String> {
        match self.consume() {
            "\"" => Ok('"'),
            "\\" => Ok('\\'),
            "/" => Ok('/'),
            "b" => Ok('\x08'),
            "f" => Ok('\x0C'),
            "n" => Ok('\n'),
            "r" => Ok('\r'),
            "t" => Ok('\t'),
            "u" => Ok('x'), // TODO
             x  => Err(format!("Invalid escape sequence: \\{x}")),
        }
    }

    fn make_keyword(&mut self) -> Result<JsonToken, String> {
        while !self.is_at_end() && is_letter(self.peek()) {
            self.consume();
        }

        match &self.source[self.start .. self.current] {
            "null" => Ok(JsonToken::Null),
            "true" => Ok(JsonToken::True),
            "false" => Ok(JsonToken::False),
            x => Err(format!("Unknown keyword {}", x)),
        }
    }

    fn make_number(&mut self) -> Result<JsonToken, String> {
        Ok(JsonToken::Colon)
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    fn peek(&self) -> &str {
        &self.source[self.current .. self.current + 1]
    }

    fn matches(&mut self, expected: &str) -> bool {
        let matched = self.peek() == expected;
        if matched { self.consume(); }
        matched
    }

    fn consume(&mut self) -> &str {
        self.current += 1;
        &self.source[self.current - 1 .. self.current]
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                "\n" => self.line += 1,
                " " | "\r" | "\t" => {},
                _ => return,
            }
            self.consume();
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

fn is_letter(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('a'..='z') | Some('A'..='Z') | Some('_')) && chars.next().is_none()
}

fn is_number(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('0'..='9') | Some('-')) && chars.next().is_none()
}
