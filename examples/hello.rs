use std::io::Result;

// main.rs
use async_std::net::TcpListener;
use async_std::task;

use kurosabi::router::{KurosabiRouter, Router}; // あなたのパスに合わせて

#[async_std::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    println!("listening on :8080");

    let router = MyRouter;

    // Router 本体（D: Default を使うなら D::default() が必要）
    let router = KurosabiRouter::new(router);

    loop {
        let (stream, addr) = listener.accept().await?;
        let router_ref = router.clone();

        task::spawn(async move {
            let reader = stream.clone();
            let writer = stream;

            // ここであなたの routing を呼ぶだけ
            let r = router_ref.routing(reader, writer).await;

            if let Err(e) = r {
                eprintln!("Error handling connection from {}: {:?}", addr, e.router_error);
            }
        });
    }
}

// ↓ 仮の Router 実装（後述）
#[derive(Default, Clone)]
struct MyRouter;

use async_std::io::{Read as AsyncRead, Write as AsyncWrite};
use kurosabi::connection::{Connection, ConnectionState, ResponseReadyToSend};

impl<
    C: Clone + Send,
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    S: ConnectionState,
> Router<C, R, W, S> for MyRouter {

    async fn router(&self, conn: Connection<C, R, W>) -> Connection<C, R, W, ResponseReadyToSend> {
        match conn.req.path_full() {
            _ => {
                conn.text_body("hello world")
            } 
        }
    }
}