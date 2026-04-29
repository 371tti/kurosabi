use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

#[cfg(feature = "logging")]
use log::{debug, info};
use tokio::{
    net::{
        TcpSocket,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Semaphore,
};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{
    connection::{Connection, NoneBody, ResponseReadyToSend},
    http::HttpMethod,
    router::{DEFAULT_KEEP_ALIVE_TIMEOUT, DefaultContext, KurosabiRouter, Router},
    server::{DEFAULT_LIMIT_HANDLE_NUM, DEFAULT_TCP_BACKLOG},
};

pub type Conn<C = DefaultContext, S = NoneBody> = Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, S>;
pub type ConnReq<C = DefaultContext> = Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, NoneBody>;
pub type ConnRes<C = DefaultContext> =
    Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend>;

pub enum RouteDispatch<C: Clone + Sync + Send + 'static> {
    Matched(ConnRes<C>),
    NotFound(ConnReq<C>),
    MethodNotAllowed(ConnReq<C>),
}

#[derive(Clone, Default)]
pub struct NoRoute;

pub trait RouteChain<C>: Clone + Send + Sync + 'static
where
    C: Clone + Sync + Send + 'static,
{
    fn dispatch(&self, conn: ConnReq<C>) -> impl Future<Output = RouteDispatch<C>> + Send;
}

#[derive(Clone)]
pub struct RouteNode<C, Tail, H>
where
    C: Clone + Sync + Send,
{
    method: HttpMethod,
    path_pattern: RoutePattern,
    handler: H,
    tail: Tail,
    _marker: PhantomData<fn() -> C>,
}

#[derive(Clone)]
enum RoutePattern {
    Exact(String),
    Segments(Vec<PatternSegment>),
}

#[derive(Clone)]
enum PatternSegment {
    Static(String),
    Param,
    CatchAll,
}

impl RoutePattern {
    fn parse(path: String) -> Self {
        let normalized = normalize_route_path(path);
        if !normalized.contains(':') && !normalized.contains('*') {
            return RoutePattern::Exact(normalized);
        }

        let mut segments = Vec::new();
        for part in split_path_for_match(&normalized) {
            if part.starts_with('*') || (part.starts_with(':') && part.ends_with("...")) {
                segments.push(PatternSegment::CatchAll);
                break;
            }
            if part.starts_with(':') {
                segments.push(PatternSegment::Param);
            } else {
                segments.push(PatternSegment::Static(part.to_string()));
            }
        }
        RoutePattern::Segments(segments)
    }

    #[inline(always)]
    fn is_match(&self, request_path: &str) -> bool {
        match self {
            RoutePattern::Exact(pattern) => pattern == request_path,
            RoutePattern::Segments(segments) => is_match_segments(segments, request_path),
        }
    }
}

#[inline(always)]
fn is_match_segments(segments: &[PatternSegment], request_path: &str) -> bool {
    let path = request_path.strip_prefix('/').unwrap_or(request_path);
    let bytes = path.as_bytes();
    let mut pat_idx = 0usize;
    let mut start = 0usize;
    let mut i = 0usize;

    loop {
        let at_end = i == bytes.len();
        if at_end || bytes[i] == b'/' {
            if pat_idx >= segments.len() {
                return false;
            }

            match &segments[pat_idx] {
                PatternSegment::Static(expected) => {
                    if &path[start..i] != expected {
                        return false;
                    }
                },
                PatternSegment::Param => {},
                PatternSegment::CatchAll => return true,
            }
            pat_idx += 1;

            if at_end {
                break;
            }

            i += 1;
            start = i;
            continue;
        }

        i += 1;
    }

    if pat_idx == segments.len() {
        return true;
    }

    pat_idx + 1 == segments.len() && matches!(segments[pat_idx], PatternSegment::CatchAll)
}

#[derive(Clone)]
pub struct Kurosabi<C: Clone + Sync + Send = DefaultContext, Routes = NoRoute> {
    routes: Routes,
    _marker: PhantomData<fn() -> C>,
}

impl<C: Clone + Sync + Send> Default for Kurosabi<C, NoRoute> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Clone + Sync + Send> Kurosabi<C, NoRoute> {
    pub fn new() -> Self {
        Self { routes: NoRoute, _marker: PhantomData }
    }
}

