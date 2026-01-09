use std::borrow::Borrow;

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::{AsyncReadExt, AsyncWriteExt};

use crate::{
    error::{ConnectionResult, ErrorPare, RouterError},
    http::{code::HttpStatusCode, request::HttpRequest, response::HttpResponse},
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

pub const STREAM_CHUNK_SIZE: usize = 4096;

pub trait SizedAsyncRead: AsyncRead + Unpin + 'static {
    fn size(&self) -> usize;
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static, S: ConnectionState> Connection<C, R, W, S> {
    pub fn path_seg_iter<'a>(&'a self) -> PathSegmentIterator<'a> {
        PathSegmentIterator::new(self.req.path_full())
    }

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

    pub fn add_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.res.header_add(key, value);
        self
    }

    pub fn remove_header<S>(mut self, key: S) -> Self
    where
        S: std::borrow::Borrow<str>,
    {
        self.res.header_remove(key);
        self
    }

    pub fn text_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).text_body(body)
    }

    pub fn binary_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).binary_body(body)
    }

    pub fn html_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).html_body(body)
    }

    pub fn json_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).json_body(body)
    }

    pub fn xml_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).xml_body(body)
    }

    pub fn csv_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).csv_body(body)
    }

    pub fn css_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).css_body(body)
    }

    pub fn js_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).js_body(body)
    }

    pub fn png_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).png_body(body)
    }

    pub fn jpg_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).jpg_body(body)
    }

    pub fn gif_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).gif_body(body)
    }

    pub fn svg_body<T>(self, body: T) -> Connection<C, R, W, ResponseReadyToSend>
    where
        T: Borrow<str> + Sized,
    {
        self.set_status_code(HttpStatusCode::OK).svg_body(body)
    }

    pub fn pdf_body(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).pdf_body(body)
    }

    pub fn xml_body_bytes(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).xml_body_bytes(body)
    }

    pub fn json_body_bytes(self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).json_body_bytes(body)
    }

    #[cfg(feature = "json")]
    pub fn json_body_serialized<T>(self, body: &T) -> Result<Connection<C, R, W, ResponseReadyToSend>, JsonSerErrorPare<Connection<C, R, W, NoneBody>>>
    where
        T: serde::Serialize,
    {
        match self.set_status_code(HttpStatusCode::OK).json_body_serialized(body) {
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

    pub fn no_body(self) -> Connection<C, R, W, ResponseReadyToSend> {
        self.set_status_code(HttpStatusCode::OK).no_body()
    }
}

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, StatusSetNoneBody> {
    pub fn set_status_code<T>(mut self, status_code: T) -> Self
    where
        T: Into<u16>,
    {
        self.res.set_status_code(status_code);
        self
    }

    pub fn add_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.res.header_add(key, value);
        self
    }

    pub fn remove_header<S>(mut self, key: S) -> Self
    where
        S: std::borrow::Borrow<str>,
    {
        self.res.header_remove(key);
        self
    }

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

    pub fn binary_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.binary_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

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

    pub fn png_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.png_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn jpg_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.jpg_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn gif_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.gif_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

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

    pub fn pdf_body(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.pdf_body(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn xml_body_bytes(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.xml_body_bytes(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn json_body_bytes(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.json_body_bytes(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn csv_body_bytes(mut self, body: &[u8]) -> Connection<C, R, W, ResponseReadyToSend> {
        self.res.csv_body_bytes(body);
        Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "json")]
    pub fn json_body_serialized<T>(mut self, body: &T) -> Result<Connection<C, R, W, ResponseReadyToSend>, JsonSerErrorPare<Connection<C, R, W, StatusSetNoneBody>>>
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
                return Err(JsonSerErrorPare {
                    serde_error: e,
                    connection: conn,
                });
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

    pub async fn streaming<T>(mut self, mut reader: T) -> ConnectionResult<Connection<C, R, W, ResponseReadyToSend>>
    where
        T: SizedAsyncRead,
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
                    Err(e) => break Err(e),
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
            },
        };

        Ok(Connection {
            c: self.c,
            req: self.req,
            res: self.res,
            phantom: std::marker::PhantomData,
        })
    }

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
    pub async fn send_chunk(&mut self, chunk: &[u8]) -> std::io::Result<()> {
        let chunk_size_hex = format!("{:X}\r\n", chunk.len());
        self.res
            .writer()
            .write_all(chunk_size_hex.as_bytes())
            .await?;
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

pub struct PathSegmentIterator<'a> {
    segs: std::str::Split<'a, char>,
    next: Option<&'a str>,
}

impl PathSegmentIterator<'_> {
    pub fn new<'a>(path: &'a str) -> PathSegmentIterator<'a> {
        let mut split = path[1..].split('/');
        let next = split.next();
        PathSegmentIterator { segs: split, next }
    }
}

impl<'a> Iterator for PathSegmentIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            Some(seg) => {
                self.next = self.segs.next();
                if self.next.is_none() { Some(seg) } else { Some(seg) }
            },
            None => None,
        }
    }
}

#[cfg(feature = "json")]
pub struct JsonSerErrorPare<T> {
    pub serde_error: serde_json::Error,
    pub connection: T,
}