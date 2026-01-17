use std::ops::Range;

use futures_io::AsyncRead;
use futures_util::AsyncReadExt;

use crate::{
    error::RouterError,
    http::{header::HttpHeader, method::HttpMethod, version::HttpVersion},
};

pub struct HttpRequest<R: AsyncRead + Unpin + 'static> {
    io_reader: R,
    buf: Vec<u8>,
    // 0 => no readed body yet
    body_start: usize,
    headers_start: usize,
    /// headers のライフタイムは 'static
    /// つまりbufと同じ
    headers: HttpHeader,
    request_line: HttpRequestLine,
}

impl<R: AsyncRead + Unpin + 'static> HttpRequest<R> {
    /// Requestに設定されたHeaderから値を取得する
    #[inline(always)]
    pub async fn header_get<S>(&mut self, key: S) -> Option<&str>
    where
        S: std::borrow::Borrow<str>,
    {
        std::str::from_utf8(self.headers.get(key, &self.buf)?).ok()
    }

    #[inline(always)]
    pub fn request_line(&self) -> &HttpRequestLine {
        &self.request_line
    }

    /// get full request path
    #[inline(always)]
    pub fn path_full(&self) -> &str {
        let path_range = &self.request_line.path;
        std::str::from_utf8(&self.buf[path_range.clone()]).expect("Invalid UTF-8 in request path")
    }

    /// get request http method
    #[inline(always)]
    pub fn method(&self) -> &HttpMethod {
        &self.request_line.method
    }

    /// get request http version
    #[inline(always)]
    pub fn version(&self) -> &HttpVersion {
        &self.request_line.version
    }

    #[inline(always)]
    pub(crate) fn into_reader(self) -> R {
        self.io_reader
    }

    #[inline(always)]
    pub async fn read_body_bytes(&mut self) -> std::io::Result<Vec<u8>> {
        let content_length = if let Some(cl_val) = self.header_get("Content-Length").await {
            cl_val.parse::<usize>().unwrap_or(0)
        } else {
            0
        };
        let have = self.buf.len().saturating_sub(self.body_start);
        if have < content_length {
            let need = content_length - have;

            // buf を need 分だけ伸ばして、その領域に read_exact
            let old_len = self.buf.len();
            self.buf.resize(old_len + need, 0);

            self.io_reader
                .read_exact(&mut self.buf[old_len..old_len + need])
                .await?;
        }
        Ok(self.buf.split_off(self.body_start))
    }

    #[inline(always)]
    pub async fn read_body_bytes_size(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        let have = self.buf.len().saturating_sub(self.body_start);
        if have < size {
            let need = size - have;

            // buf を need 分だけ伸ばして、その領域に read_exact
            let old_len = self.buf.len();
            self.buf.resize(old_len + need, 0);

            self.io_reader
                .read_exact(&mut self.buf[old_len..old_len + need])
                .await?;
        }
        Ok(self.buf.split_off(self.body_start))
    }

    #[inline(always)]
    pub async fn read_body_to_end(&mut self) -> std::io::Result<Vec<u8>> {
        let mut body = self.buf.split_off(self.body_start);
        self.io_reader.read_to_end(&mut body).await?;
        Ok(body)
    }

    #[inline(always)]
    #[cfg(feature = "json")]
    pub async fn read_json_de<T>(&mut self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let body_bytes = self
            .read_body_bytes()
            .await
            .map_err(|e| serde_json::Error::io(e))?;
        serde_json::from_slice(&body_bytes)
    }
}

impl<R: AsyncRead + Unpin + 'static> HttpRequest<R> {
    pub fn new(io_reader: R) -> Self {
        HttpRequest {
            io_reader,
            buf: Vec::with_capacity(1024 * 1),
            body_start: 0,
            headers_start: 0,
            headers: HttpHeader::new(),
            request_line: HttpRequestLine::new(),
        }
    }

    #[inline(always)]
    pub async fn parse_request_line(mut self) -> Result<HttpRequest<R>, HttpRequest<R>> {
        let (request_line, headers_start) = match HttpRequestLine::parse_async(&mut self.io_reader, &mut self.buf).await
        {
            Ok(line) => line,
            Err(e) => {
                let line = HttpRequestLine {
                    method: HttpMethod::ERR,
                    path: if let RouterError::InvalidHttpRequest(range, _) = e {
                        range
                    } else {
                        0..0
                    },
                    version: HttpVersion::ERR,
                };
                return Err(HttpRequest {
                    io_reader: self.io_reader,
                    buf: self.buf,
                    body_start: 0,
                    headers_start: 0,
                    headers: HttpHeader::new(),
                    request_line: line,
                });
            },
        };
        Ok(HttpRequest {
            io_reader: self.io_reader,
            buf: self.buf,
            body_start: 0,
            headers_start: headers_start,
            headers: self.headers,
            request_line,
        })
    }

    #[inline(always)]
    pub async fn parse_request(mut self) -> Result<HttpRequest<R>, HttpRequest<R>> {
        let (headers, body_start) =
            match HttpHeader::parse_async(&mut self.io_reader, &mut self.buf, self.headers_start).await {
                Some(headers) => headers,
                None => {
                    return Err(self);
                },
            };
        self.body_start = body_start;
        self.headers = headers;
        Ok(self)
    }
}

pub struct HttpRequestLine {
    method: HttpMethod,
    path: Range<usize>,
    version: HttpVersion,
}

impl HttpRequestLine {
    pub fn new() -> Self {
        HttpRequestLine {
            method: HttpMethod::ERR,
            path: 0..0,
            version: HttpVersion::ERR,
        }
    }

