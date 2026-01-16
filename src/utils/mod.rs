use futures_io::AsyncWrite;
use futures_timer::Delay;
use futures_util::{
    AsyncWriteExt,
    future::{Either, select},
};
use std::{borrow::Cow, io::IoSlice, time::Duration};

#[inline(always)]
pub async fn with_timeout<F, T>(fut: F, dur: Duration) -> Result<T, ()>
where
    F: std::future::Future<Output = T> + Unpin,
{
    match select(fut, Delay::new(dur)).await {
        Either::Left((val, _delay_future)) => Ok(val),
        Either::Right((_unit, _original_future)) => Err(()),
    }
}

#[inline]
pub fn write_hex_crlf(mut n: usize, out: &mut [u8; 32]) -> &[u8] {
    // 最大でも "FFFFFFFFFFFFFFFF\r\n" 程度なので 32 で十分
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut i = out.len();
    out[i - 1] = b'\n';
    out[i - 2] = b'\r';
    i -= 2;

    if n == 0 {
        i -= 1;
        out[i] = b'0';
        return &out[i..];
    }

    while n != 0 {
        let d = (n & 0xF) as usize;
        i -= 1;
        out[i] = HEX[d];
        n >>= 4;
    }

    &out[i..]
}

/// Allocation-free write_all for exactly 3 buffers (some may be empty).
#[inline]
pub async fn write_all_vectored3<W>(w: &mut W, mut a: &[u8], mut b: &[u8], mut c: &[u8]) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    while !(a.is_empty() && b.is_empty() && c.is_empty()) {
        let mut ios = [IoSlice::new(&[]), IoSlice::new(&[]), IoSlice::new(&[])];
        let mut k = 0;

        if !a.is_empty() {
            ios[k] = IoSlice::new(a);
            k += 1;
        }
        if !b.is_empty() {
            ios[k] = IoSlice::new(b);
            k += 1;
        }
        if !c.is_empty() {
            ios[k] = IoSlice::new(c);
            k += 1;
        }

        let n = w.write_vectored(&ios[..k]).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "write_vectored returned 0",
            ));
        }

        let mut left = n;

        if !a.is_empty() {
            let d = left.min(a.len());
            a = &a[d..];
            left -= d;
        }
        if left > 0 && !b.is_empty() {
            let d = left.min(b.len());
            b = &b[d..];
            left -= d;
        }
        if left > 0 && !c.is_empty() {
            let d = left.min(c.len());
            c = &c[d..];
        }
    }
    Ok(())
}


/// RFC3986 unreserved: ALPHA / DIGIT / "-" / "." / "_" / "~"
#[inline]
fn is_unreserved(b: u8) -> bool {
    matches!(b,
        b'A'..=b'Z' |
        b'a'..=b'z' |
        b'0'..=b'9' |
        b'-' | b'.' | b'_' | b'~'
    )
}

/// URL percent-encode（path向け）
/// - UTF-8 bytes を対象
/// - unreserved 以外は %HH
pub fn url_encode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len() * 3);

    for &b in bytes {
        if is_unreserved(b) || b == b'/' {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(hex_upper(b >> 4));
            out.push(hex_upper(b & 0x0F));
        }
    }
    out
}

#[inline]
fn hex_upper(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'A' + (n - 10)) as char,
        _ => unreachable!(),
    }
}

/// URL percent-decode（path向け）
/// - %HH をデコード
/// - 不正な % / 非UTF-8 は Err
pub fn url_decode_safe(input: &str) -> Result<Cow<'_, str>, UrlDecodeError> {
    if !input.as_bytes().contains(&b'%') {
        return Ok(Cow::Borrowed(input));
    }
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(UrlDecodeError::InvalidPercent);
                }
                let hi = from_hex(bytes[i + 1])?;
                let lo = from_hex(bytes[i + 2])?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }

    match String::from_utf8(out) {
        Ok(s) => Ok(Cow::Owned(s)),
        Err(_) => Err(UrlDecodeError::InvalidUtf8),
    }
}

#[inline]
fn from_hex(b: u8) -> Result<u8, UrlDecodeError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(UrlDecodeError::InvalidHex),
    }
}

/// Errなし できるだけ高速な
/// - '%' が無い場合: Borrowed を返す
/// - '%HH' はデコードする
/// - 壊れた '%' は '%' のまま残す
/// - 生成後に UTF-8 が壊れてたら、その時だけ lossy にする
#[inline]
pub fn url_decode_fast(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();

    if !bytes.contains(&b'%') {
        return Cow::Borrowed(input);
    }

    let mut out = Vec::with_capacity(bytes.len());

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 < bytes.len() {
                if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                    out.push((hi << 4) | lo);
                    i += 3;
                    continue;
                }
            }
            out.push(b'%');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    match String::from_utf8(out) {
        Ok(s) => Cow::Owned(s),
        Err(e) => Cow::Owned(String::from_utf8_lossy(e.as_bytes()).into_owned()),
    }
}

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UrlDecodeError {
    InvalidPercent,
    InvalidHex,
    InvalidUtf8,
}