use std::{io::Result, sync::Arc};

// main.rs
use tokio::net::TcpListener;

use futures_io::{AsyncRead, AsyncWrite};
use kurosabi::{connection::{Connection, ConnectionState, ResponseReadyToSend}, http::method::HttpMethod, router::{KurosabiRouter, Router}};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let router: Arc<KurosabiRouter<MyRouter>> = Arc::new(KurosabiRouter::new());

    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    println!("listening on :8080");

    loop {
        let (stream, _addr) = listener.accept().await?;
        let router_ref = router.clone();

        tokio::spawn(async move {
            let (reader, writer) = stream.into_split();
            let reader = reader.compat();
            let writer = writer.compat_write();
            router_ref.new_connection_loop(reader, writer).await;
        });
    }
}

#[derive(Default, Clone)]
struct MyRouter;

impl<
    C: Clone + Send,
    R: AsyncRead + Unpin + 'static,
    W: AsyncWrite + Unpin + 'static,
    S: ConnectionState,
> Router<C, R, W, S> for MyRouter
{
    async fn router(
        &self,
        conn: Connection<C, R, W>,
    ) -> Connection<C, R, W, ResponseReadyToSend> {
        let method = conn.req.method();
        match method {
            HttpMethod::GET => {
                match conn.path_segs().as_ref() {
                    ["hello"] => {
                        conn.text_body("Hello, World!")
                    }
                    ["hello", name] => {
                        let body = format!("Hello, {}!", name);
                        conn.text_body(body)
                    }
                    ["anything", others @ ..] => {
                        let own: String = others.join("/");
                        conn.text_body(format!("You requested /anything/{}!", own))
                    }
                    [] => {
                        conn.text_body("Welcome to the Kurosabi HTTP Server!")
                    }
                    _ => {
                        conn.set_status_code(404u16).no_body()
                    }
                }
            }
            _ => {
                conn.set_status_code(405u16).no_body()
            }
        }
    }
}