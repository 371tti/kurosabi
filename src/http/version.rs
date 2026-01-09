pub enum HttpVersion {
    HTTP10,
    HTTP11,
    HTTP20,
    ERR,
}

impl HttpVersion {
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8; 8] {
        match self {
            HttpVersion::HTTP10 => b"HTTP/1.0",
            HttpVersion::HTTP11 => b"HTTP/1.1",
            HttpVersion::HTTP20 => b"HTTP/2.0",
            HttpVersion::ERR => b"HTTP/ERR",
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpVersion::HTTP10 => "HTTP/1.0",
            HttpVersion::HTTP11 => "HTTP/1.1",
            HttpVersion::HTTP20 => "HTTP/2.0",
            HttpVersion::ERR => "HTTP/ERR",
        }
    }
}
