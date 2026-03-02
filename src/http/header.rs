use std::borrow::Borrow;
use std::fmt::Write;
use std::ops::Range;

use std::string::String;

use futures_io::AsyncRead;
use futures_util::AsyncReadExt;

pub const MAX_HEADER_BYTES: usize = 32 * 1024;
pub const MAX_HEADERS: usize = 128;

#[inline(always)]
pub fn slice_by_range<'a>(buf: &'a [u8], range: &Range<usize>) -> &'a [u8] {
    &buf[range.start..range.end]
}

#[inline(always)]
fn trim_ascii_range(buf: &[u8], mut range: Range<usize>) -> Range<usize> {
    while range.start < range.end && buf[range.start].is_ascii_whitespace() {
        range.start += 1;
    }
    while range.start < range.end && buf[range.end - 1].is_ascii_whitespace() {
        range.end -= 1;
    }
    range
}

#[inline(always)]
fn find_header_end(buf: &[u8], start: usize) -> Option<usize> {
    // start..buf.len() の範囲で探す
    let mut i = start;

    // \r\n\r\n
    while i + 3 < buf.len() {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' && buf[i + 2] == b'\r' && buf[i + 3] == b'\n' {
            return Some(i + 4);
        }
        i += 1;
    }

    // \n\n（CRが無い入力用のフォールバック）
    let mut j = start;
    while j + 1 < buf.len() {
        if buf[j] == b'\n' && buf[j + 1] == b'\n' {
            return Some(j + 2);
        }
        j += 1;
    }

    None
}

#[derive(Debug, Clone)]
pub struct HttpHeader {
    /// 線形探索のほうが高速な場合が多かったため
    headers: Vec<HeaderEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HeaderEntry {
    key: Range<usize>,
    value: Range<usize>,
    line: Range<usize>,
}

impl HttpHeader {
    pub fn new() -> Self {
        HttpHeader { headers: Vec::with_capacity(32) }
    }

    #[inline(always)]
    pub fn insert<K>(&mut self, key: K, value: String, buf: &mut Vec<u8>)
    where
        K: Into<String>,
    {
        let key_str = key.into();
        let value_str = value;

        let key_bytes = key_str.as_bytes();
        let value_bytes = value_str.as_bytes();

        let line_start = buf.len();

        let key_start = buf.len();
        buf.extend_from_slice(key_bytes);
        let key_end = buf.len();

        buf.push(b':');

        let value_start = buf.len();
        buf.push(b' ');
        buf.extend_from_slice(value_bytes);
        let value_end = buf.len();

        buf.push(b'\r');
        buf.push(b'\n');

        let line_end = buf.len();

        self.headers.push(HeaderEntry {
            key: key_start..key_end,
            value: value_start..value_end,
            line: line_start..line_end,
        });
    }

    #[inline(always)]
    pub fn get<'a, S>(&self, key: S, buf: &'a [u8]) -> Option<&'a [u8]>
    where
        S: Borrow<str>,
    {
        let key_bytes = key.borrow().as_bytes();
        for h in &self.headers {
            if slice_by_range(buf, &h.key).eq_ignore_ascii_case(key_bytes) {
                return Some(slice_by_range(buf, &h.value));
            }
        }
        None
    }

    #[inline(always)]
    pub fn remove<S>(&mut self, key: S, buf: &mut Vec<u8>)
    where
        S: Borrow<str>,
    {
        // 前提条件:
        // - self.headers の並びは buf 上の並びと一致（line.start が昇順）
        // - line Range 同士は非重複
        // この前提なら「消した分だけ shift」だけで成立する。

        let key_bytes = key.borrow().as_bytes();
        let mut deleted_total = 0usize;
        let mut write_i = 0usize;

        for read_i in 0..self.headers.len() {
            let mut h = self.headers[read_i].clone();

            h.key.start -= deleted_total;
            h.key.end -= deleted_total;
            h.value.start -= deleted_total;
            h.value.end -= deleted_total;
            h.line.start -= deleted_total;
            h.line.end -= deleted_total;

            let is_match = {
                let buf_slice = buf.as_slice();
                slice_by_range(buf_slice, &h.key).eq_ignore_ascii_case(key_bytes)
            };

            if is_match {
                let start = h.line.start;
                let end = h.line.end;
                let removed_len = end - start;

                let old_len = buf.len();
                buf.copy_within(end..old_len, start);

                let new_len = old_len - removed_len;
                buf.truncate(new_len);

                deleted_total += removed_len;
                continue;
            }

            self.headers[write_i] = h;
            write_i += 1;
        }

        self.headers.truncate(write_i);
    }
}

