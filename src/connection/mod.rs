use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{error::{ErrorPare, Result, RouterError}, http::{code::HttpStatusCode, request::HttpRequest, response::HttpResponse}};

/// Connection struct
/// one http connection per one instance
pub struct Connection<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static, S: ConnectionState = NoneBody> {
    pub c: C,
    pub req: HttpRequest<R>,
    pub res: HttpResponse<W>,
    /// State
    phantom: std::marker::PhantomData<S>,
}

pub trait ConnectionState {}
pub struct NoneBody;
impl ConnectionState for NoneBody {}
pub struct StatusSetNoneBody;
impl ConnectionState for StatusSetNoneBody {}
pub struct StreamingResponse;
impl ConnectionState for StreamingResponse {}
pub struct ChunkedResponse;
impl ConnectionState for ChunkedResponse {}
pub struct ResponseReadyToSend;
impl ConnectionState for ResponseReadyToSend {}
pub struct CompletedResponse;
impl ConnectionState for CompletedResponse {}

pub const STREAM_CHUNK_SIZE: usize = 4096;

pub trait SizedAsyncRead: AsyncRead + Unpin + 'static {
    fn size(&self) -> usize;
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, NoneBody> {
    pub fn new(c: C, req: HttpRequest<R>, res: HttpResponse<W>) -> Self {
        Connection {
            c,
            req,
            res,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, NoneBody> {
    pub fn set_status_code<T>(mut self, status_code: T) -> Connection<C, R, W, StatusSetNoneBody>
    where
        T: Into<u16>,
    {
        self.res.set_status_code(status_code);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn text_body(self, body: &str) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).text_body(body)
    }

    pub fn binary_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).binary_body(body)
    }
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, StatusSetNoneBody> {
    pub fn text_body(mut self, body: &str) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.text_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn binary_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.binary_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub async fn streaming<T>(mut self, mut reader: T) -> Result<Connection<C, R, W, ResponseReadyToSend>>
    where
        T: SizedAsyncRead + Send,
    {
        let size = reader.size();
        self.res.header_add("Content-Length", size.to_string());
        self.res.response_line_write();
        self.res.start_content();
        let res = self.res.send().await;

        let writer = self.res.writer();

        let res = if let Err(e) = res {
            Err(e)
        } else {
            loop {
                let mut buf = vec![0u8; STREAM_CHUNK_SIZE];
                let n = AsyncReadExt::read(&mut reader, &mut buf).await;
                let n = match n {
                    Ok(0) => break Ok(()), // EOF
                    Ok(n) => n,
                    Err(e) => {
                        break Err(e)
                    }
                };
                let res = writer.write_all(&buf[..n]).await;
                if let Err(e) = res {
                    break Err(e);
                }
            }
        };

        match res {
            Ok(_) => (),
            Err(e) => {
                let response_ready_conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(ErrorPare {
                    router_error: RouterError::IoError(e),
                    connection: response_ready_conn,
                });
            }
        };

        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        })
    }

    pub async fn ready_chunked(mut self) -> Result<Connection<C, R, W, ChunkedResponse>> {
        self.res.header_add("Transfer-Encoding", "chunked");
        self.res.response_line_write();
        self.res.start_content();
        match self.res.send().await {
            Ok(_) => (),
            Err(e) => {
                // ChunkedResponse型に変換してからErrorPareに渡す
                let chunked_conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(ErrorPare {
                    router_error: RouterError::IoError(e),
                    connection: chunked_conn,
                });
            }
        }
        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        })
    }
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, ChunkedResponse> {
    pub async fn send_chunk(&mut self, chunk: &[u8]) -> std::io::Result<()> {
        let chunk_size_hex = format!("{:X}\r\n", chunk.len());
        self.res.writer().write_all(chunk_size_hex.as_bytes()).await?;
        self.res.writer().write_all(chunk).await?;
        self.res.writer().write_all(b"\r\n").await
    }

    pub async fn send_last_chunk(&mut self) -> std::io::Result<()> {
        self.res.writer().write_all(b"0\r\n\r\n").await
    }

    pub fn close_chunked(mut self) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.flag_flushed_buf();
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, ResponseReadyToSend> {
    pub(crate) async fn flush(mut self) -> Result<Connection<C, R, W, CompletedResponse>> {
        if self.res.is_flushed() {
            return Ok(Connection {
                c: self.c,
                req: self.req,
                res: self.res,
                phantom: std::marker::PhantomData,
            });
        }
        self.res.response_line_write();
        match self.res.send().await {
            Ok(_) => (),
            Err(e) => {
                // CompletedResponse型に変換してからErrorPareに渡す
                let completed_conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(ErrorPare {
                    router_error: RouterError::IoError(e),
                    connection: completed_conn,
                });
            }
        }
        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        })
    }

    /// 構築したレスポンスを破棄して初期化
    /// レスポンスを再設計したいときに
    pub fn cancel(self) -> Connection<C, R, W, NoneBody> {
        Connection {
            c: self.c,
            req: self.req,
            res: self.res.reset(),
            phantom: std::marker::PhantomData,
        }
    }
}