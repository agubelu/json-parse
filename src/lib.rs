mod parser;
mod tokenizer;

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
        let json = " true \"I would like \\n to kill myself :) \"";
        let mut t = Tokenizer::new(json);
        loop {
            let token = t.next_token();
            if matches!(token, Ok(JsonToken::Eof)) { break }
            println!("{token:?}");
        }
    }
}
