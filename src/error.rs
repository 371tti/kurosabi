use crate::response::Res;

pub enum KurosabiError {
    InvalidHttpHeader(String),
    InvalidHttpRangeHeader(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for KurosabiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KurosabiError::InvalidHttpHeader(header) => write!(f, "Invalid http header: {}", header),
            
            KurosabiError::InvalidHttpRangeHeader(header) => write!(f, "Invalid http range header: {}", header),
            KurosabiError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::fmt::Debug for KurosabiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KurosabiError::InvalidHttpHeader(header) => write!(f, "Invalid http header: {}", header),
            KurosabiError::InvalidHttpRangeHeader(header) => write!(f, "Invalid http range header: {}", header),
            KurosabiError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

pub enum HttpError {
    BadRequest(String),
    NotFound,
    MethodNotAllowed,
    InternalServerError,
    CUSTOM(u16, String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(message) => write!(f, "Bad Request: {}", message),
            HttpError::NotFound => write!(f, "Not Found"),
            HttpError::MethodNotAllowed => write!(f, "Method Not Allowed"),
            HttpError::InternalServerError => write!(f, "Internal Server Error"),
            HttpError::CUSTOM(status, message) => write!(f, "Status: {}, Message: {}", status, message),
        }
    }
}

impl std::fmt::Debug for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(message) => write!(f, "Bad Request: {}", message),
            HttpError::NotFound => write!(f, "Not Found"),
            HttpError::MethodNotAllowed => write!(f, "Method Not Allowed"),
            HttpError::InternalServerError => write!(f, "Internal Server Error"),
            HttpError::CUSTOM(status, message) => write!(f, "Status: {}, Message: {}", status, message),
        }
    }
}

impl HttpError {
    pub fn err_res(&self) -> Res {
        let mut res = Res::new();
        match self {
            HttpError::BadRequest(message) => {
                res.set_status(400);
                res.set_header("Content-Type", "text/plain");
                res = res.text(format!("Bad Request: {}", message).as_str());
            }
            HttpError::NotFound => {
                res.set_status(404);
                res.set_header("Content-Type", "text/plain");
                res = res.text("Not Found");
            }
            HttpError::MethodNotAllowed => {
                res.set_status(405);
                res.set_header("Content-Type", "text/plain");
                res = res.text("Method Not Allowed");
            }
            HttpError::InternalServerError => {
                res.set_status(500);
                res.set_header("Content-Type", "text/plain");
                res = res.text("Internal Server Error");
            }
            HttpError::CUSTOM(status, message) => {
                res.set_status(*status);
                res.set_header("Content-Type", "text/plain");
                res = res.text(message);
            }
        }
        res
    }
}