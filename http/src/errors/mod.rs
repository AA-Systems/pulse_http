#[derive(Debug)]
pub enum ParseError {
    InvalidUtf8,
    MissingRequestLine,
    InvalidRequestLine,
    UnsupportedMethod,
    InvalidHeaderLine,
    EmptyHeaderName,
}

#[derive(Debug)]
pub enum ReadHeadersError {
    Closed,
    BadRequest,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FormError {
    InvalidUtf8,
    InvalidPercentEncoding,
}
