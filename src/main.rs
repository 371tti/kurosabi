use kurosabi::{error::HttpError, kurosabi::Kurosabi};

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/hello",  |req, res, c| async move {
        let res = res.text("Hello, World!");
        Ok(res)
    });

    kurosabi.get("/hello/:name",  |req, res, c| async move {
        let name = c.field("name").unwrap();
        let res = res.text(&format!("Hello, {}!", name));
        Ok(res)
    });

    kurosabi.post("/submit",  |req, res, c| async move {
        Err(HttpError::NotFound)
    });

    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(80)
        .thread(4)
        .thread_name("kurosabi-worker".to_string())
        .queue_size(128)
        .build();

    server.run().await;
}

