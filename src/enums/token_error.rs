#[derive(Debug)]
pub enum TokenError {
    ParseError,
    NotFound,
    NoTokenProvided,
}
