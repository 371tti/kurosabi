use kurosabi::kurosabi::Kurosabi;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/hello",  |mut c| async move {
        c.res.text("Hello, World!");
        c.res.set_cookie("session_id", "value");
        c.res.set_header("X-Custom-Header", "MyValue");
        c.res.set_status(200);
        Ok(c)
    });

    kurosabi.get("/hello/:name", |mut c| async move {
        let name = c.req.path.get_field("name").unwrap_or("World".to_string());
        c.res.text(&format!("Hello, {}!", name));
        c.res.set_status(200);
        Ok(c)
    });

    kurosabi.get("/json", |mut c| async move {
        let json_data = r#"{"name": "Kurosabi", "version": "0.1"}"#;
        c.res.json(json_data);
        Ok(c)
    });

    kurosabi.get("/field/:field/:value", |mut c| async move {
        let field = c.req.path.get_field("field").unwrap_or("unknown".to_string());
        let value = c.req.path.get_field("value").unwrap_or("unknown".to_string());
        c.res.text(&format!("Field: {}, Value: {}", field, value));
        Ok(c)
    });

    kurosabi.get("/gurd/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("unknown".to_string());
        c.res.text(&format!("Gurd: {}", path));
        Ok(c)
    });

    kurosabi.post("/submit", |mut c| async move {
        let body = match c.req.body().await {
            Ok(body) => body,
            Err(e) => {
                c.res = e.err_res();
                return Ok(c);
            }
        };
        println!("Received POST data: {}", body);
        c.res.html(&format!("Received: {}", body));
        Ok(c)
    });

    kurosabi.get("/submit", |mut c| async move {
        c.res.html(r#"
        <form action="/submit" method="post">
            <input type="text" name="data" placeholder="Enter some data" />
            <button type="submit">Submit</button>
        </form>
        "#);
        Ok(c)
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

