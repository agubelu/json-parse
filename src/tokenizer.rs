use crate::data::{JsonToken, ParseError, TokenKind, TokenPosition};
use std::cmp::min;

pub struct Tokenizer<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    position: TokenPosition,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        let position = TokenPosition { line: 1, column: 0 };
        Self {
            source,
            start: 0,
            current: 0,
            position,
        }
    }

    pub fn next_token(&mut self) -> Result<JsonToken, ParseError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        self.start = self.current;
        match self.consume() {
            "{" => self.make_token(TokenKind::LeftBrace),
            "}" => self.make_token(TokenKind::RightBrace),
            "[" => self.make_token(TokenKind::LeftBracket),
            "]" => self.make_token(TokenKind::RightBracket),
            "," => self.make_token(TokenKind::Comma),
            ":" => self.make_token(TokenKind::Colon),
            "\"" => self.make_string(),
            x if is_letter(x) => self.make_keyword(),
            x if is_number_start(x) => self.make_number(),
            x => {
                let msg = format!("Unexpected character: '{x}'");
                self.make_error(msg)
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // String scanning

    fn make_string(&mut self) -> Result<JsonToken, ParseError> {
        let mut string = String::new();

        while !self.matches("\"") {
            if self.is_at_end() {
                return self.make_error("Unterminated string");
            }

            match self.consume() {
                "\\" => string.push_str(&self.parse_escape()?),
                x if is_forbidden_char(x) => {
                    let msg = string_error_msg(x);
                    return self.make_error(msg);
                }
                x => string.push_str(x),
            }
        }

        self.make_token(TokenKind::String(string))
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
            x => {
                let msg = format!("Invalid escape sequence: \\{x}");
                self.make_error(msg)
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<String, ParseError> {
        // The unicode prefix has been consumed, parse the remaining sequence
        let code = self.parse_u16_encoded()?;

        // If this is part of a 32-bit surrogate sequence, we need to parse the second part
        if is_high_surrogate(code) {
            let error_msg = || {
                format!(
                    "The Unicode sequence '{code:04X}' represents an unfinished character. {}",
                    "A follow-up Unicode escape sequence was expected but not found."
                )
            };
            if !self.matches("\\") {
                return self.make_error(error_msg());
            }

            if !self.matches("u") {
                return self.make_error(error_msg());
            }

            let code2 = self.parse_u16_encoded()?;
            String::from_utf16(&[code, code2]).or_else(|_| {
                self.make_error(format!(
                    "Invalid unicode character: \\u{code:04X}\\u{code2:04X}"
                ))
            })
        } else {
            // Otherwise just turn it into a unicode point and return it if it's valid
            String::from_utf16(&[code])
                .or_else(|_| self.make_error(format!("Invalid unicode character: \\u{code:04X}")))
        }
    }

    fn parse_u16_encoded(&mut self) -> Result<u16, ParseError> {
        /* Parses the u16 represented by a single unicode escape sequence \uXXXX
         * It should be called when the scanner is at the beggining of the hex code to be scanned.
         * Returns an Err if the sequence is not a 4-character hex sequence. */
        let start = self.current;
        for _ in 0..4 {
            self.advance()
        }
        let max = self.source.len(); // Be careful not to panic by overstepping our slice's boundaries
        let seq = &self.source[min(max, start)..min(max, self.current)];

        if !is_hex(seq) {
            self.make_error(format!(
                "Invalid Unicode escape sequence: '{seq}' (should be a 4-character hex code)"
            ))
        } else {
            Ok(u16::from_str_radix(seq, 16).unwrap()) // seq is a valid 16-bit hex sequence
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Number scanning

    fn make_number(&mut self) -> Result<JsonToken, ParseError> {
        self.scan_integer()?;
        self.scan_fraction()?;
        self.scan_exponent()?;
        // At this point, the format is guaranteed to match the JSON spec
        // (save for the slight deviation above), but this format is also accepted by
        // Rust's str-to-f64 conversion in all cases, so we can safely parse and unwrap it.
        // https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64
        let s = &self.source[self.start..self.current];
        self.make_token(TokenKind::Number(s.parse().unwrap()))
    }

    fn scan_integer(&mut self) -> Result<(), ParseError> {
        // If the number started with a minus sign, demand that at least one digit is present
        if self.peek_behind() == "-" && !is_number(self.consume()) {
            return self.make_error("At least a digit was expected after '-'");
        }
        // Skip all follow-up digits to scan the integer part.
        // This violates the official spec which forbids leading zeroes,
        // but it's both simpler to implement and more flexible towards users.
        self.skip_digits();
        Ok(())
    }

    fn scan_fraction(&mut self) -> Result<(), ParseError> {
        /* Scans an optional fraction part, consisting of a dot and at least one digit. */
        if self.matches(".") {
            if !is_number(self.consume()) {
                return self.make_error("At least a digit is expected after a fraction dot");
            }
            self.skip_digits();
        }

        Ok(())
    }

    fn scan_exponent(&mut self) -> Result<(), ParseError> {
        /* Scans an optional exponent part, consisting of 'e|E', an optional sign,
         * and at least one digit. */
        if matches!(self.peek(), "e" | "E") {
            // Consume the exponent
            self.advance();
            // Consume the sign if present
            if matches!(self.peek(), "-" | "+") {
                self.advance()
            }
            // Expect one digit and consume the rest
            if !is_number(self.consume()) {
                return self.make_error("At least a digit is expected after an exponent");
            }
            self.skip_digits();
        }

        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Other

    fn make_keyword(&mut self) -> Result<JsonToken, ParseError> {
        while is_letter(self.peek()) {
            self.advance();
        }

        match &self.source[self.start..self.current] {
            "null" => self.make_token(TokenKind::Null),
            "true" => self.make_token(TokenKind::True),
            "false" => self.make_token(TokenKind::False),
            x => self.make_error(format!("Unknown (case-sensitive) keyword {x}")),
        }
    }

    fn make_token<T>(&self, kind: TokenKind) -> Result<JsonToken, T> {
        /* Creates a JsonToken at the current start position */
        // JSON tokens can't spawn multiple lines so we can deduce its start position
        let pos = TokenPosition {
            column: self.position.column - (self.current - self.start),
            line: self.position.line,
        };
        Ok(JsonToken { kind, pos })
    }

    fn make_error<T, S: Into<String>>(&self, msg: S) -> Result<T, ParseError> {
        /* Creates a ParseError in the current position */
        Err(ParseError::new(
            msg.into(),
            self.position.line,
            self.position.column,
        ))
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Scanning control

    fn advance(&mut self) {
        self.current += 1;
        self.position.column += 1;
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
        if matched {
            self.advance()
        }
        matched
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                "\n" => {
                    self.position.line += 1;
                    self.position.column = 0;
                }
                " " | "\r" | "\t" => {}
                _ => return,
            }
            self.advance();
        }
    }

    fn skip_digits(&mut self) {
        /* Advances the scanner forward until a non-number is found */
        while is_number(self.peek()) {
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

fn is_number_start(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('0'..='9') | Some('-')) && chars.next().is_none()
}

fn is_number(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('0'..='9')) && chars.next().is_none()
}

fn is_hex(s: &str) -> bool {
    s.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_forbidden_char(x: &str) -> bool {
    // Forbidden string characters: " / and everything under U+0020
    matches!(x, "\\" | "\"") || x.encode_utf16().next().unwrap_or(0) < 0x0020
}

fn string_error_msg(ch: &str) -> String {
    'a'.into() // TODO change
}

fn is_high_surrogate(x: u16) -> bool {
    (0xD800..=0xDBFF).contains(&x)
}
