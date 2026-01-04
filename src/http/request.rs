


use std::ops::Range;

use futures::{AsyncBufRead, AsyncRead, io::BufReader, AsyncBufReadExt};

use crate::{error::RouterError, http::header::HttpHeader};

pub struct HttpRequest<R: AsyncRead + Unpin + 'static> {
    io_reader: BufReader<R>,
    buf: Vec<u8>,
    /// headers のライフタイムは 'static 
    /// つまりbufと同じ
    headers: Option<HttpHeader>,
    request_line: Option<HttpRequestLine>,
}

impl<R: AsyncRead + Unpin + 'static> HttpRequest<R> {
    pub fn new(io_reader: R) -> Self {
        HttpRequest {
            io_reader: BufReader::new(io_reader),
            buf: Vec::new(),
            headers: None,
            request_line: None,
        }
    }

    pub async fn header_get<S>(&mut self, key: S) -> Result<Option<&str>, RouterError>
    where S: std::borrow::Borrow<str>, {
        if self.headers.is_none() {
            if let Some(headers) = HttpHeader::parse_async(&mut self.io_reader, &mut self.buf).await {
                self.headers = Some(headers);
            } else {
                return Err(RouterError::HttpErrorCodeWithMessage(
                    crate::http::code::HttpStatusCode::BadRequest,
                    "Failed to parse HTTP headers".to_string(),
                ));
            }
        }
        Ok(self.headers.as_ref().expect("unreachable").get(key, &self.buf).map(|v| std::str::from_utf8(v).ok()).flatten())
    }
}

impl<R: AsyncRead + Unpin + 'static> HttpRequest<R> {
    async fn parse_request(&mut self) -> Result<(), RouterError> {
        let request_line = HttpRequestLine::parse_async(&mut self.io_reader, &mut self.buf).await?;
        self.request_line = Some(request_line);
        Ok(())
    }
}

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
}

pub enum HttpVersion {
    HTTP10,
    HTTP11,
    HTTP20,
}

pub struct HttpRequestLine {
    method: HttpMethod,
    path: Range<usize>,
    version: HttpVersion,
}

impl HttpRequestLine {
    pub async fn parse_async<R: AsyncBufRead + Unpin + 'static>(
        reader: &mut R,
        buf: &mut Vec<u8>,
    ) -> Result<HttpRequestLine, RouterError> {
        let start = buf.len();
        let n = reader.read_until(b'\n', buf).await.map_err(|_| {
            RouterError::HttpErrorCodeWithMessage(
                crate::http::code::HttpStatusCode::BadRequest,
                "Failed to read request line".to_string(),
            )
        })?;

        if n == 0 {
            return Err(RouterError::HttpErrorCodeWithMessage(
                crate::http::code::HttpStatusCode::BadRequest,
                "Unexpected EOF while reading request line".to_string(),
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
                RouterError::HttpErrorCodeWithMessage(
                    crate::http::code::HttpStatusCode::BadRequest,
                    "Invalid request line format".to_string(),
                )
            })?;
            let sp2_rel = line[sp1 + 1..]
                .iter()
                .position(|&b| b == b' ')
                .ok_or_else(|| {
                    RouterError::HttpErrorCodeWithMessage(
                        crate::http::code::HttpStatusCode::BadRequest,
                        "Invalid request line format".to_string(),
                    )
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
                return Err(RouterError::HttpErrorCodeWithMessage(
                    crate::http::code::HttpStatusCode::BadRequest,
                    "Unsupported HTTP method".to_string(),
                ))
            }
        };

        if raw_path.is_empty() {
            return Err(RouterError::HttpErrorCodeWithMessage(
                crate::http::code::HttpStatusCode::BadRequest,
                "Invalid request line format".to_string(),
            ));
        }

        let version = match raw_version {
            b"HTTP/1.0" => HttpVersion::HTTP10,
            b"HTTP/1.1" => HttpVersion::HTTP11,
            b"HTTP/2.0" => HttpVersion::HTTP20,
            _ => {
                return Err(RouterError::HttpErrorCodeWithMessage(
                    crate::http::code::HttpStatusCode::BadRequest,
                    "Unsupported HTTP version".to_string(),
                ))
            }
        };

        Ok(HttpRequestLine {
            method,
            path: path_range,
            version,
        })
    }
}