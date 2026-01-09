use std::io::Result;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_util::compat::Compat;

use kurosabi::{connection::Connection, router::DefaultContext, server::tokio::KurosabiTokioServerBuilder};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let server = KurosabiTokioServerBuilder::default()
        .router_and_build(|conn: Connection<DefaultContext, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>> | async move {
                conn.text_body("test")
        });
    server.run().await
}
