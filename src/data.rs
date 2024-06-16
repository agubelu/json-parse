/* Data models */

use std::fmt::Display;

/// A representation of a JSON element.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum JsonElement {
    /// A literal `null` value
    #[default]
    Null,
    /// A boolean value (`true` / `false`)
    Boolean(bool),
    /// A numeric value
    Number(f64),
    /// A string value. Escape characters and sequences have already been parsed in the contained [String].
    String(String),
    /// An array containing any number of other JSON elements.
    Array(Vec<JsonElement>),
    /// A JSON object, consisting of a series of key-value pairs.
    ///
    /// The pairs are represented using a [Vec] and are provided in the same order in which they are
    /// defined in the original source.
    ///
    /// The [String] keys within a [JsonElement::Object] are guaranteed to be unique.
    Object(Vec<(String, JsonElement)>),
}

/// Returned when a JSON string is malformed or contains any errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// User-friendly description of the error.
    pub msg: String,
    /// 1-based index of the line within the source JSON string in which the error occured.
    pub line: usize,
    /// 0-based index of the column within the line where the error occured.
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonToken {
    pub kind: TokenKind,
    pub pos: TokenPosition,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenPosition {
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
    pub const fn dummy() -> Self {
        let pos = TokenPosition { column: 0, line: 0 };
        let kind = TokenKind::Null;
        Self { pos, kind }
    }

    pub const fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        let pos = TokenPosition { line, column };
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

impl TokenKind {
    pub fn same_kind(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        Self { line: 1, column: 0 }
    }
}
