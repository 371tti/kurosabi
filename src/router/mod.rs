use std::time::Duration;

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::pin_mut;

use crate::{
    connection::{Connection, NoneBody, ResponseReadyToSend},
    error::{ErrorPare, RouterError},
    http::{request::HttpRequest, response::HttpResponse},
    utils::with_timeout,
};

pub trait Router<C, R, W, S>: Send + Sync + 'static
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    C: Send,
{
    fn router(
        &self,
        conn: Connection<C, R, W>,
    ) -> impl Future<Output = Connection<C, R, W, ResponseReadyToSend>> + Send;
    fn invalid_http(
        &self,
        conn: Connection<C, R, W>,
    ) -> impl Future<Output = Connection<C, R, W, ResponseReadyToSend>> + Send {
        async move { conn.text_body("HELLO") }
    }
}

pub const DEFAULT_KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_HTTP_HEADER_READ_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct KurosabiRouter<D, C: Clone + Send = DefaultContext> {
    context: C,
    router: D,
    keep_alive_timeout: Duration,
    http_header_read_timeout: Duration,
}

impl<D> KurosabiRouter<D, DefaultContext> {
    pub fn new(router: D) -> Self {
        Self {
            context: DefaultContext::default(),
            router,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_HTTP_HEADER_READ_TIMEOUT,
        }
    }
}

impl<D, C: Clone + Send> KurosabiRouter<D, C> {
    pub fn with_context(router: D, context: C) -> Self {
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

impl<D, C: Clone + Send> KurosabiRouter<D, C> {
    pub fn new_connection<R, W>(&self, reader: R, writer: W) -> Connection<C, R, W, NoneBody>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let req = HttpRequest::new(reader);
        let res = HttpResponse::new(writer);
        Connection::new(self.context.clone(), req, res)
    }
    pub async fn routing<R, W>(
        &self,
        connection: Connection<C, R, W, NoneBody>,
        keep_alive_timeout: Option<Duration>,
        http_header_read_timeout: Option<Duration>,
    ) -> RoutingResult<Connection<C, R, W, NoneBody>>
    where
        D: Router<C, R, W, ResponseReadyToSend>,
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let keep_alive_timeout = keep_alive_timeout.unwrap_or(self.keep_alive_timeout);
        let http_header_read_timeout =
            http_header_read_timeout.unwrap_or(self.http_header_read_timeout);
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
                }
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
                }
            },
            Err(_) => return RoutingResult::Close(RouterError::Timeout),
        };
        let conn = Connection::new(self.context.clone(), req, res);
        match self.router.router(conn).await.flush().await {
            Ok(conn) => RoutingResult::Continue(conn),
            Err(e) => RoutingResult::CloseHaveConnection(e),
        }
    }
}

pub enum RoutingResult<T> 
{
    Continue(T),
    CloseHaveConnection(ErrorPare<T>),
    Close(RouterError),
}

#[derive(Clone, Default)]
pub struct DefaultContext {}
