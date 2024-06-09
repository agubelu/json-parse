use std::cmp::min;
use crate::ParseError;

#[derive(Debug, Clone)]
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
    line: usize,
    column: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, start: 0, current: 0, line: 1, column: 0 }
    }

    pub fn next_token(&mut self) -> Result<JsonToken, ParseError> {
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
            x => {
                let msg = format!("Unexpected character: '{x}'");
                self.make_error(msg)
            },
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Aux scanners

    fn make_string(&mut self) -> Result<JsonToken, ParseError> {
        let mut string = String::new();

        while !self.matches("\"") {
            if self.is_at_end() {
                return self.make_error("Unterminated string");
            }

            match self.consume() {
                "\\" => string.push_str(&self.parse_escape()?),
                x => string.push_str(x),
            }
        }

        Ok(JsonToken::String(string))
    }

    fn parse_escape(&mut self) -> Result<String, ParseError> {
        match self.consume() {
            "\"" => Ok("\"".to_owned()),
            "\\" => Ok("\\".to_owned()),
            "/" => Ok("/".to_owned()),
            "b" => Ok("\x08".to_owned()),
            "f" => Ok("\x0C".to_owned()),
            "n" => Ok("\n".to_owned()),
            "r" => Ok("\r".to_owned()),
            "t" => Ok("\t".to_owned()),
            "u" => self.parse_unicode_escape(),
             x  => {
                let msg = format!("Invalid escape sequence: \\{x}");
                self.make_error(msg)
            },
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<String, ParseError> {
        // The unicode prefix has been consumed, parse the remaining sequence
        let code = self.parse_u16_encoded()?;

        // If this is part of a 32-bit surrogate sequence, we need to parse the second part
        if is_high_surrogate(code) {
            let error_msg = || format!("The Unicode sequence '{code:04X}' represents an unfinished character. {}",
                                    "A follow-up Unicode escape sequence was expected but not found.");
            if !self.matches("\\") {
                return self.make_error(error_msg());
            }

            if !self.matches("u") {
                return self.make_error(error_msg());
            }

            let code2 = self.parse_u16_encoded()?;
            String::from_utf16(&[code, code2]).or_else(|_|
                self.make_error(format!("Invalid unicode character: \\u{code:04x}\\u{code2:04x}"))
            )
        } else {
            // Otherwise just turn it into a unicode point and return it if it's valid
            String::from_utf16(&[code]).or_else(|_|
                self.make_error(format!("Invalid unicode character: \\u{code:04x}"))
            )
        }
    }

    fn parse_u16_encoded(&mut self) -> Result<u16, ParseError> {
        /* Parses the u16 represented by a single unicode escape sequence \uXXXX
         * It should be called when the scanner is at the beggining of the hex code to be scanned.
         * Returns an Err if the sequence is not a 4-character hex sequence. */
        let start = self.current;
        for _ in 0..4 { self.advance() }
        let max = self.source.len(); // Be careful not to panic by overstepping our slice's boundaries
        let seq = &self.source[min(max, start) .. min(max, self.current)];

        if !is_hex(seq) {
            self.make_error(format!("Invalid Unicode escape sequence: '{seq}' (should be a 4-character hex code)"))
        } else {
            Ok(u16::from_str_radix(seq, 16).unwrap()) // seq is a valid 16-bit hex sequence
        }
    }

    fn make_keyword(&mut self) -> Result<JsonToken, ParseError> {
        while is_letter(self.peek()) {
            self.advance();
        }

        match &self.source[self.start .. self.current] {
            "null" => Ok(JsonToken::Null),
            "true" => Ok(JsonToken::True),
            "false" => Ok(JsonToken::False),
            x => self.make_error(format!("Unknown (case-sensitive) keyword {x}")),
        }
    }

    fn make_number(&mut self) -> Result<JsonToken, ParseError> {
        Ok(JsonToken::Colon)
    }

    fn make_error<T, S: Into<String>>(&self, msg: S) -> Result<T, ParseError> {
        Err(ParseError::new(msg.into(), self.line, self.column))
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Scanning control

    fn advance(&mut self) {
        self.current += 1;
        self.column += 1;
    }

    fn consume(&mut self) -> &str {
        self.advance();
        self.peek_behind()
    }

    fn peek(&self) -> &str {
        let i = min(self.current, self.source.len());
        let j = min(self.current + 1, self.source.len());
        &self.source[i..j]
    }

    fn peek_behind(&self) -> &str {
        let i = min(self.current - 1, self.source.len());
        let j = min(self.current, self.source.len());
        &self.source[i..j]
    }

    fn matches(&mut self, expected: &str) -> bool {
        let matched = self.peek() == expected;
        if matched { self.advance() }
        matched
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                "\n" => {
                    self.line += 1;
                    self.column = 0;
                },
                " " | "\r" | "\t" => {},
                _ => return,
            }
            self.advance();
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

fn is_hex(s: &str) -> bool {
    s.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_high_surrogate(x: u16) -> bool {
    (0xD800..=0xDBFF).contains(&x)
}