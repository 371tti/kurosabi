use std::sync::Arc;

use tokio;

pub struct KurosabiServer {
    host: [u8; 4],
    port: u16,
    worker: Arc<Worker>,
    runtime: tokio::runtime::Runtime,
}

pub struct Worker {

}