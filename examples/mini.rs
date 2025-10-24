use std::time::Duration;

use kurosabi::{response::Res, Kurosabi};
use tokio::{io::{duplex, AsyncWriteExt}, time::sleep};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/", |mut c| async move {
        c.res.text("hello");
        c
    });

    kurosabi.get("/file", |c| async move {
        Res::file(c, "examples/mini.rs", false, None).await.unwrap()
    });

    kurosabi.get("/chunked", |mut c| async move {
        // デュプレックス作成（書き込み側 a, 読み取り側 b）
        let (mut a, b) = duplex(64); // バッファ容量は適宜調整

        // 送信用タスクを spawn（切断時 write が失敗してループ終了）
        tokio::spawn(async move {
            let frames = ["\x1b[2K\r(。-`ω-)", "\x1b[2K\r(。l`ωl)"];
            let mut idx = 0;
            loop {
                sleep(Duration::from_millis(1000)).await;
                if a.write_all(frames[idx].as_bytes()).await.is_err() {
                    break;
                }
                if a.flush().await.is_err() {
                    break;
                }
                idx = (idx + 1) % frames.len();
            }
        });

        // 読み取り側をそのまま AsyncRead として渡す
        let buffer_size = 24;
        c.res.header.set("Content-Type", "text/plain; charset=utf-8");
        c.res.chunked_stream(Box::pin(b), buffer_size);
        c
    });

    kurosabi.not_found_handler(|mut c| async move {
        c.res.set_status(404);
        c
    });

    kurosabi.server()
        .nodelay(true)
        .host([0, 0, 0, 0])
        .port(85)
        .http_keepalive_timeout(Duration::from_secs(120))
        .build().run_async().await;
}