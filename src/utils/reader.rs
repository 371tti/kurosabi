pub struct StreamAsyncRead<S> {
    stream: S,
    buffer: Option<Bytes>,
}

impl<S> StreamAsyncRead<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: None,
        }
    }
}

impl<S> AsyncRead for StreamAsyncRead<S>
where
    S: Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            // 現在のバッファが空の場合、新しいデータをストリームから取得
            if self.buffer.is_none() {
                match Pin::new(&mut self.stream).poll_next(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        self.buffer = Some(bytes);
                    }
                    Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(e)),
                    Poll::Ready(None) => return Poll::Ready(Ok(())), // ストリーム終了
                    Poll::Pending => return Poll::Pending,
                }
            }

            // バッファにデータがある場合、それを `buf` にコピー
            if let Some(bytes) = &mut self.buffer {
                let len = bytes.len().min(buf.remaining());
                buf.put_slice(&bytes[..len]);
                *bytes = bytes.slice(len..);

                // バッファが空になったらクリア
                if bytes.is_empty() {
                    self.buffer = None;
                }

                return Poll::Ready(Ok(()));
            }
        }
    }
}

