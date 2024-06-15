mod data;
mod parser;
mod scanner;

pub use data::{JsonElement, ParseError};

pub fn parse(json: &str) -> Result<JsonElement, ParseError> {
    parser::JsonParser::from(json).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = "\"bad\\b\\b\\bgood\"";
        let res = parse(json);
        println!("{:?}", res);
        if let Err(ParseError { msg, .. }) = &res {
            println!("{msg}");
        }

        if let Ok(JsonElement::String(s)) = &res {
            println!("{s}");
        }
    }
}
