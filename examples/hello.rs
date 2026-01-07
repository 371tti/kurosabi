use std::io::Result;

// main.rs
use async_std::net::TcpListener;
use async_std::task;

use kurosabi::router::{KurosabiRouter, Router};

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
                conn = match router_ref.routing(conn).await {
                    Ok(conn_res) => {
                        match conn_res {
                            Ok(conn) => {
                                conn
                            }
                            Err(e) => {
                                eprintln!("Routing error from {}: {:?}", addr, e.router_error);
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Connection error from {}: {:?}", addr, e);
                        break;
                    }
                };
            }
        });
    }
}

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
            "/" => {
                conn.set_status_code(200u16).text_body("hello world")
            } 

            _ => {
                conn.set_status_code(404u16).text_body("not found")
            }
        }
    }
}