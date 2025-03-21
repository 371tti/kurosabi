# 🔥kurosabi🔥

kurosabi is a web framework that is extremely lightweight and simple, and makes use of rust's parallelism and safety.

## What is kurosabi?
A. "kurosabi" is black rust of japanese  
this framework design from "hono" of TypeScript web framework.  
"hono" is fire of japanese.  
In other words, "kurosabi" is rust heated to "hono".  

## Features
- Extremely lightweight and high performance
- Built with Rust for memory safety and thread safety
- Asynchronous support using Tokio
- Easy routing and middleware management
- Customizable and extendable

## Installation
Add the following dependency to your Cargo.toml:

```toml
[dependencies]
kurosabi = "0.1"  // Use the latest version available
```

## Usage
Here's a simple example to get started:

```rust
use kurosabi::kurosabi::Kurosabi;

#[tokio::main]
async fn main() {
    let mut kurosabi = Kurosabi::new();
    // Define routes, middlewares, etc.
    kurosabi.run().await;
}
```

## Contributing
Contributions are welcome!  
Please follow the project's coding style and open an issue to discuss major changes.