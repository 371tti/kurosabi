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

    kurosabi.get("/post_test", |req, res, c| async move {
        let res = res.html(r#"
            <html>
                <head>
                    <title>Post Test</title>
                </head>
                <body>
                    <form action="/submit" method="post">
                        <input type="text" name="name" />
                        <input type="submit" value="Submit" />
                    </form>
                </body>
            </html>
        "#);
        Ok(res)
    });

    kurosabi.post("/submit",  |req, res, c| async move {
        let body = req.body().await;
        println!("Received: {}", body);
        let res = res.html(&format!(
            r#"
                <html>
                    <head>
                        <title>Post Test</title>
                    </head>
                    <body>
                        <p>Received: {}</p>
                    </body>
                </html>
            "#,
            body
        ));
        Ok(res)
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

