use futures::{AsyncWrite, AsyncWriteExt};

use crate::http::{code::HttpStatusCode, header::HttpHeader, version::HttpVersion};

pub struct HttpResponse<W: AsyncWrite + Unpin + 'static> {
    io_writer: W,
    /// 先頭14byteはレスポンスライン用に予約
    buf: Vec<u8>,
    headers: Option<HttpHeader>,
    response_line: HttpResponseLine,
}

impl<W: AsyncWrite + Unpin + 'static> HttpResponse<W> {
    pub fn new(io_writer: W) -> Self {
        HttpResponse {
            io_writer,
            buf: vec![0; 14],
            headers: None,
            response_line: HttpResponseLine::new(),
        }
    }

    pub(crate) fn reset(mut self) -> Self {
        self.buf.clear();
        self.buf.resize(14, 0);
        self.headers = None;
        self.response_line = HttpResponseLine::new();
        self
    }

    pub(crate) fn flag_flushed_buf(&mut self) {
        self.buf.truncate(0);
    }

    pub(crate) fn is_flushed(&self) -> bool {
        self.buf.len() == 0
    }

    pub fn header_add<K, V>(&mut self, key: K, value: V) -> &mut Self
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
        self
    }

    pub fn header_remove<S>(&mut self, key: S) -> &mut Self
    where
        S: std::borrow::Borrow<str>,
    {
        if let Some(headers) = &mut self.headers {
            headers.remove(key, &mut self.buf);
        }
        self
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

    pub fn inner_buf(&self) -> &Vec<u8> {
        &self.buf
    }

    pub fn writer(&mut self) -> &mut W {
        &mut self.io_writer
    }

    pub(crate) fn text_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "text/plain; charset=utf-8");
        // コンテンツ開始の改行
        self.buf.push(b'\r');
        self.buf.push(b'\n');
        self.buf.extend_from_slice(body.as_bytes());
    }

    pub(crate) fn binary_body(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        // コンテンツ開始の改行
        self.buf.push(b'\r');
        self.buf.push(b'\n');
        self.buf.extend_from_slice(body);
    }

    pub(crate) fn start_content(&mut self) {
        self.buf.push(b'\r');
        self.buf.push(b'\n');
    }

    /// 自動でよばれるのでrouter側で呼び出す必要性はほぼないです
    pub(crate) async fn send(&mut self) -> std::io::Result<()> {
        self.io_writer.write_all(&self.buf).await?;
        self.io_writer.flush().await
    }

    /// HTTPレスポンスラインを書き込む
    pub(crate) fn response_line_write(&mut self) {
        self.response_line.write_to_buf(&mut self.buf);
    }

    /// set http status code
    pub(crate) fn set_status_code<T>(&mut self, status_code: T) -> &mut Self
    where 
        T: Into<u16>, 
    {
        self.response_line.status_code = HttpStatusCode::from(status_code.into());
        self
    }
}

/// HTTPレスポンスのリクエストライン
pub struct HttpResponseLine {
    pub version: HttpVersion,
    pub status_code: HttpStatusCode,
}

impl HttpResponseLine {
    pub fn new() -> Self {
        HttpResponseLine {
            version: HttpVersion::ERR,
            status_code: HttpStatusCode::InternalServerError,
        }
    }

    pub fn write_to_buf(&self, buf: &mut Vec<u8>) {
        // bufの先頭14byteに書き込む
        buf[0..8].copy_from_slice(self.version.as_bytes());
        buf[8] = b' ';
        buf[9..12].copy_from_slice(self.status_code.as_bytes());
        buf[12] = b'\r';
        buf[13] = b'\n';
    }
}