#[derive(Debug)]
pub enum ParseError {
    InvalidUtf8,
    MissingRequestLine,
    InvalidRequestLine,
    UnsupportedMethod,
    InvalidHeaderLine,
    EmptyHeaderName,
}
