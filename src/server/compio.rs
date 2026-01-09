use std::{marker::PhantomData, time::Duration};

use compio::net::{OwnedReadHalf, OwnedWriteHalf, TcpListener, TcpStream};
use compio_io::compat::AsyncStream;

use crate::{
    connection::{Connection, ResponseReadyToSend},
    router::{DEFAULT_KEEP_ALIVE_TIMEOUT, DefaultContext, KurosabiRouter, Router},
};

pub struct KurosabiCompioServerBuilder<C: Clone = DefaultContext> {
    context: C,
    bind: String,
    port: u16,
    keep_alive_timeout: Duration,
    http_header_read_timeout: Duration,
}

pub struct KurosabiCompioServer<C: Clone + Sync + Send, H> {
    router: KurosabiRouter<MyRouter<C, H>, C>,
    bind: String,
    port: u16,
}

pub trait Handler<C>: Clone + Send + Sync + 'static {
    type Fut: Future<Output = Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>, ResponseReadyToSend>>
        + Send
        + 'static;

    fn call(&self, conn: Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>>) -> Self::Fut;
}

impl<C, F, Fut> Handler<C> for F
where
    F: Fn(Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>>) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>, ResponseReadyToSend>>
        + Send
        + 'static,
{
    type Fut = Fut;

    #[inline(always)]
    fn call(&self, conn: Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>>) -> Self::Fut {
        (self)(conn)
    }
}

impl<C: Clone + Sync + Send + Default> KurosabiCompioServerBuilder<C> {
    pub fn new() -> Self {
        KurosabiCompioServerBuilder {
            context: C::default(),
            bind: "0.0.0.0".to_string(),
            port: 8080,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
        }
    }
}

impl KurosabiCompioServerBuilder<DefaultContext> {
    pub fn default() -> Self {
        KurosabiCompioServerBuilder {
            context: DefaultContext::default(),
            bind: "0.0.0.0".to_string(),
            port: 8080,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
        }
    }
}

impl<C: Clone + Sync + Send> KurosabiCompioServerBuilder<C> {
    pub fn with_context(context: C) -> Self
    where
        C: Default,
    {
        KurosabiCompioServerBuilder {
            context,
            bind: "0.0.0.0".to_string(),
            port: 8080,
            keep_alive_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
            http_header_read_timeout: DEFAULT_KEEP_ALIVE_TIMEOUT,
        }
    }

    pub fn bind(mut self, bind: [u8; 4]) -> Self {
        self.bind = format!("{}.{}.{}.{}", bind[0], bind[1], bind[2], bind[3]);
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

    pub(crate) fn router_and_build_inner<H>(self, handler: H) -> KurosabiCompioServer<C, H>
    where
        H: Handler<C>,
    {
        let my_router = MyRouter { handler, _marker: PhantomData };
        let router = KurosabiRouter::with_context_and_router(my_router, self.context);
        KurosabiCompioServer { router, bind: self.bind, port: self.port }
    }

    pub fn router_and_build<F, Fut>(self, handler: F) -> KurosabiCompioServer<C, F>
    where
        F: Fn(Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>, ResponseReadyToSend>>
            + Send
            + 'static,
    {
        self.router_and_build_inner(handler)
    }
}

impl<C: Clone + Sync + Send + 'static, H: Handler<C>> KurosabiCompioServer<C, H> {
    pub async fn run(self) -> std::io::Result<()> {
        let listener = TcpListener::bind((self.bind.as_str(), self.port)).await?;
        println!("listening on {}:{}", self.bind, self.port);

        loop {
            let (stream, _addr) = listener.accept().await?;
            let router_ref = self.router.clone();
            compio::runtime::spawn(async move {
                let (reader, writer) = stream.into_split();
                let reader: AsyncStream<OwnedReadHalf<TcpStream>> = AsyncStream::new(reader);
                let writer: AsyncStream<OwnedWriteHalf<TcpStream>> = AsyncStream::new(writer);
                router_ref.new_connection_loop(reader, writer).await;
            }).detach();
        }
    }
}

#[derive(Clone)]
struct MyRouter<C: Clone + Sync + Send, H> {
    handler: H,
    _marker: PhantomData<fn() -> C>,
}

impl<C, H> Router<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>, ResponseReadyToSend> for MyRouter<C, H>
where
    C: Clone + Sync + Send + 'static,
    H: Handler<C>,
{
    #[inline(always)]
    async fn router(
        &self,
        conn: Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>>,
    ) -> Connection<C, AsyncStream<OwnedReadHalf<TcpStream>>, AsyncStream<OwnedWriteHalf<TcpStream>>, ResponseReadyToSend> {
        self.handler.call(conn).await
    }
}
