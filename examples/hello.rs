use std::io::Result;

use kurosabi::server::tokio::{Kurosabi, KurosabiTokioServerBuilder};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let kurosabi = Kurosabi::new()
        .get(
            "/hello",
            |conn| async move { conn.text_body("Hello, World!") },
        )
        .get("/hello/:name", |conn| async move {
            let body = {
                let name = conn
                    .path_seg(1)
                    .filter(|name| !name.is_empty())
                    .unwrap_or("World");
                format!("Hello, {}!", name)
            };
            conn.text_body(body)
        })
        .get("/anything/:others...", |conn| async move {
            let body = {
                let own = conn.path_tail(1).unwrap_or("");
                format!("You requested anything/{}!", own)
            };
            conn.text_body(body)
        })
        .get("/", |conn| async move {
            conn.text_body("Welcome to the Kurosabi HTTP Server!")
        });

    let server = KurosabiTokioServerBuilder::default()
        .bind([0, 0, 0, 0])
        .port(8080)
        .kurosabi_and_build(kurosabi);

    server.run().await
}
