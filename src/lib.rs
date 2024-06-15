//! A low-level JSON parser with full spec support and a simple API.
mod data;
mod parser;
mod scanner;
mod tests;

pub use data::{JsonElement, ParseError};

/// Parses a JSON string into a [JsonElement], or returns a [ParseError].
pub fn parse(json: &str) -> Result<JsonElement, ParseError> {
    parser::JsonParser::from(json).parse()
}
