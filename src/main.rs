use std::sync::Arc;

use kurosabi::{
    api::GETJsonAPI, html_format, kurosabi::Context, Kurosabi
};
use serde::Serialize;

pub struct MyContext {
    pub name: String,
}

impl MyContext {
    pub fn new(name: String) -> Self {
        MyContext { name }
    }
}


#[derive(Clone)]
pub struct MyAPI;

#[derive(Serialize)]
pub struct MyAPIJson {
    pub name: String,
    pub version: String,
}

impl GETJsonAPI<Context<Arc<MyContext>>, MyAPIJson> for MyAPI {
    fn new() -> Self {
        MyAPI
    }

    fn handler(
            self,
            c: &mut Context<Arc<MyContext>>,
        ) -> MyAPIJson {
            let name = c.req.path.get_query("name").unwrap_or("Kurosabi".to_string());
            let version = c.req.path.get_query("version").unwrap_or("0.1".to_string());
            c.res.header.set("Connection", "keep-alive");
            c.res.header.set("Keep-Alive", "timeout=60, max=100");
            
            MyAPIJson {
                name,
                version,
            }
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();

    let arc_context = Arc::new(MyContext::new("Kurosabi".to_string()));

    let mut kurosabi = Kurosabi::with_context(arc_context);

    kurosabi.get_json_api("/jsonapi", MyAPI::new());




    kurosabi.get("/hello",  |mut c| async move {
        c.res.text("Hello, World!");
        let key = "session_id";
        let value = "123456";
        c.res.header.set_cookie(key, value);
        c.res.header.set("X-Custom-Header", "MyValue");
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
        let body = match c.req.body_form().await {
            Ok(data) => data,
            Err(e) => {
                println!("Error receiving POST data: {}", e);
                c.res.set_status(400);
                return Ok(c);
            }
        };
        println!("Received POST data: {:?}", body);
        c.res.html(&format!("Received: {:?}", body));
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

    kurosabi.get("/", |mut c| async move {
        c.res.html(r#"
        <h1>Welcome to Kurosabi!</h1>
        <p>Try the following routes:</p>
        <ul>
            <li><a href="/hello">/hello</a></li>
            <li><a href="/hello/John">/hello/John</a></li>
            <li><a href="/json">/json</a></li>
            <li><a href="/field/name/Kurosabi">/field/name/Kurosabi</a></li>
            <li><a href="/gurd/some/path">/gurd/some/path</a></li>
            <li><a href="/submit">/submit</a></li>
            <li><a href="/gurd/*">/gurd/*</a></li>
        </ul>
        "#);
        Ok(c)
    });

    kurosabi.not_found_handler(|mut c| async move {
        let html = html_format!(
            "<h1>404 Not Found</h1>
            <p>The page you are looking for does not exist.</p>
            <p>debug: {{data}}</p>",
            data = c.req.header.get_user_agent().unwrap_or("unknown")
        );
        c.res.html(&html);
        c.res.set_status(404);
        Ok(c)
    });


    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8080)
        .thread(8)
        .thread_name("kurosabi-worker".to_string())
        .queue_size(128)
        .build();

    server.run().await;
}

