use futures::{AsyncRead, AsyncWrite};

use crate::{connection::Connection, error::RouterError, http::{request::{HttpRequest, NotParsedHttpRequest}, response::HttpResponse}};

pub trait Router<C, R, W>: Send + Sync + 'static
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    type Fut: Future<Output = Connection<C, R, W>> + Send;

    fn router(&self, conn: Connection<C, R, W>) -> Self::Fut;
    fn invalid_http(&self, conn: Connection<C, R, W>) -> Self::Fut;
}

pub trait AsyncInit {
    async fn init() -> Self;
}

pub struct KurosabiRouter<D, C: Clone = DefaultContext> {
    context: C,
    router: D,
}

impl<D: AsyncInit, C: Clone> KurosabiRouter<D, C> {
    pub async fn new(context: C) -> Self {
        Self { context, router: D::init().await }
    }
}

impl<D, C: Clone> KurosabiRouter<D, C> {
    pub fn with_context(router: D, context: C) -> Self {
        KurosabiRouter { context, router }
    }
}

impl<D, C: Clone> KurosabiRouter<D, C>
where
    D: Router<C, R, W>,
{
    pub async fn routing<R, W>(&self, reader: R, writer: W) -> Connection<C, R, W>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let res = HttpResponse::new(writer);
        let not_parsed_req = NotParsedHttpRequest::new(reader);
        let req = HttpRequest::new(reader).await;
        if req.method() == &crate::http::request::HttpMethod::ERR {
            let conn = Connection {
                c: self.context.clone(),
                req: not_parsed_req,
                res,
            };
            self.router.invalid_http(conn).await
        } else {
            let conn = Connection {
                c: self.context.clone(),
                req,
                res,
            };
            self.router.router(conn).await
        }
    }
}

#[derive(Clone, Default)]
pub struct DefaultContext {}

impl AsyncInit for DefaultContext {
    async fn init() -> Self {
        DefaultContext {}
    }
}