impl<C: Clone + Sync + Send, Routes> Kurosabi<C, Routes> {
    pub fn with_routes(routes: Routes) -> Self {
        Self { routes, _marker: PhantomData }
    }
}

impl<C, Routes> Kurosabi<C, Routes>
where
    C: Clone + Sync + Send,
{
    pub fn route<H, Fut, S>(self, method: HttpMethod, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        Kurosabi {
            routes: RouteNode {
                method,
                path_pattern: RoutePattern::parse(path.into()),
                handler,
                tail: self.routes,
                _marker: PhantomData,
            },
            _marker: PhantomData,
        }
    }

    pub fn get<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::GET, path, handler)
    }

    pub fn post<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::POST, path, handler)
    }

    pub fn put<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::PUT, path, handler)
    }

    pub fn delete<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::DELETE, path, handler)
    }

    pub fn patch<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::PATCH, path, handler)
    }

    pub fn head<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::HEAD, path, handler)
    }

    pub fn options<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::OPTIONS, path, handler)
    }

    pub fn trace<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::TRACE, path, handler)
    }

    pub fn connect<H, Fut, S>(self, path: S, handler: H) -> Kurosabi<C, RouteNode<C, Routes, H>>
    where
        H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ConnRes<C>> + Send + 'static,
        S: Into<String>,
    {
        self.route(HttpMethod::CONNECT, path, handler)
    }
}

impl<C, Routes> Kurosabi<C, Routes>
where
    C: Clone + Sync + Send + 'static,
    Routes: RouteChain<C>,
{
    pub async fn handle(&self, conn: ConnReq<C>) -> ConnRes<C> {
        match self.routes.dispatch(conn).await {
            RouteDispatch::Matched(conn) => conn,
            RouteDispatch::NotFound(conn) => conn.set_status_code(404u16).no_body(),
            RouteDispatch::MethodNotAllowed(conn) => conn.set_status_code(405u16).no_body(),
        }
    }
}

impl<C> RouteChain<C> for NoRoute
where
    C: Clone + Sync + Send + 'static,
{
    async fn dispatch(&self, conn: ConnReq<C>) -> RouteDispatch<C> {
        RouteDispatch::NotFound(conn)
    }
}

impl<C, Tail, H, Fut> RouteChain<C> for RouteNode<C, Tail, H>
where
    C: Clone + Sync + Send + 'static,
    Tail: RouteChain<C>,
    H: Fn(ConnReq<C>) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ConnRes<C>> + Send + 'static,
{
    async fn dispatch(&self, conn: ConnReq<C>) -> RouteDispatch<C> {
        let dispatched = self.tail.dispatch(conn).await;
        match dispatched {
            RouteDispatch::Matched(conn) => RouteDispatch::Matched(conn),
            RouteDispatch::MethodNotAllowed(conn) => {
                if self.method != *conn.req.method() {
                    RouteDispatch::MethodNotAllowed(conn)
                } else {
                    let own_path_match = {
                        let path = strip_query(conn.req.path_full());
                        self.path_pattern.is_match(path)
                    };
                    if own_path_match {
                        RouteDispatch::Matched((self.handler)(conn).await)
                    } else {
                        RouteDispatch::MethodNotAllowed(conn)
                    }
                }
            },
            RouteDispatch::NotFound(conn) => {
                let own_path_match = {
                    let path = strip_query(conn.req.path_full());
                    self.path_pattern.is_match(path)
                };
                if own_path_match {
                    if self.method == *conn.req.method() {
                        RouteDispatch::Matched((self.handler)(conn).await)
                    } else {
                        RouteDispatch::MethodNotAllowed(conn)
                    }
                } else {
                    RouteDispatch::NotFound(conn)
                }
            },
        }
    }
}

fn normalize_route_path(mut path: String) -> String {
    if path.is_empty() {
        return "/".to_string();
    }
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    path
}

#[inline(always)]
fn split_path_for_match(path: &str) -> std::str::Split<'_, char> {
    path.strip_prefix('/').unwrap_or(path).split('/')
}

#[inline(always)]
fn strip_query(path: &str) -> &str {
    path.split_once('?').map_or(path, |(path, _)| path)
}

