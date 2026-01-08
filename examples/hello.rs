use std::io::Result;

// main.rs
use async_std::net::TcpListener;
use async_std::task;

use futures_io::{AsyncRead, AsyncWrite};
use kurosabi::{connection::{Connection, ConnectionState, ResponseReadyToSend}, http::method::HttpMethod, router::{KurosabiRouter, Router, RoutingResult}};

#[async_std::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    println!("listening on :8080");

    let router = MyRouter;

    let router = KurosabiRouter::new(router);

    loop {
        let (stream, addr) = listener.accept().await?;
        let router_ref = router.clone();

        task::spawn(async move {
            let (reader, writer) = (stream.clone(), stream.clone());
            let mut conn = router_ref.new_connection(reader, writer);
            loop {
                conn = match router_ref.routing(conn, None, None).await {
                    RoutingResult::Continue(c) => c,
                    RoutingResult::Close(e) => {
                        eprintln!("Connection with {} closed: {:?}", addr, e);
                        break;
                    }
                    RoutingResult::CloseHaveConnection(e) => {
                        eprintln!("Connection with {} closed: {:?}", addr, e.router_error);
                        break;
                    }
                };
            }
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
        let path = conn.req.path_full();
        let mut it = path.split('/').skip(1);
        match method {
            HttpMethod::GET => {
                match it.next() {
                    Some("hello") => {
                        if let Some(name) = it.next() {
                            let name = name.to_string();
                            conn.text_body(format!("Hello, {}!", name))
                        } else {
                            conn.text_body("Hello, World!")
                        }
                    }
                    Some("") => {
                        conn.text_body("Welcome to the root path!")
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
