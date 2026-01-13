use std::{io::Result, path::Path};

use kurosabi::{connection::file::FileContentBuilder, http::HttpMethod, server::tokio::KurosabiTokioServerBuilder};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let server = KurosabiTokioServerBuilder::default()
        .bind([0, 0, 0, 0])
        .port(8080)
        .router_and_build(|conn| async move {
            match conn.req.method() {
                HttpMethod::GET => match conn.path_segs().as_ref() {
                    // GET /hello
                    [ "file", path @ .. ] => {
                        let path = Path::new("./").join(path.join("/"));
                        let content_b = FileContentBuilder::new(path)
                            .inline();
                        conn.file_body(content_b).await.unwrap_or_else(|e|
                            e.connection
                        )
                    },
                    // GET /
                    [""] => conn.text_body("Welcome to the Kurosabi HTTP Server!"),

                    _ => conn.set_status_code(404u16).no_body(),
                },
                _ => conn.set_status_code(405u16).no_body(),
            }
        });
    server.run().await
}
