<div align="center">
<h1 style="font-size: 50px">🔥kurosabi🔥</h1>
</div>

[jp](https://github.com/371tti/kurosabi/blob/master/README-jp.md) | (en)

kurosabi is an ultra-lightweight, fast, and simple web backend router that leverages Rust's safety and parallelism.

We value performance, lightweight design, and ease of use.

## Known Issues
Critical  
If a worker thread panics, it will not restart, and the number of workers will decrease.  
If all worker threads are gone, processing will stop.  
As a temporary workaround, please avoid causing panics such as by using unwrap.

## ToDo
- Initial Implementation
  - [x] Implement http_server
  - [x] Implement router
  - [x] Implement basic syntax
- Feature Additions 1
  - [x] Implement keep_alive
  - [x] Add server configuration
  - [x] Add response-related features
- Optimization 1
  - [x] Fix keep_alive
  - [x] Add html format macro
  - [x] Improve to allow direct TCP stream manipulation
- Breaking Changes 1
  - [x] Aggregate everything into Context for easier syntax
  - [x] Improve http_server for higher throughput
- Optimization 2
  - [ ] Improve port handling for TCP operations on Linux
  - [ ] Make error handling easier
  - [x] Support for middleware
  - [ ] Enhance security

## Features
- Ultra-lightweight and fast
- Simple and expressive routing
- Async handler support
- Path parameters and wildcards
- JSON and file responses
- Custom context support
- Easy 404 and error handling
- Flexible server configuration

## Installation
Add the following to your `Cargo.toml`:

```toml
[dependencies]
kurosabi = "0.4" # Use the latest version
```

## Try it out
You can see a demo in the examples with the following command:
```
cargo run --example start
```

## Getting Started

### 1. Import
```rust
use kurosabi::{Kurosabi, kurosabi::Context};
```

### 2. Create the server, add routes, and run
```rust
fn main() {
    // Create an instance of Kurosabi
    let mut kurosabi = Kurosabi::new();

    // Define a route handler like this.
    kurosabi.get("/",  |mut c| async move {
        c.res.text("Hello, Kurosabi!");
        c
    });

    // Define a handler for GET "/field/:field/:value"
    // This handler gets the :field and :value parts from the URL path and returns "Field: {field}, Value: {value}" as a text response.
    kurosabi.get("/field/:field/:value", |mut c| async move {
        let field = c.req.path.get_field("field").unwrap_or("unknown".to_string());
        let value = c.req.path.get_field("value").unwrap_or("unknown".to_string());
        c.res.text(&format!("Field: {}, Value: {}", field, value));
        c
    });

    // Define a handler for GET "/gurd/*"
    // This handler gets the * part from the URL path and returns "Gurd: {path}" as a text response.
    // * is a wildcard and accepts any string.
    kurosabi.get("/gurd/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("unknown".to_string());
        c.res.text(&format!("Gurd: {}", path));
        c
    });

    // Define a handler for POST "/submit"
    // This returns the response data as is.
    kurosabi.post("/submit", |mut c| async move {
        let body = match c.req.body_form().await {
            Ok(data) => data,
            Err(e) => {
                println!("Error receiving POST data: {}", e);
                c.res.set_status(400);
                return c;
            }
        };
        c.res.html(&format!("Received: {:?}", body));
        c
    });

    // Define a handler for 404 not found
    kurosabi.not_found_handler(|mut c| async move {
        let html = html_format!(
            "<h1>404 Not Found</h1>
            <p>The page you are looking for does not exist.</p>
            <p>debug: {{data}}</p>",
            data = c.req.header.get_user_agent().unwrap_or("unknown")
        );
        c.res.html(&html);
        c.res.set_status(404);
        c
    });

    // Configure and build the server
    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8082)
        .build();

    // Run the server
    server.run();
}
```

## Suggestions
If you have suggestions, please open an issue.  
Pull requests are also welcome.

---

## License
MIT
