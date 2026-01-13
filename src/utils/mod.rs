use futures_io::AsyncWrite;
use futures_timer::Delay;
use futures_util::{
    AsyncWriteExt,
    future::{Either, select},
};
use std::{io::IoSlice, time::Duration};

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
