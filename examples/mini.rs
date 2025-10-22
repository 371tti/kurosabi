use kurosabi::{response::body::CompressionConfig, Kurosabi};

#[tokio::main]
async fn main() {
    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/", |mut c| async move {
        c.res.text("hello, kurosabi!");
        c.res.compress_config = CompressionConfig::Hi;
        c
    });

    kurosabi.server().build().run_async().await;
}