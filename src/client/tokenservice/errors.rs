use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum TokenServiceError {
    Client(String),
    Server(String),
    Connection(String),
    Parse(String),
    Other(String),
}

impl fmt::Display for TokenServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenServiceError::Client(ref msg) => write!(f, "Client error: {}", msg),
            TokenServiceError::Server(ref msg) => write!(f, "Server error: {}", msg),
            TokenServiceError::Connection(ref msg) => write!(f, "Connection error: {}", msg),
            TokenServiceError::Parse(ref msg) => write!(f, "Parse error: {}", msg),
            TokenServiceError::Other(ref msg) => write!(f, "Other error {}", msg),
        }
    }
}

impl Error for TokenServiceError {
    fn description(&self) -> &str {
        match *self {
            TokenServiceError::Client(_) => "the request to the token service was invalid",
            TokenServiceError::Server(_) => "the token service returned an error",
            TokenServiceError::Connection(_) => "the connection broke",
            TokenServiceError::Parse(_) => "the response from the token service couldn't be parsed",
            TokenServiceError::Other(_) => "something unexpected happened",
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl From<TokenServiceError> for ::client::error::ErrorKind {
    fn from(what: TokenServiceError) -> ::client::error::ErrorKind {
        ::client::error::ErrorKind::TokenService(what)
    }
}