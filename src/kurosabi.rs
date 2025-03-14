use std::{pin::Pin, sync::Arc};
use log::{error, info, warn};
use tokio::io::{AsyncBufReadExt, BufReader};

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::error::HttpError;
use crate::{context::{Context, DefaultContext}, error::KurosabiError, request::Req, response::Res, router::{BoxedHandler, DefaultRouter, GenRouter}, server::{worker::{Worker}, KurosabiServerBuilder, TcpConnection}, utils::header::Method};

pub struct Kurosabi<C, R>
where
    C: Context + Clone + 'static,
    R: GenRouter<C, Arc<BoxedHandler>> + 'static,
{
    router: R,
    context: C
}

impl Kurosabi<DefaultContext<String>, DefaultRouter> {
    pub fn new() -> Kurosabi<DefaultContext<String>, DefaultRouter> {
        Kurosabi {
            router: DefaultRouter::new(),
            context: DefaultContext::new(),
        }
    }
}

impl<C, R> Kurosabi<C, R>
where
    C: Context + Clone + 'static,
    R: GenRouter<C, Arc<BoxedHandler>> + 'static,
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
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        let boxed_handler: Box<
            dyn Fn(
                Req,
                Res,
                Box<dyn Context>,
            ) -> Pin<Box<dyn std::future::Future<Output = Result<Res, HttpError>> + Send + 'static>>
                + Send
                + Sync,
        > = Box::new(move |req, res, ctx| Box::pin(handler(req, res, ctx)));
        self.router.regist(method, pattern, std::sync::Arc::new(boxed_handler));
    }

    pub fn get<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::GET, pattern, handler);
    }

    pub fn post<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::POST, pattern, handler);
    }

    pub fn put<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::PUT, pattern, handler);
    }

    pub fn delete<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::DELETE, pattern, handler);
    }

    pub fn patch<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::PATCH, pattern, handler);
    }

    pub fn options<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::OPTIONS, pattern, handler);
    }

    pub fn trace<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::TRACE, pattern, handler);
    }

    pub fn head<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::HEAD, pattern, handler);
    }

    pub fn connect<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
    {
        self.register_route(Method::CONNECT, pattern, handler);
    }

    pub fn any<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Req, Res, Box<dyn Context>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Res, HttpError>> + Send + 'static,
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
    C: Context + Clone + 'static,
    R: GenRouter<C, Arc<BoxedHandler>> + 'static,
{
    router: Arc<R>,
    context: Arc<C>,
}

impl<C, R> DefaultWorker<C, R>
where
    C: Context + Clone + 'static,
    R: GenRouter<C, Arc<BoxedHandler>> + 'static,
{
    pub fn new(router: Arc<R>, context: Arc<C>) -> DefaultWorker<C, R> {
        DefaultWorker {
            router,
            context,
        }
    }

    async fn http_reader_head(req: &mut Req) -> std::result::Result<(), KurosabiError> {
        let mut line_buf = String::with_capacity(1024);
        let mut connection = req.connection.lock().await;
        let mut reader = connection.reader();

        reader.read_line(&mut line_buf).await.map_err(KurosabiError::IoError)?;

        let parts: Vec<&str> = line_buf.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(KurosabiError::InvalidHttpHeader(line_buf));
        }

        req.method = Method::from_str(parts[0]).unwrap();
        req.path.path = parts[1].to_string();
        req.version = parts[2].to_string();

        loop {
            line_buf.clear();
            reader.read_line(&mut line_buf).await.map_err(KurosabiError::IoError)?;
            let trimmed = line_buf.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some((key, value)) = trimmed.split_once(": ") {
                req.header.set(key, value);
            } else {
                return Err(KurosabiError::InvalidHttpHeader(line_buf));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<C, R> Worker for DefaultWorker<C, R>
where
    C: Context + Clone,
    R: GenRouter<C, Arc<BoxedHandler>>,
{
    async fn execute(&self, connection: TcpConnection) {
        let conn = Arc::new(Mutex::new(connection));
        let mut req = Req::new(conn.clone());
        if let Err(_e) = Self::http_reader_head(&mut req).await {
            let e = HttpError::BadRequest("Invalid HTTP Header".to_string());
            error!("{:?}", e);
            let mut res = e.err_res();
            warn!("{}- \x1b[33m{}\x1b[0m\n", "Invalid HTTP Header", res.code);
            let mut locked_conn = conn.lock().await;
            let writer = locked_conn.writer();
            res.write_out_connection(writer).await.unwrap();
            return;
        }
        
        let head_info = format!("{} {} {} ", req.method.to_str(), req.path.path, req.version);
        
        let mut c: C = (*self.context).clone();
        if let Some(handler) = self.router.route(&mut req, &mut c) {
            let mut res = Res::new();
            res = handler(req, res, Box::new(c)).await.unwrap_or_else(|e| {
                error!("{:?}", e);
                e.err_res()
            });
            if res.code >= 500 {
                error!("{}- \x1b[31m{}\x1b[0m\n", head_info, res.code);
            } else if res.code >= 400 {
                warn!("{}- \x1b[33m{}\x1b[0m\n", head_info, res.code);
            } else {
                info!("{}- \x1b[32m{}\x1b[0m\n", head_info, res.code);
            }
            let mut locked_conn = conn.lock().await;
            let mut writer = locked_conn.writer();
            res.write_out_connection(&mut writer).await.unwrap();
        } else {
            let e = HttpError::NotFound;
            warn!("{:?}", e);
            let mut res = e.err_res();
            warn!("{}- \x1b[33m{}\x1b[0m\n", head_info, res.code);
            let mut locked_conn = conn.lock().await;
            let mut writer = locked_conn.writer();
            res.write_out_connection(&mut writer).await.unwrap();
        }
    }
}
