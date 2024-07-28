//! A low-level JSON parser with full spec support and a simple API.
mod data;
mod parser;
mod scanner;
mod tests;

pub use data::{JsonElement, ParseError};

/// Parses a JSON string into a [JsonElement], or returns a [ParseError].
///
/// ```
/// use json_parse::{parse, JsonElement::*};
///
/// let json = "[1, true, null]";
/// let parsed = parse(json).unwrap();
///
/// assert_eq!(parsed, Array(
///    vec![Number(1.0), Boolean(true), Null]
/// ));
/// ```
///
/// ```
/// use json_parse::{parse, ParseError};
/// let bad_json = r#"
///     {
///         "one": 1,
///         2: "two"
///     }
/// "#;
/// let error = parse(bad_json).unwrap_err();
///
/// assert_eq!(error, ParseError{
///     line: 4,
///     column: 8,
///     msg: "Expected string, found number (2)".into()
/// });
/// ```
pub fn parse(json: impl AsRef<str>) -> Result<JsonElement, ParseError> {
    parser::JsonParser::from(json.as_ref()).parse()
}
