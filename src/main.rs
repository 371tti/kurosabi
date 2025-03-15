use kurosabi::{context::DefaultContext, error::HttpError, kurosabi::Kurosabi};

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/hello",  |mut c| async move {
        c.res.text("Hello, World!");
        Ok(c)
    });

    
    kurosabi.get("/goodbye",  |mut c| async move {
        c.res.text("Goodbye, World!");
        Ok(c)
    });

    kurosabi.get("/field/:name", |mut c| async move {
        let message = format!("Field value: {}", c.req.path.get_field("name").unwrap_or("unknown".to_string()));
        c.res.text(&message);
        Ok(c)
    });

    kurosabi.post("/submit", |mut c| async move {
        let body = c.req.body().await;
        println!("Body: {}", body);
        c.res.text(format!("Submission received! Body: {}", body).as_str());
        Ok(c)
    });

    kurosabi.get("/submit", |mut c| async move {
        c.res.html(
            r#"
            <form method="post" action="/submit">
                <input type="text" name="name" />
                <input type="submit" value="Submit" />
            </form>
            "#
        );
        Ok(c)
    });

    kurosabi.get("/", |mut c| async move {
        c.res.html(
            r#"
            <h1>Welcome to Kurosabi!</h1>
            <p>Click <a href="/hello">here</a> to say hello!</p>
            <p>Click <a href="/goodbye">here</a> to say goodbye!</p>
            <p>Click <a href="/field/yourname">here</a> to see a field value!</p>
            <p>Click <a href="/submit">here</a> to submit a form!</p>
            "#
        );
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

