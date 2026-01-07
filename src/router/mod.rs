
use futures::{AsyncRead, AsyncWrite, pin_mut};

use crate::{connection::{Connection, ConnectionState, NoneBody, ResponseReadyToSend}, error::{ConnectionResult, RouterError}, http::{request::HttpRequest, response::HttpResponse}, utils::with_timeout};

pub trait Router<C, R, W, S>: Send + Sync + 'static
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    S: ConnectionState,
    C: Send
{
    fn router(&self, conn: Connection<C, R, W>) -> impl Future<Output = Connection<C, R, W, ResponseReadyToSend>> + Send;
    fn invalid_http(&self, conn: Connection<C, R, W>) -> impl Future<Output = Connection<C, R, W, ResponseReadyToSend>> + Send {
        async move { conn.text_body("HELLO") }
    }
}

#[derive(Clone)]
pub struct KurosabiRouter<D, C: Clone + Send = DefaultContext> {
    context: C,
    router: D,
}

impl<D> KurosabiRouter<D, DefaultContext> {
    pub fn new(router: D) -> Self {
        Self { context: DefaultContext::default(), router }
    }
}

impl<D, C: Clone + Send> KurosabiRouter<D, C> {
    pub fn with_context(router: D, context: C) -> Self {
        KurosabiRouter { context, router }
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
    pub async fn routing<R, W>(&self, connection: Connection<C, R, W, NoneBody>) -> Result<ConnectionResult<Connection<C, R, W, NoneBody>>, RouterError>
    where
        D: Router<C, R, W, ResponseReadyToSend>,
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let Connection { c, req, res, .. } = connection;
        let res = res.reset();
        let reader = req.into_reader();
        let new_req = HttpRequest::new(reader);
        let new_req_fut = new_req.parse_request_line();
        pin_mut!(new_req_fut);
        let req_uf = match with_timeout(new_req_fut, std::time::Duration::from_secs(5)).await {
            Ok(req) => match req {
                Ok(r) => r,
                Err(req_err) => {
                    let conn = Connection::new(
                        c,
                        req_err,
                        res,
                    );
                    return Ok(self.router.invalid_http(conn).await.flush().await)
                }
            },
            Err(_) => {
                return Err(RouterError::KeepAliveTimeout)
            }
        };
        let req_fut = req_uf.parse_request();
        pin_mut!(req_fut);
        let req = match with_timeout(req_fut, std::time::Duration::from_secs(5)).await {
            Ok(r) => match r {
                Ok(req) => req,
                Err(r_err) => {
                    let conn = Connection::new(
                        self.context.clone(),
                        r_err,
                        res,
                    );
                    return Ok(self.router.invalid_http(conn).await.flush().await)
                }
            },
            Err(_) => {
                return Err(RouterError::Timeout)
            }
        };
        let conn = Connection::new(
            self.context.clone(),
            req,
            res,
        );
        Ok(self.router.router(conn).await.flush().await)
    }
}

#[derive(Clone, Default)]
pub struct DefaultContext {}