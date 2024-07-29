use crate::data::{JsonToken, ParseError, TokenKind, TokenPosition};
use std::cmp::min;
use std::iter::Peekable;
use std::str::Chars;

pub struct Scanner<'a> {
    // The original string, which is directly sliced to parse things like keywords and numbers.
    // From it, we derive a per-character iterator, because iterating directly over
    // the indices of a UTF-8 encoded &str can land us midway through multi-byte characters.
    source: &'a str,
    char_iter: Peekable<Chars<'a>>,
    // Remember the last consumed character, to support peeking backwards in the scanning process.
    prev_char: char,
    // These indices are used to address the original string. Note that they are byte indices,
    // and not character indices. This means that every `char` from `char_iter` can advance
    // `current` anywhere between 1 and 4 positions, depending on how many bytes the
    // char requires to be represented in UTF-8.
    start: usize,
    current: usize,
    // 2-dimensional (line, column) position of a token. If the token is multi-character,
    // it points to the position of the starting character. Line is 1-based and column is 0-based.
    // This info is user-facing so, differently from the indices above, these positions are
    // per-character instead of per-byte.
    position: TokenPosition,
    // Aux data to remember where the current token started. Since (current - start) is a number
    // of bytes, which doesn't necessarily equal to the number of characters advanced, storing
    // the initial position is simpler and quicker than doing the match backwards to find out
    // how many characters we advanced.
    start_position: TokenPosition,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            char_iter: source.chars().peekable(),
            prev_char: '\0',
            start: 0,
            current: 0,
            position: TokenPosition::default(),
            start_position: TokenPosition::default(),
        }
    }

    pub fn next_token(&mut self) -> Result<JsonToken, ParseError> {
        self.skip_whitespace();
        self.start = self.current;
        self.start_position = self.position;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        match self.consume() {
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            '[' => self.make_token(TokenKind::LeftBracket),
            ']' => self.make_token(TokenKind::RightBracket),
            ',' => self.make_token(TokenKind::Comma),
            ':' => self.make_token(TokenKind::Colon),
            '"' => self.make_string(),
            x if is_letter(x) => self.make_keyword(),
            x if is_number_start(x) => self.make_number(),
            x => {
                let msg = format!("Unexpected character: '{x}'");
                self.make_error_behind(msg)
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // String scanning

    fn make_string(&mut self) -> Result<JsonToken, ParseError> {
        let mut string = String::new();

        while !self.matches('"') {
            if self.is_at_end() {
                return self.make_error_behind("Unterminated string");
            }

            match self.consume() {
                '\\' => string.push(self.parse_escape()?),
                x if is_forbidden_char(x) => {
                    let msg = string_error_msg(x);
                    return self.make_error_behind(msg);
                }
                x => string.push(x),
            }
        }

        self.make_token(TokenKind::String(string))
    }

    fn parse_escape(&mut self) -> Result<char, ParseError> {
        match self.consume() {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'b' => Ok('\x08'),
            'f' => Ok('\x0C'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => self.parse_unicode_escape(),
            x => {
                let msg = if x == ' ' {
                    "A lone \\ is not allowed inside a string (hint: you can escape it with \\\\)"
                        .into()
                } else {
                    format!("Invalid escape sequence: \\{x}")
                };
                self.make_error_behind(msg)
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, ParseError> {
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
            if !self.matches('\\') {
                return self.make_error_here(error_msg());
            }

            if !self.matches('u') {
                return self.make_error_here(error_msg());
            }

            let code2 = self.parse_u16_encoded()?;
            char::decode_utf16([code, code2])
                .next()
                .unwrap()
                .or_else(|_| {
                    self.make_error_behind(format!(
                        "Invalid unicode character: \\u{code:04X}\\u{code2:04X}"
                    ))
                })
        } else {
            // Otherwise just turn it into a unicode point and return it if it's valid
            char::decode_utf16([code]).next().unwrap().or_else(|_| {
                self.make_error_behind(format!("Invalid unicode character: \\u{code:04X}"))
            })
        }
    }

    fn parse_u16_encoded(&mut self) -> Result<u16, ParseError> {
        /* Parses the u16 represented by a single unicode escape sequence \uXXXX
         * It should be called when the scanner is at the beggining of the hex code to be scanned.
         * Returns an Err if the sequence is not a 4-character hex sequence. */
        let start = self.current;
        for _ in 0..4 {
            self.advance();
        }
        let max = self.source.len(); // Be careful not to panic by overstepping our slice's boundaries
        let seq = &self.source[min(max, start)..min(max, self.current)];

        if !is_hex(seq) {
            self.make_error_behind(format!(
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
        // At this point, the format is guaranteed to match the JSON spec.
        // This format is a subset of Rust's str-to-f64 accepted strings,
        // so we can safely parse and unwrap it.
        // https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64
        let s = &self.source[self.start..self.current];
        self.make_token(TokenKind::Number(s.parse().unwrap()))
    }

    fn scan_integer(&mut self) -> Result<(), ParseError> {
        // If the number started with a minus sign, demand that at least one digit is present
        if self.peek_behind() == '-' && !is_number(self.consume()) {
            return self.make_error_behind("At least a digit is expected after '-'");
        }
        // Skip all follow-up digits to scan the integer part.
        // This violates the official spec which forbids leading zeroes,
        // but it's both simpler to implement and more flexible towards users.
        self.skip_digits();
        Ok(())
    }

    fn scan_fraction(&mut self) -> Result<(), ParseError> {
        /* Scans an optional fraction part, consisting of a dot and at least one digit. */
        if self.matches('.') {
            if !is_number(self.consume()) {
                return self.make_error_behind("At least a digit is expected after a fraction dot");
            }
            self.skip_digits();
        }

        Ok(())
    }

    fn scan_exponent(&mut self) -> Result<(), ParseError> {
        /* Scans an optional exponent part, consisting of 'e|E', an optional sign,
         * and at least one digit. */
        if matches!(self.peek(), 'e' | 'E') {
            // Consume the exponent
            self.advance();
            // Consume the sign if present
            if matches!(self.peek(), '-' | '+') {
                self.advance();
            }
            // Expect one digit and consume the rest
            if !is_number(self.consume()) {
                return self.make_error_behind("At least a digit is expected after an exponent");
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
            "true" => self.make_token(TokenKind::True),
            "false" => self.make_token(TokenKind::False),
            "null" => self.make_token(TokenKind::Null),
            x => {
                let hint = match x.to_lowercase().as_str() {
                    "true" => " (hint: maybe you meant 'true')",
                    "false" => " (hint: maybe you meant 'false')",
                    "null" => " (hint: maybe you meant 'null')",
                    _ => "",
                };
                self.make_error_at_start(format!("Unknown keyword '{x}'{hint}"))
            }
        }
    }

    fn make_token<T>(&self, kind: TokenKind) -> Result<JsonToken, T> {
        /* Creates a JsonToken at the current start position */
        Ok(JsonToken {
            kind,
            pos: self.start_position,
        })
    }

    fn make_error_here<T, S: Into<String>>(&self, msg: S) -> Result<T, ParseError> {
        /* Creates a ParseError at the current character */
        self.make_error_at(msg, self.position.line, self.position.column)
    }

    fn make_error_behind<T, S: Into<String>>(&self, msg: S) -> Result<T, ParseError> {
        /* Creates a ParseError at the previous character */
        self.make_error_at(msg, self.position.line, self.position.column - 1)
    }

    fn make_error_at_start<T>(&self, msg: String) -> Result<T, ParseError> {
        /* Creates a ParseError in the token's starting position */
        let col = self.position.column - (self.current - self.start);
        self.make_error_at(msg, self.position.line, col)
    }

    fn make_error_at<T, S: Into<String>>(
        &self,
        msg: S,
        line: usize,
        column: usize,
    ) -> Result<T, ParseError> {
        /* Creates a ParseError in the current position */
        Err(ParseError::new(msg.into(), line, column))
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Scanning control

    fn advance(&mut self) {
        self.prev_char = self.char_iter.next().unwrap_or('\0');
        self.position.column += 1;
        self.current += self.prev_char.len_utf8();
    }

    fn consume(&mut self) -> char {
        self.advance();
        self.peek_behind()
    }

    fn peek(&mut self) -> char {
        self.char_iter.peek().copied().unwrap_or('\0')
    }

    fn peek_behind(&self) -> char {
        self.prev_char
    }

    fn matches(&mut self, expected: char) -> bool {
        let matched = self.peek() == expected;
        if matched {
            self.advance();
        }
        matched
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                '\n' => {
                    self.advance();
                    self.position.line += 1;
                    self.position.column = 0;
                }
                ' ' | '\r' | '\t' => self.advance(),
                _ => return,
            }
        }
    }

    fn skip_digits(&mut self) {
        /* Advances the scanner forward until a non-number is found */
        while is_number(self.peek()) {
            self.advance();
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.char_iter.peek().is_none()
    }
}

fn is_letter(s: char) -> bool {
    matches!(s, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_number_start(s: char) -> bool {
    matches!(s, '0'..='9' | '-')
}

fn is_number(s: char) -> bool {
    s.is_ascii_digit()
}

fn is_hex(s: &str) -> bool {
    s.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_forbidden_char(x: char) -> bool {
    // Forbidden string characters: " / and everything under U+0020
    matches!(x, '\\' | '"') || x < 0x0020 as char
}

fn string_error_msg(ch: char) -> String {
    // ch must be a control character, because lone \'s are handled by parse_escape(),
    // and misplaced double quotes will cause other kind of trouble.
    match ch {
        '\n' => "Line breaks are not allowed inside a string (hint: you can escape them as \\n)".into(),
        '\t' => "Literal tabs are not allowed inside a string (hint: you can escape them as \\t)".into(),
        '\r' => "Carriage return line breaks are not allowed inside a string (hint: you can escape them as \\r)".into(),
        '\x08' =>  "Backspace control characters are not allowed inside a string (hint: you can escape them as \\b)".into(),
        '\x0C' =>  "Form-feed control characters are not allowed inside a string (hint: you can escape them as \\f)".into(),
        _ => {
            let code = ch as u32;
            let hex = format!("{code:04X}");
            format!("The control character U+{hex} is not allowed inside a string (hint: you can escape it as \\u{hex}")
        }
    }
}

fn is_high_surrogate(x: u16) -> bool {
    (0xD800..=0xDBFF).contains(&x)
}
