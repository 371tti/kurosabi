use kurosabi::{response::{body::CompressionConfig, Res}, Kurosabi};

#[tokio::main]
async fn main() {
    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/hi", |mut c| async move {
        c.res.text("hello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶる");
        c.res.compress_config = CompressionConfig::Hi;
        c
    });

    kurosabi.get("/low", |mut c| async move {
        c.res.text("hello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶるhello, kurosabi! 錆を炎であぶる");
        c.res.compress_config = CompressionConfig::None;
        c
    });

    kurosabi.get("/file", |mut c| async move {
        Res::file(c, "examples/mini.rs", false, None).await.unwrap()
    });

    kurosabi.not_found_handler(|mut c| async move {
        c.res.text("404");
        c.res.set_status(404);
        c
    });

    kurosabi.server().build().run_async().await;
}