use futures::{AsyncWrite, io::BufWriter};

use crate::http::header::HttpHeader;

pub struct HttpResponse<W: AsyncWrite + Unpin + 'static> {
    io_writer: BufWriter<W>,
    buf: Vec<u8>,
    headers: Option<HttpHeader>,
}

impl<W: AsyncWrite + Unpin + 'static> HttpResponse<W> {
    pub fn new(io_writer: W) -> Self {
        HttpResponse {
            io_writer: BufWriter::new(io_writer),
            buf: Vec::new(),
            headers: None,
        }
    }

    pub fn header_add<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        if self.headers.is_none() {
            self.headers = Some(HttpHeader::new());
        }

        if let Some(headers) = &mut self.headers {
            headers.insert(key, value.into(), &mut self.buf);
        }
    }

    pub fn header_remove<S>(&mut self, key: S)
    where
        S: std::borrow::Borrow<str>,
    {
        if let Some(headers) = &mut self.headers {
            headers.remove(key, &mut self.buf);
        }
    }

    pub fn header_get<S>(&self, key: S) -> Option<&str>
    where
        S: std::borrow::Borrow<str>,
    {
        if let Some(headers) = &self.headers {
            if let Some(value_bytes) = headers.get(key, &self.buf) {
                return std::str::from_utf8(value_bytes).ok();
            }
        }
        None
    }
}