mod data;
mod parser;
mod tokenizer;

pub use data::{JsonElement, ParseError};

pub fn parse(json: &str) -> Result<JsonElement, ParseError> {
    parser::JsonParser::from(json).parse()
}

#[cfg(test)]
mod tests {
    use parser::JsonParser;

    use super::*;
    #[test]
    fn it_works() {
        let json = "[true, false, false, \"bad\\b\\b\\bgood\"]";
        let t = JsonParser::from(json);
        println!("{:?}", t.parse());
    }
}
