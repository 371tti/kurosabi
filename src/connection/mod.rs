use std::borrow::Borrow;

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::{AsyncReadExt, AsyncWriteExt, future::join};

use crate::{
    error::{ConnectionResult, ErrorPare, RouterError},
    http::{code::HttpStatusCode, request::HttpRequest, response::HttpResponse}, utils::{write_all_vectored3, write_hex_crlf},
};

/// Connection struct
/// one http connection per one instance
pub struct Connection<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static, S: ConnectionState = NoneBody>
{
    pub c: C,
    pub req: HttpRequest<R>,
    pub res: HttpResponse<W>,
    /// State
    pub phantom: std::marker::PhantomData<S>,
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

pub const STREAM_CHUNK_SIZE: usize = 1024 * 32; // 32KB

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static, S: ConnectionState> Connection<C, R, W, S> {
    #[inline(always)]
    pub fn path_seg_iter<'a>(&'a self) -> std::str::Split<'a, char> {
        self.req.path_full()[1..].split('/')
    }

    #[inline(always)]
    pub fn path_segs<'a>(&'a self) -> Box<[&'a str]> {
        self.path_seg_iter().collect::<Box<[_]>>()
    }
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
    #[inline]
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

    #[inline]
    pub fn add_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.res.header_add(key, value);
        self
    }

    #[inline]
    pub fn remove_header<S>(mut self, key: S) -> Self
    where
        S: std::borrow::Borrow<str>,
    {
        self.res.header_remove(key);
        self
    }

    #[inline]
    pub fn text_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).text_body(body)
    }

    #[inline]
    pub fn binary_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).binary_body(body)
    }

    #[inline]
    pub fn html_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).html_body(body)
    }

    #[inline]
    pub fn json_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).json_body(body)
    }

    #[inline]
    pub fn xml_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).xml_body(body)
    }

    #[inline]
    pub fn csv_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).csv_body(body)
    }

    #[inline]
    pub fn css_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).css_body(body)
    }

    #[inline]
    pub fn js_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).js_body(body)
    }

    #[inline]
    pub fn png_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).png_body(body)
    }

    #[inline]
    pub fn jpg_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).jpg_body(body)
    }

    #[inline]
    pub fn gif_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).gif_body(body)
    }

    #[inline]
    pub fn svg_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).svg_body(body)
    }

    #[inline]
    pub fn pdf_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).pdf_body(body)
    }

    #[inline]
    pub fn xml_body_bytes(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK)
            .xml_body_bytes(body)
    }

    #[inline]
    pub fn json_body_bytes(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK)
            .json_body_bytes(body)
    }

    #[inline]
    #[cfg(feature = "json")]
    pub fn json_body_serialized<T>(
        self,
        body: &T,
    ) -> Result<Connection<C, R, W, ResponseReadyToSend>, JsonSerErrorPare<Connection<C, R, W, NoneBody>>>
    where
        T: serde::Serialize,
    {
        match self
            .set_status_code(HttpStatusCode::OK)
            .json_body_serialized(body)
        {
            Ok(conn) => Ok(conn),
            Err(e) => Err(JsonSerErrorPare {
                serde_error: e.serde_error,
                connection: Connection {
                    c: e.connection.c,
                    req: e.connection.req,
                    res: e.connection.res,
                    phantom: std::marker::PhantomData,
                },
            }),
        }
    }

    #[inline]
    pub fn no_body(self) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).no_body()
    }
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, StatusSetNoneBody> {
    #[inline]
    pub fn set_status_code<T>(mut self, status_code: T) -> Self
    where
        T: Into<u16>,
    {
        self.res.set_status_code(status_code);
        self
    }

    #[inline]
    pub fn add_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.res.header_add(key, value);
        self
    }

    #[inline]
    pub fn remove_header<S>(mut self, key: S) -> Self
    where
        S: std::borrow::Borrow<str>,
    {
        self.res.header_remove(key);
        self
    }

    #[inline]
    pub fn text_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.text_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn binary_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.binary_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn html_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.html_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn json_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.json_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn xml_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.xml_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn csv_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.csv_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn css_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.css_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn js_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.js_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn png_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.png_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn jpg_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.jpg_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn gif_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.gif_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn svg_body<T>(mut self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.res.svg_body(body.borrow());
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn pdf_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.pdf_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn xml_body_bytes(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.xml_body_bytes(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn json_body_bytes(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.json_body_bytes(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn csv_body_bytes(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.csv_body_bytes(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    #[cfg(feature = "json")]
    pub fn json_body_serialized<T>(
        mut self,
        body: &T,
    ) -> Result<Connection<C, R, W, ResponseReadyToSend>, JsonSerErrorPare<Connection<C, R, W, StatusSetNoneBody>>>
    where
        T: serde::Serialize,
    {
        let serialized = match serde_json::to_string(body) {
            Ok(s) => s,
            Err(e) => {
                let conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(JsonSerErrorPare { serde_error: e, connection: conn });
            },
        };
        self.res.json_body(&serialized);
        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        })
    }

    #[inline]
    #[deprecated(
        note = "変更される可能性があるため、実務での使用は避けてください。代わりにstreamingメソッドを使用してください。"
    )]
    #[cfg(feature = "tokio-server")]
    pub async fn file_body<P>(mut self, file_path: P) -> ConnectionResult<Connection<C, R, W, ResponseReadyToSend>>
    where
        P: AsRef<std::path::Path>,
    {
        use std::ops::Range;
        use tokio::io::AsyncSeekExt;

        let range: Option<Range<usize>> = self.req.header_get("Range").await.and_then(|r| {
            crate::http::request::parse_range_header(r)
        });
        let mut file = match tokio::fs::File::open(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                let conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(ErrorPare {
                    router_error: RouterError::IoError(e),
                    connection: conn,
                });
            },
        };
        let metadata = match file.metadata().await {
            Ok(m) => Some(m),
            Err(_) => None,
        };
        let start = range.as_ref().map_or(0, |r| r.start as u64);
        let end = range.as_ref().map_or_else(
            || metadata.as_ref().map_or(0, |m| m.len()),
            |r| r.end as u64,
        );
        let size = end - start;
        let reader = file.seek(std::io::SeekFrom::Start(start)).await.and_then(|_| {
            use tokio::io::AsyncReadExt;
            use tokio_util::compat::TokioAsyncReadCompatExt;
            Ok(file.take(size).compat())
        });
        match reader {
            Ok(r) => {
                if range.is_some() {
                    self.res.set_status_code(HttpStatusCode::PartialContent);
                    let content_range = format!("bytes {}-{}/{}", start, end - 1, metadata.as_ref().map_or("*".to_string(), |m| m.len().to_string()));
                    self.res.header_add("Content-Range", content_range);
                    self.streaming_unchunked(r, size as usize).await
                } else {
                    self.res.set_status_code(HttpStatusCode::OK);
                    self.streaming_unchunked(r, size as usize).await
                }
            }
            Err(e) => {
                let conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(ErrorPare {
                    router_error: RouterError::IoError(e),
                    connection: conn,
                });
            },
        }
    }

    #[inline]
    pub fn no_body(mut self) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.header_add("Content-Length", "0");
        self.res.response_line_write();
        self.res.start_content();
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub async fn streaming<T>(self, reader: T, size: Option<usize>) -> ConnectionResult<Connection<C, R, W, ResponseReadyToSend>>
    where
        T: AsyncRead + Unpin + 'static,
    {
        if let Some(size) = size {
            self.streaming_unchunked(reader, size).await
        } else {
            self.streaming_chunked(reader).await
        }
    }

    #[inline]
    pub async fn streaming_unchunked<T>(mut self, mut reader: T, size: usize) -> ConnectionResult<Connection<C, R, W, ResponseReadyToSend>>
    where
        T: AsyncRead + Unpin + 'static,
    {
        self.res.header_add("Content-Length", size.to_string());
        self.res.response_line_write();
        self.res.start_content();
        if let Err(e) = self.res.send().await {
            let response_ready_conn = Connection {
                c: self.c,
                req: self.req,
                res: self.res,
                phantom: core::marker::PhantomData,
            };
            return Err(ErrorPare {
                router_error: RouterError::IoError(e),
                connection: response_ready_conn,
            });
        }

        let mut buf0 = [0u8; STREAM_CHUNK_SIZE];
        let mut buf1 = [0u8; STREAM_CHUNK_SIZE];

        let stream_res: std::io::Result<()> = 'brk: {
            let writer = self.res.writer();

            let mut cur_n = match reader.read(&mut buf0).await {
                Ok(n) => n,
                Err(e) => break 'brk Err(e),
            };
            let mut cur = &mut buf0;
            let mut nxt = &mut buf1;

            loop {
                if cur_n == 0 {
                    break Ok(());
                }

                let write_fut = writer.write_all(&cur[..cur_n]);
                let read_fut  = reader.read(nxt);
                
                // write and read in parallel
                let (write_res, read_res) = join(write_fut, read_fut).await;

                if let Err(e) = write_res {
                    break Err(e);
                }
                let nxt_n = match read_res {
                    Ok(n) => n,
                    Err(e) => break Err(e),
                };

                core::mem::swap(&mut cur, &mut nxt);
                cur_n = nxt_n;
            }
        };
        
        if let Err(e) = stream_res {
            let response_ready_conn = Connection {
                c: self.c,
                req: self.req,
                res: self.res,
                phantom: core::marker::PhantomData,
            };
            return Err(ErrorPare {
                router_error: RouterError::IoError(e),
                connection: response_ready_conn,
            });
        }

        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        })
    }

    #[inline]
    pub async fn streaming_chunked<T>(
        mut self,
        mut reader: T,
    ) -> ConnectionResult<Connection<C, R, W, ResponseReadyToSend>>
    where
        T: AsyncRead + Unpin + 'static,
    {
        self.res.header_add("Transfer-Encoding", "chunked");
        self.res.response_line_write();
        self.res.start_content();

        if let Err(e) = self.res.send().await {
            let response_ready_conn = Connection {
                c: self.c,
                req: self.req,
                res: self.res,
                phantom: core::marker::PhantomData,
            };
            return Err(ErrorPare {
                router_error: RouterError::IoError(e),
                connection: response_ready_conn,
            });
        }

        let mut buf0 = [0u8; STREAM_CHUNK_SIZE];
        let mut buf1 = [0u8; STREAM_CHUNK_SIZE];
        let mut hexline = [0u8; 32];

        let stream_res: std::io::Result<()> = 'brk: {
            let mut writer = self.res.writer();

            let mut cur_n = match reader.read(&mut buf0).await {
                Ok(n) => n,
                Err(e) => break 'brk Err(e),
            };
            let mut cur = &mut buf0;
            let mut nxt = &mut buf1;

            loop {
                if cur_n == 0 {
                    break writer.write_all(b"0\r\n\r\n").await;
                }

                let head = write_hex_crlf(cur_n, &mut hexline);

                let write_fut = write_all_vectored3(&mut writer, head, &cur[..cur_n], b"\r\n");
                let read_fut  = reader.read(nxt);
                
                // write and read in parallel
                let (write_res, read_res) = join(write_fut, read_fut).await;

                if let Err(e) = write_res {
                    break Err(e);
                }
                let nxt_n = match read_res {
                    Ok(n) => n,
                    Err(e) => break Err(e),
                };

                core::mem::swap(&mut cur, &mut nxt);
                cur_n = nxt_n;
            }
        };

        if let Err(e) = stream_res {
            let response_ready_conn = Connection {
                c: self.c,
                req: self.req,
                res: self.res,
                phantom: core::marker::PhantomData,
            };
            return Err(ErrorPare {
                router_error: RouterError::IoError(e),
                connection: response_ready_conn,
            });
        }

        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: core::marker::PhantomData,
        })
    }

    #[inline]
    pub async fn ready_chunked(mut self) -> ConnectionResult<Connection<C, R, W, ChunkedResponse>> {
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
            },
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
    #[inline]
    pub async fn send_chunk(&mut self, chunk: &[u8]) -> std::io::Result<()> {
        let chunk_size_hex = format!("{:X}\r\n", chunk.len());
        self.res
            .writer()
            .write_all(chunk_size_hex.as_bytes())
            .await?;
        self.res.writer().write_all(chunk).await?;
        self.res.writer().write_all(b"\r\n").await
    }

    #[inline]
    pub async fn send_last_chunk(&mut self) -> std::io::Result<()> {
        self.res.writer().write_all(b"0\r\n\r\n").await
    }

    #[inline]
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
    #[inline(always)]
    pub(crate) async fn flush(mut self) -> ConnectionResult<Connection<C, R, W, NoneBody>> {
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
            },
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

#[cfg(feature = "json")]
pub struct JsonSerErrorPare<T> {
    pub serde_error: serde_json::Error,
    pub connection: T,
}

pub struct FileContentBuilder {
    path: std::path::PathBuf,
    content_type: ContentType,
    content_range: ContentRange,
    content_disposition: ContentDisposition,
}

enum ContentType {
    Guess,
    Custom(String),
    TextPlain,
    TextHtml,
    ApplicationJson,
    ApplicationXml,
    TextCsv,
    TextCss,
    ApplicationJavascript,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageSvgXml,
    ApplicationPdf,
    VideoMp2t,
    VideoMp4,
    VideoWebm,
    VideoOgg,
}

enum ContentRange {
    Auto,
    StartEnd(u64, u64),
    Start(u64),
    End(u64),
    Unsatisfiable,
}

enum ContentDisposition {
    Inline,
    Attachment,
    AttachmentWithFilename(String),
}

