use std::io::Result;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_util::compat::Compat;

use kurosabi::{connection::Connection, http::method::HttpMethod, router::DefaultContext, server::tokio::KurosabiTokioServerBuilder};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let server = KurosabiTokioServerBuilder::default().router_and_build(
        |conn: Connection<DefaultContext, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>| async move {
            let method = conn.req.method();

            match method {
                HttpMethod::GET => match conn.path_segs().as_ref() {

                    // GET /hello
                    ["hello"] => conn.text_body("Hello, World!"),

                    // GET /hello/:name
                    ["hello", name] => {
                        let body = format!("Hello, {}!", name);
                        conn.text_body(body)
                    },

                    // GET /anything/:anything...
                    ["anything", others @ ..] => {
                        let own: String = others.join("/");
                        conn.text_body(format!("You requested anything/{}!", own))
                    },

                    // GET /
                    [""] => conn.text_body("Welcome to the Kurosabi HTTP Server!"),

                    _ => conn.set_status_code(404u16).no_body(),
                },
                _ => conn.set_status_code(405u16).no_body(),
            }
        },
    );
    server.run().await
}