impl HttpHeader {
    #[inline(always)]
    pub async fn parse_async<R>(reader: &mut R, buf: &mut Vec<u8>, start: usize) -> Option<(HttpHeader, usize)>
    where
        R: AsyncRead + Unpin,
    {
        // まずはヘッダ終端を探しつつ buf に追記
        let header_end = loop {
            // 既存の追記済み領域で終端が見つかるかチェック
            if let Some(end) = find_header_end(buf, start) {
                break end;
            }

            // まだなら追加で読む
            let mut tmp = [0u8; 4096];
            let n = reader.read(&mut tmp).await.ok()?;
            if n == 0 {
                return None; // EOF
            }
            buf.extend_from_slice(&tmp[..n]);

            // サイズ制限（start以降の増分だけをカウント）
            if buf.len() - start > MAX_HEADER_BYTES {
                return None;
            }
        };

        let body_start = header_end;

        // 次にヘッダ部を1パスでパース（buf[start..header_end]のみ）
        let mut header = HttpHeader::new();
        header.headers.clear();

        let mut cursor = start;
        let mut header_lines = 0usize;

        while cursor < header_end {
            // 1行切り出し（\n まで）
            let mut line_end = cursor;
            while line_end < header_end && buf[line_end] != b'\n' {
                line_end += 1;
            }
            if line_end >= header_end {
                // ヘッダ終端は見つかっているので、ここに来るなら壊れてる
                return None;
            }
            line_end += 1; // include '\n'

            let line_start = cursor;
            cursor = line_end;

            // 空行なら終端（ただし header_end で既に区切ってるので通常ここで終わる）
            let line = &buf[line_start..line_end];
            if line == b"\n" || line == b"\r\n" {
                break;
            }

            header_lines += 1;
            if header_lines > MAX_HEADERS {
                return None;
            }

            // 末尾の LF/CRLF を除いた content 範囲
            let mut content_end = line_end - 1; // exclude '\n'
            if content_end > line_start && buf[content_end - 1] == b'\r' {
                content_end -= 1;
            }
            let content = line_start..content_end;

            // ':' を探す（単純ループでOK。必要なら後でmemchr化）
            let mut colon = content.start;
            while colon < content.end && buf[colon] != b':' {
                colon += 1;
            }
            if colon == content.end {
                return None;
            }

            let key_range = trim_ascii_range(buf, content.start..colon);
            let value_range = trim_ascii_range(buf, (colon + 1)..content.end);

            header.headers.push(HeaderEntry {
                key: key_range,
                value: value_range,
                line: line_start..line_end,
            });
        }

        Some((header, body_start))
    }
}

#[derive(Clone, Debug)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    /// RFC1123/RFC2822-ish HTTP-date string (e.g. "Wed, 21 Oct 2015 07:28:00 GMT")
    pub expires: Option<String>,
    /// seconds
    pub max_age: Option<u64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
    pub partitioned: bool,
    pub priority: Option<CookiePriority>,
}

#[derive(Clone, Debug)]
pub enum CookiePriority {
    Low,
    Medium,
    High,
}

impl CookiePriority {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            CookiePriority::Low => "Low",
            CookiePriority::Medium => "Medium",
            CookiePriority::High => "High",
        }
    }
}

impl Cookie {
    /// Render into a `Set-Cookie` header value.
    /// Note: this does not include the "Set-Cookie: " prefix.
    pub fn to_set_cookie_value(&self) -> String {
        // Rough capacity guess to reduce realloc
        let mut out = String::with_capacity(self.name.len() + self.value.len() + 128);

        // name=value
        // (Caller should pre-encode value as needed; here we keep it raw.)
        let _ = write!(&mut out, "{}={}", self.name, self.value);

        if let Some(domain) = &self.domain {
            let _ = write!(&mut out, "; Domain={}", domain);
        }
        if let Some(path) = &self.path {
            let _ = write!(&mut out, "; Path={}", path);
        }
        if let Some(expires) = &self.expires {
            let _ = write!(&mut out, "; Expires={}", expires);
        }
        if let Some(max_age) = self.max_age {
            let _ = write!(&mut out, "; Max-Age={}", max_age);
        }

        if self.secure {
            out.push_str("; Secure");
        }
        if self.http_only {
            out.push_str("; HttpOnly");
        }
        if let Some(ss) = &self.same_site {
            let _ = write!(&mut out, "; SameSite={}", ss.as_str());
        }
        if let Some(p) = &self.priority {
            let _ = write!(&mut out, "; Priority={}", p.as_str());
        }
        if self.partitioned {
            out.push_str("; Partitioned");
        }

        out
    }
}

#[derive(Clone, Debug)]
pub struct CookieBuilder {
    cookie: Cookie,
}

impl CookieBuilder {
    /// By default:
    /// - Secure = true
    /// - HttpOnly = true
    /// - SameSite = None (unset; recommend setting explicitly)
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        CookieBuilder {
            cookie: Cookie {
                name: name.into(),
                value: value.into(),
                domain: None,
                path: None,
                expires: None,
                max_age: None,
                secure: true,
                http_only: true,
                same_site: None,
                partitioned: false,
                priority: None,
            },
        }
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.cookie.domain = Some(domain.into());
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.cookie.path = Some(path.into());
        self
    }

    /// Set Expires as a raw HTTP-date string (e.g. "Wed, 21 Oct 2015 07:28:00 GMT").
    /// If you use chrono, you can format it yourself before passing.
    pub fn expires(mut self, expires_http_date: impl Into<String>) -> Self {
        self.cookie.expires = Some(expires_http_date.into());
        self
    }

    /// Set Max-Age seconds.
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.cookie.max_age = Some(seconds);
        self
    }

    pub fn secure(mut self, on: bool) -> Self {
        self.cookie.secure = on;
        self
    }

    pub fn http_only(mut self, on: bool) -> Self {
        self.cookie.http_only = on;
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.cookie.same_site = Some(same_site);
        self
    }

    pub fn priority(mut self, p: CookiePriority) -> Self {
        self.cookie.priority = Some(p);
        self
    }

    /// CHIPS (Partitioned cookies). Typically requires Secure in practice.
    pub fn partitioned(mut self, on: bool) -> Self {
        self.cookie.partitioned = on;
        self
    }

    /// Convenience: delete cookie (set Max-Age=0). You still must match Path/Domain of the cookie you want to delete.
    pub fn delete(mut self) -> Self {
        self.cookie.max_age = Some(0);
        // Expires is optional if Max-Age=0, but some clients are happier if both exist.
        // Leave expires as-is unless caller sets it.
        self
    }

    pub fn build(self) -> Cookie {
        self.cookie
    }

    /// Build and render directly to a Set-Cookie header value.
    pub fn finish(self) -> String {
        self.cookie.to_set_cookie_value()
    }
}

pub fn format_cookie_expires(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
