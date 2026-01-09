use futures_io::AsyncWrite;
use futures_util::AsyncWriteExt;

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
        let mut buf = vec![0; 14];
        buf.reserve(1024 * 1 - 14);
        HttpResponse {
            io_writer,
            buf,
            headers: None,
            response_line: HttpResponseLine::new(),
        }
    }

    /// 構築したレスポンスを消し飛ばす
    pub(crate) fn reset(mut self) -> Self {
        self.buf.clear();
        self.buf.resize(14, 0);
        self.headers = None;
        self.response_line = HttpResponseLine::new();
        self
    }

    /// バッファを空にしてフラッシュ済みとマークする
    #[inline]
    pub fn flag_flushed_buf(&mut self) {
        self.buf.truncate(0);
    }

    #[inline]
    pub(crate) fn is_flushed(&self) -> bool {
        self.buf.len() == 0
    }

    /// # please use `Connection::add_header` instead
    /// # Performance
    /// bodyを追加する前にheader_addでContent-Lengthを設定しておくことを推奨
    /// bodyを追加した後にContent-Lengthを設定すると、buf shiftが発生しパフォーマンスが低下する可能性があります
    #[inline]
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

    /// # please use `Connection::remove_header` instead
    /// # Performance
    /// bodyを追加する前にheader_removeでContent-Lengthを削除しておくことを推奨
    /// bodyを追加した後にContent-Lengthを削除すると、buf shiftが発生しパフォーマンスが低下する可能性があります
    #[inline]
    pub(crate) fn header_remove<S>(&mut self, key: S) -> &mut Self
    where
        S: std::borrow::Borrow<str>,
    {
        if let Some(headers) = &mut self.headers {
            headers.remove(key, &mut self.buf);
        }
        self
    }

    /// Responseに設定されたHeaderから値を取得する
    #[inline]
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

    /// 内部バッファへの参照を取得する
    #[inline]
    pub fn inner_buf(&self) -> &Vec<u8> {
        &self.buf
    }

    /// 内部バッファへの可変参照を取得する
    ///
    /// # Safety
    /// HTTPレスポンスの構築の責任はこれであなたのもの
    #[inline]
    pub fn inner_buf_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    /// 内部ライターへの可変参照を取得する
    /// 独自拡張用
    ///
    /// # Safety
    /// HTTPレスポンスの送信の責任はこれであなたのもの
    #[inline]
    pub fn writer(&mut self) -> &mut W {
        &mut self.io_writer
    }

    #[inline(always)]
    pub(crate) fn text_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "text/plain; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn binary_body(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn html_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "text/html; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn json_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "application/json; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn xml_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "application/xml; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn csv_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "text/csv; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn css_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "text/css; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn js_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "application/javascript; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn png_body(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "image/png");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn jpg_body(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "image/jpeg");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn gif_body(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "image/gif");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn svg_body(&mut self, body: &str) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "image/svg+xml; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body.as_bytes());
    }

    #[inline(always)]
    pub(crate) fn pdf_body(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "application/pdf");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn xml_body_bytes(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "application/xml; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn json_body_bytes(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "application/json; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub(crate) fn csv_body_bytes(&mut self, body: &[u8]) {
        self.header_add("Content-Length", body.len().to_string());
        self.header_add("Content-Type", "text/csv; charset=utf-8");
        self.start_content();
        self.buf.extend_from_slice(body);
    }

    #[inline(always)]
    pub fn start_content(&mut self) {
        self.buf.push(b'\r');
        self.buf.push(b'\n');
    }

    /// 自動でよばれるのでrouter側で呼び出す必要性はほぼないです
    #[inline(always)]
    pub async fn send(&mut self) -> std::io::Result<()> {
        self.io_writer.write_all(&self.buf).await?;
        self.io_writer.flush().await
    }

    /// HTTPレスポンスラインを書き込む
    #[inline(always)]
    pub fn response_line_write(&mut self) {
        self.response_line.write_to_buf(&mut self.buf);
    }

    /// set http status code
    #[inline(always)]
    pub fn set_status_code<T>(&mut self, status_code: T) -> &mut Self
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
            version: HttpVersion::HTTP11,
            status_code: HttpStatusCode::InternalServerError,
        }
    }

    #[inline(always)]
    pub fn write_to_buf(&self, buf: &mut Vec<u8>) {
        // bufの先頭14byteに書き込む
        buf[0..8].copy_from_slice(self.version.as_bytes());
        buf[8] = b' ';
        buf[9..12].copy_from_slice(self.status_code.as_bytes());
        buf[12] = b'\r';
        buf[13] = b'\n';
    }
}
