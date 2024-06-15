/* Data models */

use std::fmt::Display;

pub type KeyValue = (String, JsonElement);

#[derive(Clone, Debug)]
pub enum JsonElement {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonElement>),
    Object(Vec<KeyValue>),
}

#[derive(Debug, Clone)]
pub struct JsonToken {
    pub kind: TokenKind,
    pub pos: TokenPosition,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
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
    Eof,
}

#[derive(Debug, Clone, Copy)]
pub struct TokenPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub msg: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn new(msg: String, line: usize, column: usize) -> Self {
        Self { msg, line, column }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::LeftBrace => f.write_str("'{'"),
            TokenKind::RightBrace => f.write_str("'}'"),
            TokenKind::LeftBracket => f.write_str("'['"),
            TokenKind::RightBracket => f.write_str("']'"),
            TokenKind::Comma => f.write_str("','"),
            TokenKind::Colon => f.write_str("':'"),
            TokenKind::True => f.write_str("boolean (true)"),
            TokenKind::False => f.write_str("boolean (false)"),
            TokenKind::Null => f.write_str("null"),
            TokenKind::Number(n) => f.write_str(&format!("number ({n})")),
            TokenKind::String(s) => f.write_str(&format!("string (\"{s}\")")),
            TokenKind::Eof => f.write_str("end-of-file"),
        }
    }
}

impl JsonToken {
    pub fn dummy() -> Self {
        let pos = TokenPosition { column: 0, line: 0 };
        let kind = TokenKind::Null;
        Self { pos, kind }
    }

    pub fn get_string(self) -> String {
        /* Consumes a String-kind token to return the String inside it.
        Will panic if called on a non-string token. */
        match self.kind {
            TokenKind::String(s) => s,
            _ => panic!("Tried to extract a string from an invalid token"),
        }
    }
}

impl PartialEq for TokenKind {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
