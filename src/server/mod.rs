pub mod worker;

use std::{net::SocketAddr, sync::Arc};

use socket2::{Domain, Protocol, Socket};
use tokio::{self, io::AsyncWriteExt};
use log::info;
use worker::{Worker, WorkerPool};

pub struct KurosabiServer<W> {
    config: KurosabiConfig,
    worker: Arc<WorkerPool<W>>,
    runtime: tokio::runtime::Runtime,
}

pub struct KurosabiServerBuilder<W> {
    config: KurosabiConfig,
    prc_worker: Arc<W>,
}

#[derive(Clone)]
pub struct KurosabiConfig {
    host: [u8; 4],
    port: u16,
    thread: usize,
    thread_name: String,
    queue_size: usize,
}

impl<W: Worker + 'static> KurosabiServerBuilder<W> {
    pub fn new(worker: Arc<W>) -> KurosabiServerBuilder<W> {
        KurosabiServerBuilder {
            config: KurosabiConfig {
                host: [127, 0, 0, 1],
                port: 8080,
                thread: 4,
                thread_name: "kurosabi-worker".to_string(),
                queue_size: 128,
            },
            prc_worker: worker,
        }
    }

    /// サーバーのホストを設定する
    pub fn host(mut self, host: [u8; 4]) -> Self {
        self.config.host = host;
        self
    }

    /// サーバーのポートを設定する
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// スレッド数を設定する
    pub fn thread(mut self, thread: usize) -> Self {
        self.config.thread = thread;
        self
    }

    /// スレッド名を設定する
    pub fn thread_name(mut self, thread_name: String) -> Self {
        self.config.thread_name = thread_name;
        self
    }

    /// タスクキューのサイズを設定する
    pub fn queue_size(mut self, queue_size: usize) -> Self {
        self.config.queue_size = queue_size;
        self
    }

    /// サーバーを構成する
    pub fn build(self) -> KurosabiServer<W> {
        let worker = Arc::new(WorkerPool::new(self.config.queue_size, self.prc_worker.clone()));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.thread)
            .enable_all()
            .build()
            .unwrap();
        KurosabiServer {
            config: self.config.clone(),
            worker,
            runtime,
        }
    }
}

impl<W: Worker + 'static> KurosabiServer<W> {
    /// サーバーを起動する
    pub async fn run(&mut self) {
        let addr = SocketAddr::from((self.config.host, self.config.port));
        let socket = Socket::new(Domain::IPV4, socket2::Type::STREAM, Some(Protocol::TCP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket.set_nodelay(true).unwrap();
        socket.bind(&addr.into()).unwrap();
        socket.listen(1024).unwrap();

        let listener = tokio::net::TcpListener::from_std(socket.into()).unwrap();
        info!("Server starting on http://{}", addr);

        // ワーカースレッドを起動する
        for i in 0..self.config.thread {
            let worker = self.worker.clone();
            self.runtime.handle().spawn(async move {
                worker.main_loop().await;
            });
            info!("{}.{} running !", self.config.thread_name, i);
        }

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            socket.set_nodelay(true).unwrap();
            let connection = TcpConnection::new(socket);
            self.worker.assign_connection(connection).await;
        }
    }
}

pub struct TcpConnection {
    pub reader: tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>,
    pub writer: tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf>,
}

impl TcpConnection {
    /// 分割して BufReader と BufWriter を一度だけ作成し、保存します
    pub fn new(socket: tokio::net::TcpStream) -> TcpConnection {
        let (read_half, write_half) = socket.into_split();
        TcpConnection {
            reader: tokio::io::BufReader::new(read_half),
            writer: tokio::io::BufWriter::new(write_half),
        }
    }

    /// BufReader への可変参照を返します
    pub fn reader(&mut self) -> &mut tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf> {
        &mut self.reader
    }
    
    /// BufWriter への可変参照を返します
    pub fn writer(&mut self) -> &mut tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf> {
        &mut self.writer
    }

    pub async fn close(&mut self) {
        self.writer.shutdown().await.unwrap();
    }
}
