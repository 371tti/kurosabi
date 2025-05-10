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
    InternalServerError(String),
    RangeNotSatisfiable,
    InvalidLength(String),
    CUSTOM(u16, String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(message) => write!(f, "Bad Request: {}", message),
            HttpError::NotFound => write!(f, "Not Found"),
            HttpError::MethodNotAllowed => write!(f, "Method Not Allowed"),
            HttpError::InternalServerError(message) => write!(f, "Internal Server Error: {}", message),
            HttpError::RangeNotSatisfiable => write!(f, "Range Not Satisfiable"),
            HttpError::InvalidLength(message) => write!(f, "Invalid Length: {}", message),
            HttpError::CUSTOM(status, message) => write!(f, "Status: {}, Message: {}", status, message),
        }
    }
}

impl std::fmt::Debug for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpError::BadRequest(message) =>           write!(f, "Bad Request: {}", message),
            HttpError::NotFound =>                               write!(f, "Not Found ============="),
            HttpError::MethodNotAllowed =>                       write!(f, "Method Not Allowed ===="),
            HttpError::InternalServerError(message) =>          write!(f, "Internal Server Error: {}", message),
            HttpError::RangeNotSatisfiable =>                     write!(f, "Range Not Satisfiable"),
            HttpError::InvalidLength(message) =>       write!(f, "Invalid Length: {}", message),
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
                res.header.set("Content-Type", "text/plain");
                res.text(format!("Bad Request: {}", message).as_str());
            }
            HttpError::NotFound => {
                res.set_status(404);
                res.header.set("Content-Type", "text/plain");
                res.text("Not Found");
            }
            HttpError::MethodNotAllowed => {
                res.set_status(405);
                res.header.set("Content-Type", "text/plain");
                res.text("Method Not Allowed");
            }
            HttpError::InternalServerError(message) => {
                res.set_status(500);
                res.header.set("Content-Type", "text/plain");
                res.text(format!("Internal Server Error: {}", message).as_str());
            }
            HttpError::RangeNotSatisfiable => {
                res.set_status(416);
                res.header.set("Content-Type", "text/plain");
                res.text("Range Not Satisfiable");
            }
            HttpError::InvalidLength(message) => {
                res.set_status(416);
                res.header.set("Content-Type", "text/plain");
                res.text(format!("Invalid Length: {}", message).as_str());
            }
            HttpError::CUSTOM(status, message) => {
                res.set_status(*status);
                res.header.set("Content-Type", "text/plain");
                res.text(message);
            }
        }
        res
    }
}