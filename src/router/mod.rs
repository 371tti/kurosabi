use futures::{AsyncRead, AsyncWrite};

use crate::{connection::{CompletedResponse, Connection, ConnectionState, ResponseReadyToSend}, error::Result, http::{method::HttpMethod, request::HttpRequest, response::HttpResponse}, router};

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
    pub async fn routing<R, W>(&self, reader: R, writer: W) -> Result<Connection<C, R, W, CompletedResponse>>
    where
        D: Router<C, R, W, ResponseReadyToSend>,
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let res = HttpResponse::new(writer);
        let req = HttpRequest::new(reader).await;
        if req.method() == &HttpMethod::ERR {
            let conn = Connection::new(
                self.context.clone(),
                req,
                res,
            );
            self.router.invalid_http(conn).await.flush().await
        } else {
            let conn = Connection::new(
                self.context.clone(),
                req,
                res,
            );
            self.router.router(conn).await.flush().await
        }
    }
}

#[derive(Clone, Default)]
pub struct DefaultContext {}