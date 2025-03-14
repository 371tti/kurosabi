pub enum KurosabiError {
    InvalidHttpHeader(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for KurosabiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KurosabiError::InvalidHttpHeader(header) => write!(f, "Invalid http header: {}", header),
            KurosabiError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}
