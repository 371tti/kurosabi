use std::time::Duration;

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::pin_mut;
use log::debug;

use crate::{
    connection::{Connection, ConnectionState, NoneBody, ResponseReadyToSend},
    error::{ErrorPare, RouterError},
    http::{code::HttpStatusCode, request::HttpRequest, response::HttpResponse},
    utils::with_timeout,
};

pub trait Router<C, R, W, S>: Sync
where
    R: AsyncRead + Unpin + 'static,
    W: AsyncWrite + Unpin + 'static,
{
    fn router(&self, conn: Connection<C, R, W>) -> impl Future<Output = Connection<C, R, W, ResponseReadyToSend>>;
    #[inline(always)]
    fn invalid_http(
        &self,
        conn: Connection<C, R, W>,
    ) -> impl Future<Output = Connection<C, R, W, ResponseReadyToSend>> {
        async move {
            conn.set_status_code(HttpStatusCode::BadRequest)
                .text_body("Invalid HTTP request")
        }
    }
}

pub const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_HTTP_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct KurosabiRouter<D, C: Clone + Sync = DefaultContext> {
    context: C,
    router: D,
    keep_alive_timeout: Duration,
    http_header_read_timeout: Duration,
}

impl<D: Default> KurosabiRouter<D, DefaultContext> {
    pub fn new() -> Self {
        Self {
            context: DefaultContext::default(),
            router: D::default(),
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_HTTP_HEADER_READ_TIMEOUT,
        }
    }

    pub fn with_router(router: D) -> Self {
        Self {
            context: DefaultContext::default(),
            router,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_HTTP_HEADER_READ_TIMEOUT,
        }
    }
}

impl<D, C: Clone + Sync> KurosabiRouter<D, C> {
    pub fn with_context(context: C) -> Self
    where
        D: Default,
    {
        KurosabiRouter {
            context,
            router: D::default(),
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_HTTP_HEADER_READ_TIMEOUT,
        }
    }

    pub fn with_context_and_router(router: D, context: C) -> Self {
        KurosabiRouter {
            context,
            router,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_HTTP_HEADER_READ_TIMEOUT,
        }
    }

    pub fn set_keep_alive_timeout(&mut self, duration: Duration) {
        self.keep_alive_timeout = duration;
    }

    pub fn set_http_header_read_timeout(&mut self, duration: Duration) {
        self.http_header_read_timeout = duration;
    }
}

impl<D, C: Clone + Sync> KurosabiRouter<D, C> {
    #[inline(always)]
    pub fn new_connection<R, W>(&self, reader: R, writer: W) -> Connection<C, R, W, NoneBody>
    where
        R: AsyncRead + Unpin + 'static,
        W: AsyncWrite + Unpin + 'static,
    {
        let req = HttpRequest::new(reader);
        let res = HttpResponse::new(writer);
        Connection::new(self.context.clone(), req, res)
    }

    #[inline(always)]
    pub async fn routing<R, W>(
        &self,
        connection: Connection<C, R, W, NoneBody>,
        keep_alive_timeout: Option<Duration>,
        http_header_read_timeout: Option<Duration>,
    ) -> RoutingResult<Connection<C, R, W, NoneBody>>
    where
        D: Router<C, R, W, ResponseReadyToSend>,
        R: AsyncRead + Unpin + 'static,
        W: AsyncWrite + Unpin + 'static,
    {
        let keep_alive_timeout = keep_alive_timeout.unwrap_or(self.keep_alive_timeout);
        let http_header_read_timeout = http_header_read_timeout.unwrap_or(self.http_header_read_timeout);
        let Connection { c, req, res, .. } = connection;
        let res = res.reset();
        let reader = req.into_reader();
        let new_req = HttpRequest::new(reader);
        let new_req_fut = new_req.parse_request_line();
        pin_mut!(new_req_fut);
        let req_uf = match with_timeout(new_req_fut, keep_alive_timeout).await {
            Ok(req) => match req {
                Ok(r) => r,
                Err(req_err) => {
                    let conn = Connection::new(c, req_err, res);
                    match self.router.invalid_http(conn).await.flush().await {
                        Ok(conn) => return RoutingResult::Continue(conn),
                        Err(e) => return RoutingResult::CloseHaveConnection(e),
                    }
                },
            },
            Err(_) => return RoutingResult::Close(RouterError::KeepAliveTimeout),
        };
        let req_fut = req_uf.parse_request();
        pin_mut!(req_fut);
        let req = match with_timeout(req_fut, http_header_read_timeout).await {
            Ok(r) => match r {
                Ok(req) => req,
                Err(r_err) => {
                    let conn = Connection::new(self.context.clone(), r_err, res);
                    match self.router.invalid_http(conn).await.flush().await {
                        Ok(conn) => return RoutingResult::Continue(conn),
                        Err(e) => return RoutingResult::CloseHaveConnection(e),
                    }
                },
            },
            Err(_) => return RoutingResult::Close(RouterError::Timeout),
        };
        let conn = Connection::new(self.context.clone(), req, res);
        match self.router.router(conn).await.flush().await {
            Ok(conn) => RoutingResult::Continue(conn),
            Err(e) => RoutingResult::CloseHaveConnection(e),
        }
    }

