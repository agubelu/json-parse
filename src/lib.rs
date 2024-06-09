mod errors;
mod parser;
mod tokenizer;

pub use errors::ParseError;
pub use parser::JsonElement;

pub use tokenizer::{Tokenizer, JsonToken};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let json = " true \"\\u0061\\u0062\\u0064\\uD834\\uDD1e\"";
        let mut t = Tokenizer::new(json);
        loop {
            let token = t.next_token();
            if matches!(token, Ok(JsonToken::Eof)) { break }
            println!("{token:?}");
        }
    }
}
