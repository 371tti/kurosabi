use std::marker::PhantomData;

use tokio::net::{TcpListener, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{connection::{Connection, ResponseReadyToSend}, router::{DefaultContext, KurosabiRouter, Router}};

pub struct KurosabiServerBuilder {}
pub struct KurosabiTokioServerBuilder<C: Clone = DefaultContext> {
    context: C,
    bind: String,
    port: u16,
}

pub struct KurosabiTokioServer<C: Clone + Sync + Send, H> {
    router: KurosabiRouter<MyRouter<C, H>, C>,
    bind: String,
    port: u16,
}

pub trait Handler<C>: Clone + Send + Sync + 'static {
    type Fut: Future<
            Output = Connection<
                C,
                Compat<OwnedReadHalf>,
                Compat<OwnedWriteHalf>,
                ResponseReadyToSend,
            >,
        > + Send
        + 'static;

    fn call(
        &self,
        conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>,
    ) -> Self::Fut;
}

impl<C, F, Fut> Handler<C> for F
where
    F: Fn(Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>) -> Fut
        + Clone
        + Send
        + Sync
        + 'static,
    Fut: Future<
            Output = Connection<
                C,
                Compat<OwnedReadHalf>,
                Compat<OwnedWriteHalf>,
                ResponseReadyToSend,
            >,
        > + Send
        + 'static,
{
    type Fut = Fut;

    fn call(
        &self,
        conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>,
    ) -> Self::Fut {
        (self)(conn)
    }
}

impl<C: Clone + Sync + Send + Default> KurosabiTokioServerBuilder<C> {
    pub fn new() -> Self {
        KurosabiTokioServerBuilder {
            context: C::default(),
            bind: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

impl KurosabiTokioServerBuilder<DefaultContext> {
    pub fn default() -> Self {
        KurosabiTokioServerBuilder {
            context: DefaultContext::default(),
            bind: "0.0.0.0".to_string(),
            port: 8080,
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
            bind: "0.0.0.0".to_string(),
            port: 8080,
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

    pub fn router_and_build<H>(self, handler: H) -> KurosabiTokioServer<C, H>
    where
        H: Handler<C>,
    {
        let my_router = MyRouter {
            handler,
            _marker: PhantomData,
        };
        let router = KurosabiRouter::with_context_and_router(my_router, self.context);
        KurosabiTokioServer {
            router,
            bind: self.bind,
            port: self.port,
        }
    }
}

impl<C: Clone + Sync + Send + 'static, H: Handler<C>> KurosabiTokioServer<C, H> {
    pub async fn run(self) -> std::io::Result<()> {
        let listener = TcpListener::bind((self.bind.as_str(), self.port)).await?;
        println!("listening on {}:{}", self.bind, self.port);

        loop {
            let (stream, _addr) = listener.accept().await?;
            let router_ref = self.router.clone();
            tokio::spawn(async move {
                let (reader, writer) = stream.into_split();
                let reader = reader.compat();
                let writer = writer.compat_write();
                router_ref.new_connection_loop(reader, writer).await;
            });
        }
    }
}

#[derive(Clone)]
struct MyRouter<C: Clone + Sync + Send, H> {
    handler: H,
    _marker: PhantomData<fn() -> C>,
}

impl<C, H> Router<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend>
    for MyRouter<C, H>
where
    C: Clone + Sync + Send + 'static,
    H: Handler<C>,
{
    async fn router(
        &self,
        conn: Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>,
    ) -> Connection<C, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>, ResponseReadyToSend> {
        self.handler.call(conn).await
    }
}