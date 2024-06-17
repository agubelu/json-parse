use crate::data::{JsonElement, JsonToken, TokenKind, TokenPosition};
use crate::scanner::Scanner;
use crate::ParseError;

use std::collections::HashSet;
use std::mem::replace;
use std::rc::Rc;

pub struct JsonParser<'a> {
    scanner: Scanner<'a>,
    upcoming: JsonToken,
}

impl<'a> JsonParser<'a> {
    pub fn from(json: &'a str) -> Self {
        // Populate `upcoming` with a dummy token that will be replaced
        Self {
            upcoming: JsonToken::dummy(),
            scanner: Scanner::new(json),
        }
    }

    pub fn parse(mut self) -> Result<JsonElement, ParseError> {
        self.consume()?; // Initialize the token pipeline
        let elem = self.parse_element()?;
        self.expect(TokenKind::Eof)?;
        Ok(elem)
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    fn parse_element(&mut self) -> Result<JsonElement, ParseError> {
        let current = self.consume()?;
        match current.kind {
            TokenKind::LeftBrace => self.parse_object(),
            TokenKind::LeftBracket => self.parse_array(),
            TokenKind::Number(x) => Ok(JsonElement::Number(x)),
            TokenKind::String(x) => Ok(JsonElement::String(x)),
            TokenKind::True => Ok(JsonElement::Boolean(true)),
            TokenKind::False => Ok(JsonElement::Boolean(false)),
            TokenKind::Null => Ok(JsonElement::Null),
            _ => self.unexpected_token_error(&current),
        }
    }

    fn parse_array(&mut self) -> Result<JsonElement, ParseError> {
        // Opening [ has already been consumed
        let mut arr = vec![];

        if !self.matches(TokenKind::RightBracket)? {
            loop {
                arr.push(self.parse_element()?);
                if !self.matches(TokenKind::Comma)? {
                    break;
                }
            }
            // Consume the closing ]
            self.expect(TokenKind::RightBracket)?;
        }

        Ok(JsonElement::Array(arr))
    }

    fn parse_object(&mut self) -> Result<JsonElement, ParseError> {
        // Opening { has already been consumed
        let mut pairs = vec![];
        if !self.matches(TokenKind::RightBrace)? {
            let mut keys = HashSet::new();

            loop {
                let key_token = self.expect_string()?;
                let pos = key_token.pos; // Copy this before consuming the token in case we need to error out

                // Wrap the String key in a Rc so we can share it between the key-value vec and the key hashset,
                // since cloning the Rc is cheaper than cloning the String itself.
                let key = Rc::new(key_token.get_string());

                if keys.contains(&key) {
                    return self.make_error_at(format!("Duplicated object key: \"{key}\""), &pos);
                }

                // Parse the rest of the value
                self.expect(TokenKind::Colon)?;
                let value = self.parse_element()?;

                keys.insert(key.clone());
                pairs.push((key, value));

                if !self.matches(TokenKind::Comma)? {
                    break;
                }
            }
            // Consume the closing }
            self.expect(TokenKind::RightBrace)?;
        }

        // The HashSet with the keys has been dropped here so all Rc<String> should have only one
        // reference left, held in `pairs`. We can unwrap them into the actual Strings now.
        let data = pairs
            .into_iter()
            .map(|(k, v)| (Rc::into_inner(k).unwrap(), v))
            .collect();
        Ok(JsonElement::Object(data))
    }

    fn unexpected_token_error<T>(&self, token: &JsonToken) -> Result<T, ParseError> {
        let msg = format!("Unexpected {}", token.kind);
        self.make_error(msg, token)
    }

    fn make_error<T>(&self, msg: String, token: &JsonToken) -> Result<T, ParseError> {
        self.make_error_at(msg, &token.pos)
    }

    fn make_error_at<T>(&self, msg: String, pos: &TokenPosition) -> Result<T, ParseError> {
        Err(ParseError::new(msg, pos.line, pos.column))
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////

    fn consume(&mut self) -> Result<JsonToken, ParseError> {
        let next = self.scanner.next_token()?;
        Ok(replace(&mut self.upcoming, next))
    }

    fn matches(&mut self, expected: TokenKind) -> Result<bool, ParseError> {
        let matched = self.upcoming.kind == expected;
        if matched {
            self.upcoming = self.scanner.next_token()?;
        }
        Ok(matched)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<JsonToken, ParseError> {
        /* Consumes and returns the current token only if it matches the expected type.
         * If not, returns a ParseError indicating the expected and actual tokens.
         * Only use this method with empty TokenKinds to avoid allocating useless data. */
        if self.upcoming.kind == expected {
            // == for TokenKind is overriden to compare only variant type
            self.consume()
        } else {
            self.make_error(
                format!("Expected {}, found {}", expected, &self.upcoming.kind),
                &self.upcoming,
            )
        }
    }

    fn expect_string(&mut self) -> Result<JsonToken, ParseError> {
        /* Special case of self.expect() to avoid having to allocate a TokenKind::String */
        if matches!(self.upcoming.kind, TokenKind::String(_)) {
            self.consume()
        } else {
            self.make_error(
                format!("Expected string, found {}", &self.upcoming.kind),
                &self.upcoming,
            )
        }
    }
}