    #[inline(always)]
    pub async fn parse_async<R: AsyncRead + Unpin + 'static>(
        reader: &mut R,
        buf: &mut Vec<u8>,
    ) -> Result<(HttpRequestLine, usize), RouterError> {
        let start = buf.len();
        // Read bytes into buf until we find a newline or EOF
        let mut temp_buf = [0u8; 1024];
        let mut n = 0;
        loop {
            let read_bytes = reader.read(&mut temp_buf).await.map_err(|_| {
                RouterError::InvalidHttpRequest(start..buf.len(), "Failed to read request line".to_string())
            })?;
            if read_bytes == 0 {
                break;
            }
            buf.extend_from_slice(&temp_buf[..read_bytes]);
            if let Some(pos) = buf[start..].iter().position(|&b| b == b'\n') {
                n = pos + 1;
                break;
            }
        }

        if n == 0 {
            return Err(RouterError::InvalidHttpRequest(
                start..buf.len(),
                "Empty request line".to_string(),
            ));
        }

        // `read_until` includes '\n'. Strip trailing CRLF/LF for parsing.
        let mut line_end = start + n;
        if line_end > start && buf[line_end - 1] == b'\n' {
            line_end -= 1;
        }
        if line_end > start && buf[line_end - 1] == b'\r' {
            line_end -= 1;
        }

        let line = &buf[start..line_end];

        let (raw_method, raw_path, raw_version, path_range) = {
            let sp1 = line.iter().position(|&b| b == b' ').ok_or_else(|| {
                RouterError::InvalidHttpRequest(start..line_end, "Invalid request line format".to_string())
            })?;
            let sp2_rel = line[sp1 + 1..]
                .iter()
                .position(|&b| b == b' ')
                .ok_or_else(|| {
                    RouterError::InvalidHttpRequest(start..line_end, "Invalid request line format".to_string())
                })?;
            let sp2 = sp1 + 1 + sp2_rel;

            let raw_method = &line[..sp1];
            let raw_path = &line[sp1 + 1..sp2];
            let raw_version = &line[sp2 + 1..];
            let path_range = (start + sp1 + 1)..(start + sp2);
            (raw_method, raw_path, raw_version, path_range)
        };

        let method = match raw_method {
            b"GET" => HttpMethod::GET,
            b"POST" => HttpMethod::POST,
            b"PUT" => HttpMethod::PUT,
            b"DELETE" => HttpMethod::DELETE,
            b"HEAD" => HttpMethod::HEAD,
            b"OPTIONS" => HttpMethod::OPTIONS,
            b"PATCH" => HttpMethod::PATCH,
            b"TRACE" => HttpMethod::TRACE,
            b"CONNECT" => HttpMethod::CONNECT,
            _ => {
                return Err(RouterError::InvalidHttpRequest(
                    start..line_end,
                    "Unsupported HTTP method".to_string(),
                ));
            },
        };

        if raw_path.is_empty() {
            return Err(RouterError::InvalidHttpRequest(
                start..line_end,
                "Invalid request line format".to_string(),
            ));
        }

        let version = match raw_version {
            b"HTTP/1.0" => HttpVersion::HTTP10,
            b"HTTP/1.1" => HttpVersion::HTTP11,
            b"HTTP/2.0" => HttpVersion::HTTP20,
            _ => {
                return Err(RouterError::InvalidHttpRequest(
                    start..line_end,
                    "Unsupported HTTP version".to_string(),
                ));
            },
        };

        let headers_start = start + n;
        Ok((
            HttpRequestLine { method, path: path_range, version },
            headers_start,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeSpec {
    FromToInclusive { start: u64, end: u64 }, // bytes=START-END (end inclusive)
    From { start: u64 },                      // bytes=START-
    Suffix { len: u64 },                      // bytes=-SUFFIX
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeParse {
    Invalid,
    Valid(RangeSpec),
}

#[inline]
pub fn parse_range_header_value(range_value: &str) -> RangeParse {
    let s = range_value.trim();
    let rest = if let Some(v) = s.strip_prefix("bytes=") {
        v
    } else if let Some(v) = s.strip_prefix("Bytes=") {
        v
    } else {
        return RangeParse::Invalid;
    };
    if rest.contains(',') {
        return RangeParse::Invalid;
    }
    let (a, b) = match rest.split_once('-') {
        Some(v) => v,
        None => return RangeParse::Invalid,
    };

    match (a.is_empty(), b.is_empty()) {
        (true, true) => RangeParse::Invalid,

        (false, false) => {
            let start: u64 = match a.parse() {
                Ok(v) => v,
                Err(_) => return RangeParse::Invalid,
            };
            let end: u64 = match b.parse() {
                Ok(v) => v,
                Err(_) => return RangeParse::Invalid,
            };
            if start > end {
                return RangeParse::Invalid;
            }
            RangeParse::Valid(RangeSpec::FromToInclusive { start, end })
        },

        (false, true) => {
            let start: u64 = match a.parse() {
                Ok(v) => v,
                Err(_) => return RangeParse::Invalid,
            };
            RangeParse::Valid(RangeSpec::From { start })
        },

        (true, false) => {
            let len: u64 = match b.parse() {
                Ok(v) => v,
                Err(_) => return RangeParse::Invalid,
            };
            if len == 0 {
                return RangeParse::Invalid;
            }
            RangeParse::Valid(RangeSpec::Suffix { len })
        },
    }
}
