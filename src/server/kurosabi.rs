use std::collections::VecDeque;

use tokio::runtime::Runtime;

use super::TcpConnection;

pub struct DefaultWorker {
    pub runtime: Runtime,
    pub queue: VecDeque<TcpConnection>,
}

pub struct 