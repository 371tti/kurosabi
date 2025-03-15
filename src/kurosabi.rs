use std::sync::Mutex;
use std::{pin::Pin, sync::Arc};
use log::{error, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::error::HttpError;
use crate::{context::DefaultContext, request::Req, response::Res, router::{BoxedHandler, DefaultRouter, GenRouter}, server::{worker::{Worker}, KurosabiServerBuilder, TcpConnection}, utils::header::Method};

pub struct Kurosabi<C, R>
where
    C: Clone + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    router: R,
    context: C
}

impl Kurosabi<DefaultContext<String>, DefaultRouter<DefaultContext<String>>> {
    pub fn new() -> Kurosabi<DefaultContext<String>, DefaultRouter<DefaultContext<String>>> {
        Kurosabi {
            router: DefaultRouter::new(),
            context: DefaultContext::new(),
        }
    }
}

impl<C, R> Kurosabi<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    pub fn with_context(router: R, context: C) -> Kurosabi<C, R> {
        Kurosabi {
            router,
            context,
        }
    }

    #[inline]
    fn register_route<F, Fut>(&mut self, method: Method, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        let boxed_handler: Box<
            dyn Fn(
                Context<C>,
            ) -> Pin<Box<dyn std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static>>
                + Send
                + Sync,
        > = Box::new(move |c| Box::pin(handler(c)));
        self.router.regist(method, pattern, std::sync::Arc::new(boxed_handler));
    }

    pub fn get<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::GET, pattern, handler);
    }

    pub fn post<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::POST, pattern, handler);
    }

    pub fn put<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::PUT, pattern, handler);
    }

    pub fn delete<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::DELETE, pattern, handler);
    }

    pub fn patch<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::PATCH, pattern, handler);
    }

    pub fn options<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::OPTIONS, pattern, handler);
    }

    pub fn trace<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::TRACE, pattern, handler);
    }

    pub fn head<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::HEAD, pattern, handler);
    }

    pub fn connect<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::CONNECT, pattern, handler);
    }

    pub fn any<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send + 'static,
    {
        self.register_route(Method::UNKNOWN("OTHER".to_string()), pattern, handler);
    }



    pub fn server(mut self) -> KurosabiServerBuilder<DefaultWorker<C, R>> {
        self.router.build();
        let worker = std::sync::Arc::new(DefaultWorker::new(std::sync::Arc::new(self.router), std::sync::Arc::new(self.context)));
        KurosabiServerBuilder::new(worker)
    }
}

pub struct DefaultWorker<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    router: Arc<R>,
    context: Arc<C>,
}

impl<C, R> DefaultWorker<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    pub fn new(router: Arc<R>, context: Arc<C>) -> DefaultWorker<C, R> {
        DefaultWorker {
            router,
            context,
        }
    }
}

pub struct Context<C> {
    pub req: Req,
    pub res: Res,
    pub ctx: Box<C>,
}

#[async_trait::async_trait]
impl<C, R> Worker for DefaultWorker<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>>,
{
    async fn execute(&self, connection: TcpConnection) {
        let mut req = Req::new(connection);
        req.parse_headers().await.unwrap_or_else(|e| {
            error!("{:?}", e);
        });
        
        let method = req.method.to_str();
        let path = req.path.path.clone();
        let version = req.version.clone();
        let head_info = format!("{} {} {} ", method, path, version);
        
        let c: C = (*self.context).clone();
        if let Some(handler) = self.router.route(&mut req) {
            let res = Res::new();

            let mut c = Context {
                req,
                res,
                ctx: Box::new(c),
            };

            let mut err = HttpError::InternalServerError;
            c = match handler(c).await {
                Ok(c) => c,
                Err((mut c,e)) => {
                    err = e;
                    c.res = err.err_res();
                    c
                }
            };
            if c.res.code >= 500 {
                error!("{}- \x1b[31m{}\x1b[0m\n{}", head_info, c.res.code, err);
            } else if c.res.code >= 400 {
                warn!("{}- \x1b[33m{}\x1b[0m\n{}", head_info, c.res.code, err);
            } else {
                info!("{}- \x1b[32m{}\x1b[0m", head_info, c.res.code);
            }
            c.res.flush(&mut c.req).await.unwrap();
        } else {
            let e = HttpError::NotFound;
            let mut res = e.err_res();
            res.flush(&mut req).await.unwrap();
            warn!("{}- \x1b[33m{}\x1b[0m\n{}",head_info, res.code, e);
        }
    }
}