use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};

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
    connection::{Connection, ResponseReadyToSend},
    router::{DEFAULT_KEEP_ALIVE_TIMEOUT, DefaultContext, KurosabiRouter, Router},
    server::{DEFAULT_LIMIT_HANDLE_NUM, DEFAULT_TCP_BACKLOG},
};

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
    pub fn with_context(context: C) -> Self
    where
        C: Default,
    {
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
        let router = KurosabiRouter::with_context_and_router(my_router, self.context);
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

        // 同時に処理する接続数を制限
        let sem = Arc::new(Semaphore::new(self.limit_handle_num));
        let router = self.router;

        loop {
            let (stream, addr) = listener.accept().await?;
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
