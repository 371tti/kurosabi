pub mod worker;

use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use tokio::{self, sync::Notify};
use log::{error, info};
use worker::{Worker, WorkerPool};

pub struct KurosabiServer<W> {
    config: KurosabiConfig,
    worker: Arc<WorkerPool<W>>,
    prc_worker: Arc<W>,
    runtime: tokio::runtime::Runtime,
}

pub struct KurosabiConfig {
    host: [u8; 4],
    port: u16,
    thread: usize,
    thread_name: String,
    queue_size: usize,
}

impl<W: Worker + 'static> KurosabiServer<W> {
    pub fn new(worker: Arc<W>) -> KurosabiServer<W> {
        KurosabiServer {
            config: KurosabiConfig {
                host: [127, 0, 0, 1],
                port: 8080,
                thread: 4,
                thread_name: "kurosabi-worker".to_string(),
                queue_size: 128,
            },
            worker: Arc::new(WorkerPool::new(128, worker.clone())),
            prc_worker: worker,
            runtime: tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .thread_name("kurosabi-worker")
                .enable_all()
                .build()
                .unwrap(),
        }
    }

    /// サーバーのホストを設定する
    pub fn host(&mut self, host: [u8; 4]) -> &mut Self {
        self.config.host = host;
        self
    }

    /// サーバーのポートを設定する
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.config.port = port;
        self
    }

    /// スレッド数を設定する
    pub fn thread(&mut self, thread: usize) -> &mut Self {
        self.config.thread = thread;
        self
    }

    /// スレッド名を設定する
    pub fn thread_name(&mut self, thread_name: String) -> &mut Self {
        self.config.thread_name = thread_name;
        self
    }

    /// タスクキューのサイズを設定する
    pub fn queue_size(&mut self, queue_size: usize) -> &mut Self {
        self.config.queue_size = queue_size;
        self
    }

    /// サーバーを構成する
    pub fn build(&mut self) -> &mut Self {
        self.runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.thread)
            .thread_name(self.config.thread_name.clone())
            .enable_all()
            .build()
            .unwrap();
        self.worker = Arc::new(WorkerPool::new(self.config.queue_size, self.prc_worker.clone()));
        self
    }

    /// サーバーを起動する
    async fn run(&mut self) {
        let addr = format!("{}.{}.{}.{}:{}", self.config.host[0], self.config.host[1], self.config.host[2], self.config.host[3], self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        info!("Server starting on {}", addr);

        // ワーカースレッドを起動する
        for i in 0..self.config.thread {
            let worker = self.worker.clone();
            tokio::spawn(async move {
                worker.main_loop().await;
            });
            info!("{}.{} started", self.config.thread_name, i);
        }

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let connection = TcpConnection { socket };
            self.worker.assign_connection(connection).await;
        }
    }
}

pub struct TcpConnection {
    pub socket: tokio::net::TcpStream,
}