use std::borrow::Borrow;
use std::ops::Range;

use std::string::String;

use futures_io::AsyncBufRead;
use futures_util::AsyncBufReadExt;

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
        HttpHeader { headers: Vec::new() }
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
    pub async fn parse_async<R>(reader: &mut R, buf: &mut Vec<u8>) -> Option<HttpHeader>
    where
        R: AsyncBufRead + Unpin,
    {
        const MAX_HEADER_BYTES: usize = 32 * 1024; // 好みで調整
        const MAX_HEADERS: usize = 128; // 好みで調整

        let mut header = HttpHeader::new();

        let mut total = 0usize;

        // 参照スライスを保持する前に、ヘッダ全体を buf に読み切る。
        // そうしないと、&[u8] を保持したまま buf を伸長できず借用エラーになる。
        let start = buf.len();
        let mut header_lines = 0usize;

        loop {
            // read_until は '\n' を含めて入る
            let cursor = buf.len();
            let n = reader.read_until(b'\n', buf).await.ok()?;
            if n == 0 {
                // EOF
                return None;
            }

            let line_end = cursor + n;
            let line = &buf[cursor..line_end];

            total += n;
            if total > MAX_HEADER_BYTES {
                // ヘッダ肥大化防止（431相当）
                return None;
            }

            // 空行（CRLF or LF）でヘッダ終端
            if line == b"\n" || line == b"\r\n" {
                break;
            }

            header_lines += 1;
            if header_lines > MAX_HEADERS {
                return None;
            }
        }

        // ヘッダ行数に応じて Vec を事前確保
        header.headers.reserve(header_lines);

        // 2パス目：ここから先は buf を伸ばさないので、Range(オフセット)を保持できる。
        let end = buf.len();
        let mut cursor = start;
        while cursor < end {
            let rel_nl = match buf[cursor..end].iter().position(|&b| b == b'\n') {
                Some(i) => i,
                None => break,
            };
            let line_start = cursor;
            let line_end = cursor + rel_nl + 1;
            let line = &buf[line_start..line_end];
            cursor = line_end;

            // 空行（CRLF or LF）でヘッダ終端
            if line == b"\n" || line == b"\r\n" {
                break;
            }

            // 末尾の LF/CRLF を取り除く（key/value の Range 用）
            let mut content_end = line_end - 1; // exclude '\n'
            if content_end > line_start && buf[content_end - 1] == b'\r' {
                content_end -= 1;
            }
            let content = line_start..content_end;

            // ':' で2つの Range にZero-copy分割
            let line_slice = &buf[content.start..content.end];
            let colon_rel = match line_slice.iter().position(|&b| b == b':') {
                Some(i) => i,
                None => return None,
            };

            let key_range = content.start..(content.start + colon_rel);
            let value_range = (content.start + colon_rel + 1)..content.end;

            // 先頭と末尾の空白を取り除く
            let key_range = trim_ascii_range(buf, key_range);
            let value_range = trim_ascii_range(buf, value_range);
            header.headers.push(HeaderEntry {
                key: key_range,
                value: value_range,
                line: line_start..line_end,
            });
        }

        Some(header)
    }
}