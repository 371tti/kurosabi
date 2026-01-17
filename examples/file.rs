use std::io::Result;

use kurosabi::{
    connection::file::FileContentBuilder, http::HttpMethod, server::tokio::KurosabiTokioServerBuilder,
    utils::url_encode,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();
    let server = KurosabiTokioServerBuilder::default()
        .bind([0, 0, 0, 0])
        .port(8080)
        .router_and_build(|conn| async move {
            match conn.req.method() {
                HttpMethod::GET => match conn.path_segs().as_ref() {
                    // GET /file/:path
                    ["file", path @ ..] => {
                        let content = FileContentBuilder::base("./").path_url_segs(path).inline();
                        match content.check_file_exists().await {
                            Ok(content) => conn
                                .file_body(content)
                                .await
                                .unwrap_or_else(|e| e.connection),
                            Err(Some(file_paths)) => {
                                let html = file_paths
                                    .into_iter()
                                    .filter_map(|p| p.to_str().map(|s| s.to_string()))
                                    .map(|p| {
                                        format!(
                                            "<li><a href=\"/file/{}\">{}</a></li>",
                                            p.split(&['/', '\\'][..])
                                                .map(|s| url_encode(s))
                                                .collect::<Vec<_>>()
                                                .join("/"),
                                            p
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                conn.html_body(html)
                            },
                            Err(None) => conn.set_status_code(404u16).no_body(),
                        }
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
