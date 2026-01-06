use std::fmt;
use std::error::Error;
use std::ops::Range;

use crate::http::code::HttpStatusCode;

#[derive(Debug)]
pub enum RouterError {
    HttpErrorCode(HttpStatusCode),
    HttpErrorCodeWithMessage(HttpStatusCode, String),
    InvalidHttpRequest(Range<usize>, String),
    IoError(std::io::Error),
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
            RouterError::InvalidHttpRequest(range, msg) => {
                write!(f, "RouterError: Invalid HTTP Request at {:?} - {}", range, msg)
            }
            RouterError::IoError(e) => {
                write!(f, "RouterError: IO Error - {}", e)
            }
        }
    }
}

impl Error for RouterError {}

pub type Result<T> = std::result::Result<T, ErrorPare<T>>;
pub struct ErrorPare<T> {
    pub router_error: RouterError,
    pub connection: T,
}

