use kurosabi::Kurosabi;

#[tokio::main]
async fn main() {
    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/", |mut c| async move {
        c.res.text("hello");
        c
    });

    kurosabi.not_found_handler(|mut c| async move {
        c.res.set_status(404);
        c
    });

    let handle = tokio::spawn(async move {
        kurosabi.server()
            .build().run_async().await;
    });

    println!("Default binding server is running on http://localhost:8080");

    handle.await.unwrap();
}