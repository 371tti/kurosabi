use std::fmt;
use std::error::Error;

use crate::http::code::HttpStatusCode;

#[derive(Debug)]
pub enum RouterError {
    HttpErrorCode(HttpStatusCode),
    HttpErrorCodeWithMessage(HttpStatusCode, String),
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::HttpErrorCode(code) => {
                write!(f, "RouterError: HTTP Error Code {}", *code as u16)
            }
            RouterError::HttpErrorCodeWithMessage(code, msg) => {
                write!(f, "RouterError: HTTP Error Code {} - {}", *code as u16, msg)
            }
        }
    }
}

impl Error for RouterError {}

