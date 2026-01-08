use std::io::Result;

// main.rs
use async_std::net::TcpListener;
use async_std::task;

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::AsyncReadExt;
use kurosabi::{connection::{Connection, ConnectionState, ResponseReadyToSend}, http::method::HttpMethod, router::{KurosabiRouter, Router}};

#[async_std::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    println!("listening on :8080");

    let router: KurosabiRouter<MyRouter> = KurosabiRouter::new();

    loop {
        let (stream, _addr) = listener.accept().await?;
        let router_ref = router.clone();

        task::spawn(async move {
            let (reader, writer) = stream.split();
            router_ref.new_connection_loop(reader, writer).await;
        });
    }
}

#[derive(Default, Clone)]
struct MyRouter;

impl<
    C: Clone + Send,
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
    S: ConnectionState,
> Router<C, R, W, S> for MyRouter
{
    async fn router(
        &self,
        conn: Connection<C, R, W>,
    ) -> Connection<C, R, W, ResponseReadyToSend> {
        // return conn.text_body("uee");
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
                        let own: String = others.iter().map(|s| *s).collect::<Vec<&str>>().join("/");
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