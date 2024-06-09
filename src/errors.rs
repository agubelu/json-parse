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