    #[inline(always)]
    pub async fn new_connection_loop<R, W>(&self, reader: R, writer: W)
    where
        D: Router<C, R, W, ResponseReadyToSend>,
        R: AsyncRead + Unpin + 'static,
        W: AsyncWrite + Unpin + 'static,
    {
        let mut conn = self.new_connection(reader, writer);
        loop {
            conn = match self.routing(conn, None, None).await {
                RoutingResult::Continue(c) => {
                    #[cfg(feature = "logging")]
                    http_log(&c);
                    c
                }
                RoutingResult::Close(c) => {
                    #[cfg(feature = "logging")]
                    debug!("Connection closed: {:?}", c);
                    break;
                },
                RoutingResult::CloseHaveConnection(p) => {
                    #[cfg(feature = "logging")]
                    http_log(&p.connection);
                    #[cfg(feature = "logging")]
                    debug!("Connection closed: {:?}", p.router_error);
                    break;
                },
            };
        }
    }
}

pub enum RoutingResult<T> {
    Continue(T),
    CloseHaveConnection(ErrorPare<T>),
    Close(RouterError),
}

#[derive(Clone, Default)]
pub struct DefaultContext {}

#[cfg(feature = "logging")]
fn http_log<C, R, W, S>(c: &Connection<C, R, W, S>)
where 
    R: AsyncRead + Unpin + 'static,
    W: AsyncWrite + Unpin + 'static,
    S: ConnectionState,
{
    use log::info;

    let method = c.req.method();
    let version = c.req.version().as_str();
    let status = c.res.status_code();
    let path = c.req.path_full();
    let req_log = match method {
        crate::http::HttpMethod::GET => format!("\x1b[1;32m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),      // 太字緑
        crate::http::HttpMethod::POST => format!("\x1b[1;35m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),     // 太字桃
        crate::http::HttpMethod::PUT => format!("\x1b[1;33m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),      // 太字黄
        crate::http::HttpMethod::DELETE => format!("\x1b[1;31m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),   // 太字赤
        crate::http::HttpMethod::HEAD => format!("\x1b[1;36m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),     // 太字シアン
        crate::http::HttpMethod::OPTIONS => format!("\x1b[1;35m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),  // 太字紫
        crate::http::HttpMethod::PATCH => format!("\x1b[1;37m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),    // 太字白
        crate::http::HttpMethod::TRACE => format!("\x1b[1;90m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),    // 太字グレー
        crate::http::HttpMethod::CONNECT => format!("\x1b[1;94m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),  // 太字明るい青
        crate::http::HttpMethod::ERR => format!("\x1b[1;41m{}\x1b[0m \x1b[37m{}\x1b[0m \x1b[37m{}\x1b[0m", method.as_str(), path, version),      // 太字赤背景
    };
    let res_log = match status.info().code {
        100..200 => format!("\x1b[1;37m{}\x1b[0m \x1b[37m{}\x1b[0m", status.as_str(), status.info().message),    // 太字白 + 白
        200..300 => format!("\x1b[1;32m{}\x1b[0m \x1b[32m{}\x1b[0m", status.as_str(), status.info().message),    // 太字緑 + 緑
        300..400 => format!("\x1b[1;34m{}\x1b[0m \x1b[34m{}\x1b[0m", status.as_str(), status.info().message),    // 太字青 + 青
        400..500 => format!("\x1b[1;33m{}\x1b[0m \x1b[33m{}\x1b[0m", status.as_str(), status.info().message),    // 太字黄 + 黄
        500..600 => format!("\x1b[1;31m{}\x1b[0m \x1b[31m{}\x1b[0m", status.as_str(), status.info().message),    // 太字赤 + 赤
        _ => format!("\x1b[1;35m{}\x1b[0m \x1b[35m{}\x1b[0m", status.as_str(), status.info().message),           // 太字紫 + 紫
    };
    info!("{} \x1b[1;97m->\x1b[0m {}", req_log, res_log);
}