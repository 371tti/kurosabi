# 🔥kurosabi🔥

Kurosabi is a blazing fast, lightweight, and simple web framework for Rust, designed to leverage Rust's safety and parallelism. Inspired by the TypeScript framework "hono", Kurosabi aims to provide a productive and enjoyable web development experience.

---

## Features
- Ultra-lightweight and fast
- Simple and expressive routing
- Async handler support
- Path parameters and wildcards
- JSON and file responses
- Custom context support
- Easy 404 and error handling
- Fine-grained server configuration

---

## Installation
Add Kurosabi to your `Cargo.toml`:

```toml
[dependencies]
kurosabi = "0.3.0"
```

---

## Getting Started

### 1. Define Your Context (Optional)
```rust
pub struct MyContext {
    pub name: String,
}
impl MyContext {
    pub fn new(name: String) -> Self {
        MyContext { name }
    }
}
```

### 2. Create the Server and Add Routes
```rust
use std::{path::PathBuf, sync::Arc};
use kurosabi::{Kurosabi, kurosabi::Context};

#[tokio::main]
async fn main() {
    let arc_context = Arc::new(MyContext::new("Kurosabi".to_string()));
    let mut kurosabi = Kurosabi::with_context(arc_context);

    // Simple text response
    kurosabi.get("/hello", |mut c| async move {
        c.res.text("Hello, World!");
        c
    });

    // Path parameter
    kurosabi.get("/hello/:name", |mut c| async move {
        let name = c.req.path.get_field("name").unwrap_or("World".to_string());
        c.res.text(&format!("Hello, {}!", name));
        c
    });

    // Wildcard
    kurosabi.get("/wild/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("unknown".to_string());
        c.res.text(&format!("Wildcard: {}", path));
        c
    });

    // JSON response
    kurosabi.get("/json", |mut c| async move {
        let json_data = r#"{"name": "Kurosabi", "version": "0.1"}"#;
        c.res.json(json_data);
        c
    });

    // File response
    kurosabi.get("/file", |mut c| async move {
        let _ = c.res.file(&c.req, PathBuf::from("README.md"), true).await.unwrap();
        c
    });

    // Form (GET and POST)
    kurosabi.get("/submit", |mut c| async move {
        c.res.html(r#"
        <form action=\"/submit\" method=\"post\">
            <input type=\"text\" name=\"data\" placeholder=\"Enter some data\" />
            <button type=\"submit\">Submit</button>
        </form>
        "#);
        c
    });
    kurosabi.post("/submit", |mut c| async move {
        let body = match c.req.body_form().await {
            Ok(data) => data,
            Err(_) => {
                c.res.set_status(400);
                return c;
            }
        };
        c.res.html(&format!("Received: {:?}", body));
        c
    });

    // 404 handler
    kurosabi.not_found_handler(|mut c| async move {
        let html = format!(
            "<h1>404 Not Found</h1>\n<p>The page you are looking for does not exist.</p>\n<p>debug: {}</p>",
            c.req.header.get_user_agent().unwrap_or("unknown")
        );
        c.res.html(&html);
        c.res.set_status(404);
        c
    });

    // Server configuration
    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8080)
        .thread(8)
        .thread_name("kurosabi-worker".to_string())
        .queue_size(128)
        .build();

    server.run().await;
}
```

---

## Advanced Features

### JSON API with Custom Handler
```rust
use kurosabi::api::GETJsonAPI;
use serde::Serialize;

#[derive(Clone)]
pub struct MyAPI;
#[derive(Serialize)]
pub struct ResJsonSchemaVersion {
    pub name: String,
    pub version: String,
}
#[derive(Serialize)]
#[serde(untagged)]
pub enum ResJsonSchema {
    Version(ResJsonSchemaVersion),
    Error(String),
}

#[async_trait::async_trait]
impl GETJsonAPI<Context<Arc<MyContext>>, ResJsonSchema> for MyAPI {
    fn new() -> Self { MyAPI }
    async fn handler(self, c: &mut Context<Arc<MyContext>>) -> ResJsonSchema {
        let name = c.req.path.get_query("name").unwrap_or("Kurosabi".to_string());
        let version = c.req.path.get_query("version").unwrap_or("0.1".to_string());
        ResJsonSchema::Version(ResJsonSchemaVersion { name, version })
    }
}

// Register the API route
kurosabi.get_json_api("/jsonapi", MyAPI::new());
```

---

## License
MIT