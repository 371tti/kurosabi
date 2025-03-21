
# !! Currently under development !!
# Please note that this content is incomplete.

# 🔥kurosabi🔥

kurosabi is a web framework that is extremely lightweight and simple, and makes use of rust's parallelism and safety.

## What is kurosabi?
A. "kurosabi" is black rust of japanese  
this framework design from "hono" of TypeScript web framework.  
"hono" is fire of japanese.  
In other words, "kurosabi" is rust heated to "hono".

## Installation
Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
kurosabi = "0.1"  // Use the latest version available
```

## Usage
Here's a detailed example to get started:

```rust
// Initialized with the default router and context
let mut kurosabi = Kurosabi::new();
// let mut custom_kurosabi = Kurosabi::with_context(...);
```

### Define Routes
You can define routes using methods like `get`, `post`, etc. Here's an example:

```rust
kurosabi.get("/hello", |mut c| async move {
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
```

### JSON Response
You can return JSON responses easily:

```rust
kurosabi.get("/json", |mut c| async move {
    let json_data = r#"{"name": "Kurosabi", "version": "0.1"}"#;
    c.res.json(json_data);
    Ok(c)
});
```

### Form Handling
Serve an HTML form and handle form submissions:

```rust
kurosabi.get("/submit", |mut c| async move {
    c.res.html(r#"
    <form action="/submit" method="post">
        <input type="text" name="data" placeholder="Enter some data" />
        <button type="submit">Submit</button>
    </form>
    "#);
    Ok(c)
});

kurosabi.post("/submit", |mut c| async move {
    let body = c.req.body_string().await.unwrap_or_default();
    println!("Received POST data: {}", body);
    c.res.html(&format!("Received: {}", body));
    Ok(c)
});
```

### Server Configuration
Configure the server with custom settings:

```rust
let mut server = kurosabi.server()
    .host([0, 0, 0, 0])
    .port(80)
    .thread(4)
    .thread_name("kurosabi-worker".to_string())
    .queue_size(128)
    .build();

server.run().await;