pub struct KurosabiServerBuilder {}
pub struct KurosabiTokioServerBuilder<C: Clone = DefaultContext> {
    context: C,
    bind: [u8; 4],
    port: u16,
    keep_alive_timeout: Duration,
    http_header_read_timeout: Duration,
    limit_handle_num: usize,
    tcp_backlog: u32,
}

pub struct KurosabiTokioServer<C: Clone + Sync + Send, H> {
    router: KurosabiRouter<MyRouter<C, H>, C>,
    bind: [u8; 4],
    port: u16,
    limit_handle_num: usize,
    tcp_backlog: u32,
}

pub struct KurosabiTokioDslServer<C: Clone + Sync + Send, Routes> {
    router: KurosabiRouter<KurosabiDslRouter<C, Routes>, C>,
    bind: [u8; 4],
    port: u16,
    limit_handle_num: usize,
    tcp_backlog: u32,
}

pub trait Handler<C>: Clone + Send + Sync + 'static {
    type Fut: Future<Output = Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend>>
        + Send
        + 'static;

    fn call(&self, conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>) -> Self::Fut;
}

impl<C, F, Fut> Handler<C> for F
where
    F: Fn(Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend>>
        + Send
        + 'static,
{
    type Fut = Fut;

    #[inline(always)]
    fn call(&self, conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>) -> Self::Fut {
        (self)(conn)
    }
}

impl<C: Clone + Sync + Send + Default> KurosabiTokioServerBuilder<C> {
    pub fn new() -> Self {
        KurosabiTokioServerBuilder {
            context: C::default(),
            bind: [0, 0, 0, 0],
            port: 8080,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            limit_handle_num: DEFAULT_LIMIT_HANDLE_NUM,
            tcp_backlog: DEFAULT_TCP_BACKLOG,
        }
    }
}

impl KurosabiTokioServerBuilder<DefaultContext> {
    pub fn default() -> Self {
        KurosabiTokioServerBuilder {
            context: DefaultContext::default(),
            bind: [0, 0, 0, 0],
            port: 8080,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            limit_handle_num: DEFAULT_LIMIT_HANDLE_NUM,
            tcp_backlog: DEFAULT_TCP_BACKLOG,
        }
    }
}

impl<C: Clone + Sync + Send> KurosabiTokioServerBuilder<C> {
    pub fn with_context(context: C) -> Self {
        KurosabiTokioServerBuilder {
            context,
            bind: [0, 0, 0, 0],
            port: 8080,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            limit_handle_num: DEFAULT_LIMIT_HANDLE_NUM,
            tcp_backlog: DEFAULT_TCP_BACKLOG,
        }
    }

    pub fn bind(mut self, bind: [u8; 4]) -> Self {
        self.bind = bind;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn keep_alive_timeout(mut self, duration: Duration) -> Self {
        self.keep_alive_timeout = duration;
        self
    }

    pub fn http_header_read_timeout(mut self, duration: Duration) -> Self {
        self.http_header_read_timeout = duration;
        self
    }

    pub fn limit_handle_num(mut self, num: usize) -> Self {
        self.limit_handle_num = num;
        self
    }

    pub fn tcp_backlog(mut self, backlog: u32) -> Self {
        self.tcp_backlog = backlog;
        self
    }

    pub(crate) fn router_and_build_inner<H>(self, handler: H) -> KurosabiTokioServer<C, H>
    where
        H: Handler<C>,
    {
        let my_router = MyRouter { handler, _marker: PhantomData };
        let mut router = KurosabiRouter::with_context_and_router(my_router, self.context);
        router.set_keep_alive_timeout(self.keep_alive_timeout);
        router.set_http_header_read_timeout(self.http_header_read_timeout);
        KurosabiTokioServer {
            router,
            bind: self.bind,
            port: self.port,
            limit_handle_num: self.limit_handle_num,
            tcp_backlog: self.tcp_backlog,
        }
    }

    pub fn router_and_build<F, Fut>(self, handler: F) -> KurosabiTokioServer<C, F>
    where
        F: Fn(Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend>>
            + Send
            + 'static,
    {
        self.router_and_build_inner(handler)
    }

    pub fn kurosabi_and_build<Routes>(self, kurosabi: Kurosabi<C, Routes>) -> KurosabiTokioDslServer<C, Routes>
    where
        C: 'static,
        Routes: RouteChain<C>,
    {
        let dsl_router = KurosabiDslRouter { kurosabi, _marker: PhantomData };
        let mut router = KurosabiRouter::with_context_and_router(dsl_router, self.context);
        router.set_keep_alive_timeout(self.keep_alive_timeout);
        router.set_http_header_read_timeout(self.http_header_read_timeout);
        KurosabiTokioDslServer {
            router,
            bind: self.bind,
            port: self.port,
            limit_handle_num: self.limit_handle_num,
            tcp_backlog: self.tcp_backlog,
        }
    }
}

impl<C: Clone + Sync + Send + 'static, H: Handler<C>> KurosabiTokioServer<C, H> {
    pub async fn run(self) -> std::io::Result<()> {
        let socket = TcpSocket::new_v4()?;
        let addr = SocketAddrV4::new(Ipv4Addr::from(self.bind), self.port);
        socket.bind(SocketAddr::V4(addr))?;

        let listener = socket.listen(self.tcp_backlog)?;
        #[cfg(feature = "logging")]
        info!(
            "Server listening on {}:{}",
            self.bind
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join("."),
            self.port
        );

        let sem = Arc::new(Semaphore::new(self.limit_handle_num));
        let router = self.router;

        loop {
            let (stream, _addr) = listener.accept().await?;
            #[cfg(feature = "logging")]
            debug!("Accepted connection from {}", addr);
            let permit = sem
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore unexpectedly closed");

            let router_ref = router.clone();
            tokio::spawn(async move {
                let _permit = permit; // dropで返却される
                let (reader, writer) = stream.into_split();
                let reader = reader.compat();
                let writer = writer.compat_write();
                let _ = router_ref.new_connection_loop(reader, writer).await;
            });
        }
    }
}

impl<C, Routes> KurosabiTokioDslServer<C, Routes>
where
    C: Clone + Sync + Send + 'static,
    Routes: RouteChain<C>,
{
    pub async fn run(self) -> std::io::Result<()> {
        let socket = TcpSocket::new_v4()?;
        let addr = SocketAddrV4::new(Ipv4Addr::from(self.bind), self.port);
        socket.bind(SocketAddr::V4(addr))?;

        let listener = socket.listen(self.tcp_backlog)?;
        #[cfg(feature = "logging")]
        info!(
            "Server listening on {}:{}",
            self.bind
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join("."),
            self.port
        );

        let sem = Arc::new(Semaphore::new(self.limit_handle_num));
        let router = self.router;

        loop {
            let (stream, _addr) = listener.accept().await?;
            #[cfg(feature = "logging")]
            debug!("Accepted connection from {}", addr);
            let permit = sem
                .clone()
                .acquire_owned()
                .await
                .expect("Semaphore unexpectedly closed");

            let router_ref = router.clone();
            tokio::spawn(async move {
                let _permit = permit; // dropで返却される
                let (reader, writer) = stream.into_split();
                let reader = reader.compat();
                let writer = writer.compat_write();
                let _ = router_ref.new_connection_loop(reader, writer).await;
            });
        }
    }
}

#[derive(Clone)]
struct KurosabiDslRouter<C: Clone + Sync + Send, Routes> {
    kurosabi: Kurosabi<C, Routes>,
    _marker: PhantomData<fn() -> C>,
}

impl<C, Routes> Router<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend>
    for KurosabiDslRouter<C, Routes>
where
    C: Clone + Sync + Send + 'static,
    Routes: RouteChain<C>,
{
    #[inline(always)]
    async fn router(
        &self,
        conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>,
    ) -> Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend> {
        self.kurosabi.handle(conn).await
    }
}

#[derive(Clone)]
struct MyRouter<C: Clone + Sync + Send, H> {
    handler: H,
    _marker: PhantomData<fn() -> C>,
}

impl<C, H> Router<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend> for MyRouter<C, H>
where
    C: Clone + Sync + Send + 'static,
    H: Handler<C>,
{
    #[inline(always)]
    async fn router(
        &self,
        conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>,
    ) -> Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend> {
        self.handler.call(conn).await
    }
}
