#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    TRACE,
    CONNECT,
    ERR,
}

impl HttpMethod {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            HttpMethod::GET => b"GET",
            HttpMethod::POST => b"POST",
            HttpMethod::PUT => b"PUT",
            HttpMethod::DELETE => b"DELETE",
            HttpMethod::HEAD => b"HEAD",
            HttpMethod::OPTIONS => b"OPTIONS",
            HttpMethod::PATCH => b"PATCH",
            HttpMethod::TRACE => b"TRACE",
            HttpMethod::CONNECT => b"CONNECT",
            HttpMethod::ERR => b"ERR",
        }
    }

    pub fn as_str(&self) -> &'static str {
        std::str::from_utf8(self.as_bytes()).unwrap()
    }
}