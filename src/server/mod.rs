pub mod worker;

use std::{net::SocketAddr, sync::{atomic::AtomicU64, Arc}, time::Duration};

use crossbeam_queue::ArrayQueue;
use socket2::{Domain, Protocol, Socket, TcpKeepalive};
use tokio::{self, io::AsyncWriteExt, net::TcpListener};
use log::{error, info};
use worker::{Worker};
use std::sync::atomic::Ordering::Relaxed;

use crate::server::worker::{DefaultWorker, Executor};

pub struct KurosabiServer<E> 
where E: Executor + Send + Sync + 'static {
    config: KurosabiConfig,
    global_queue: Arc<ArrayQueue<TcpConnection>>,
    workers_load: Arc<Box<[AtomicU64]>>,
    proc_executor: Arc<E>,
    workers: Vec<DefaultWorker<E>>,
    _marker: std::marker::PhantomData<E>, // Added PhantomData to use E
}

pub struct KurosabiServerBuilder<E> {
    config: KurosabiConfig,
    prc_executor: Arc<E>,
}

/// Configuration for the Kurosabi server.
///
/// This struct contains various settings for the server, such as host, port, thread count,
/// buffer sizes, and keep-alive options. These configurations allow fine-grained control
/// over the server's behavior and performance.
///
/// Kurosabiサーバーの設定。
///
/// この構造体は、サーバーのホスト、ポート、スレッド数、バッファサイズ、
/// Keep-Aliveオプションなど、さまざまな設定を保持します。
/// これらの設定により、サーバーの動作やパフォーマンスを細かく制御できます。
#[derive(Clone)]
pub struct KurosabiConfig {
    /// The IP address of the server.
    ///
    /// This specifies the address the server will bind to. For example, `[127, 0, 0, 1]` binds
    /// the server to localhost, while `[0, 0, 0, 0]` binds it to all available network interfaces.
    ///
    /// サーバーのIPアドレス。
    ///
    /// サーバーがバインドするアドレスを指定します。例: `[127, 0, 0, 1]`はローカルホスト、
    /// `[0, 0, 0, 0]`は全てのネットワークインターフェースにバインドします。
    host: [u8; 4],
    /// The port number the server will listen on.
    ///
    /// This determines the port where the server will accept incoming connections.
    ///
    /// サーバーがリッスンするポート番号。
    ///
    /// サーバーが接続を受け付けるポートを指定します。
    port: u16,
    /// The number of threads to use for the server.
    ///
    /// This controls the number of worker threads that will handle incoming requests.
    ///
    /// サーバーで使用するスレッド数。
    ///
    /// リクエストを処理するワーカースレッドの数を指定します。
    thread: usize,
    /// The name of the worker threads.
    ///
    /// This is used to identify the worker threads in logs or debugging tools.
    ///
    /// ワーカースレッドの名前。
    ///
    /// ログやデバッグツールでスレッドを識別するために使用されます。
    thread_name: String,
    /// The size of the task queue for the worker pool.
    ///
    /// This defines how many tasks can be queued for processing before new tasks are rejected.
    ///
    /// ワーカープールのタスクキューサイズ。
    ///
    /// 新しいタスクが拒否されるまでにキューできるタスク数を指定します。
    queue_size: usize,
    /// Whether to allow the reuse of local addresses.
    ///
    /// If enabled, the server can bind to an address that is in a TIME_WAIT state.
    ///
    /// ローカルアドレスの再利用を許可するかどうか。
    ///
    /// 有効にすると、TIME_WAIT状態のアドレスにもバインドできます。
    reuse_address: bool,
    /// Whether to disable Nagle's algorithm for the socket.
    ///
    /// Disabling Nagle's algorithm can reduce latency for small packets by sending them immediately.
    ///
    /// ソケットのNagleアルゴリズムを無効化するかどうか。
    ///
    /// 無効化すると、小さなパケットを即時送信し、遅延を減らせます。
    nodelay: bool,
    /// The maximum number of pending connections in the backlog.
    ///
    /// This sets the maximum number of connections that can be queued before the server starts rejecting new ones.
    ///
    /// バックログの最大保留接続数。
    ///
    /// サーバーが新しい接続を拒否するまでにキューできる最大接続数を指定します。
    backlog: usize,
    /// The size of the send buffer for the socket, in bytes.
    ///
    /// This controls the amount of data that can be buffered for sending before the socket blocks.
    ///
    /// ソケットの送信バッファサイズ（バイト単位）。
    ///
    /// ソケットがブロックする前に送信できるデータ量を制御します。
    send_buffer_size: usize,
    /// The size of the receive buffer for the socket, in bytes.
    ///
    /// This controls the amount of data that can be buffered for receiving before the socket blocks.
    ///
    /// ソケットの受信バッファサイズ（バイト単位）。
    ///
    /// ソケットがブロックする前に受信できるデータ量を制御します。
    recv_buffer_size: usize,
    /// Whether to enable TCP keep-alive.
    ///
    /// If enabled, the server will periodically send keep-alive packets to ensure the connection is still active.
    ///
    /// TCP Keep-Aliveを有効にするかどうか。
    ///
    /// 有効にすると、サーバーは定期的にKeep-Aliveパケットを送信し、接続が生きているか確認します。
    keepalive_enabled: bool,
    /// The idle time before the first keep-alive packet is sent.
    ///
    /// This specifies how long the connection can remain idle before the first keep-alive packet is sent.
    ///
    /// 最初のKeep-Aliveパケット送信までのアイドル時間。
    ///
    /// 接続がアイドル状態になってから最初のKeep-Aliveパケットを送信するまでの時間を指定します。
    keepalive_time: Duration,
    /// The interval between subsequent keep-alive packets.
    ///
    /// This specifies the time between keep-alive packets after the first one is sent.
    ///
    /// 2回目以降のKeep-Aliveパケット送信間隔。
    ///
    /// 最初の送信以降、Keep-Aliveパケットを送信する間隔を指定します。
    keepalive_interval: Duration,
    /// 接続受け入れスレッド数。未設定時はワーカースレッド数の半分を使用。
    accept_threads: Option<usize>,
}

impl<E> KurosabiServerBuilder<E>
where E: Executor + Send + Sync + 'static {
    /// Creates a new `KurosabiServerBuilder` with default configuration.
    ///
    /// # Arguments
    /// * `worker` - An `Arc` containing the worker implementation.
    ///
    /// This initializes the server builder with default settings, which can be customized
    /// using the provided builder methods.
    ///
    /// デフォルト設定で`KurosabiServerBuilder`を作成します。
    ///
    /// # 引数
    /// * `worker` - ワーカー実装を格納した`Arc`。
    ///
    /// このメソッドは、ビルダーメソッドでカスタマイズ可能なデフォルト設定で初期化します。
    pub fn new(executor: E) -> KurosabiServerBuilder<E> {
        KurosabiServerBuilder {
            config: KurosabiConfig {
                host: [127, 0, 0, 1],
                port: 8080,
                thread: 4,
                thread_name: "kurosabi-worker".to_string(),
                queue_size: 128,
                reuse_address: true,
                nodelay: true,
                backlog: 2048,
                send_buffer_size: 64 * 1024,
                recv_buffer_size: 64 * 1024,
                keepalive_enabled: true,
                keepalive_time: Duration::from_secs(30),
                keepalive_interval: Duration::from_secs(10),
                accept_threads: None,
            },
            prc_executor: Arc::new(executor),
        }
    }

    /// Sets the host address for the server.
    /// 
    /// # Arguments
    /// * `host` - An array of 4 bytes representing the IPv4 address.
    /// 
    /// This specifies the address the server will bind to. For example, `[127, 0, 0, 1]` binds
    /// the server to localhost, while `[0, 0, 0, 0]` binds it to all available network interfaces.
    /// 
    /// サーバーのホストアドレスを設定します。
    /// 
    /// # 引数
    /// * `host` - IPv4アドレスを表す4バイトの配列。
    /// 
    /// このアドレスは、サーバーがバインドするアドレスを指定します。例: `[127, 0, 0, 1]`はローカルホスト、
    /// `[0, 0, 0, 0]`は全てのネットワークインターフェースにバインドします。
    pub fn host(mut self, host: [u8; 4]) -> Self {
        self.config.host = host;
        self
    }

    /// Sets the port number for the server.
    /// 
    /// # Arguments
    /// * `port` - The port number the server will listen on.
    /// 
    /// This determines the port where the server will accept incoming connections.
    /// 
    /// サーバーのポート番号を設定します。
    /// 
    /// # 引数
    /// * `port` - サーバーがリッスンするポート番号。
    /// 
    /// このポートは、サーバーが接続を受け付けるポートを指定します。
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Sets the number of threads for the server.
    ///     
    /// # Arguments
    /// * `thread` - The number of threads to use for the server.
    /// 
    /// This controls the number of worker threads that will handle incoming requests.
    /// 
    /// サーバーのスレッド数を設定します。
    /// 
    /// # 引数
    /// * `thread` - サーバーで使用するスレッド数。
    /// 
    /// このスレッド数は、リクエストを処理するワーカースレッドの数を指定します。
    pub fn thread(mut self, thread: usize) -> Self {
        self.config.thread = thread;
        self
    }

    /// Sets the name of the worker threads.
    /// 
    /// # Arguments
    /// * `thread_name` - The name of the worker threads.
    /// 
    /// スレッド名を設定します。
    /// 
    /// # 引数
    /// * `thread_name` - ワーカースレッドの名前。
    pub fn thread_name(mut self, thread_name: String) -> Self {
        self.config.thread_name = thread_name;
        self
    }

    /// Sets the size of the task queue for the worker pool.
    /// 
    /// # Arguments
    /// * `queue_size` - The size of the task queue.
    /// 
    /// This defines how many tasks can be queued for processing before new tasks are rejected.
    /// 
    /// タスクキューのサイズを設定します。
    /// 
    /// # 引数
    /// * `queue_size` - タスクキューのサイズ。
    /// 
    /// このサイズは、新しいタスクが拒否されるまでにキューできるタスク数を指定します。
    pub fn queue_size(mut self, queue_size: usize) -> Self {
        self.config.queue_size = queue_size;
        self
    }

    /// Sets the reuse address option for the server socket.
    ///
    /// # Arguments
    /// * `val` - A boolean indicating whether to enable reuse address.
    ///
    /// This allows the server to bind to an address that is in a TIME_WAIT state, which can
    /// be useful for quickly restarting the server.
    ///
    /// サーバーソケットのアドレス再利用オプションを設定します。
    ///
    /// # 引数
    /// * `val` - アドレス再利用を有効にするかどうか。
    ///
    /// 有効にすると、TIME_WAIT状態のアドレスにもバインドでき、サーバーの素早い再起動が可能です。
    pub fn reuse_address(mut self, val: bool) -> Self {
        self.config.reuse_address = val;
        self
    }

    /// Sets the nodelay option for the server socket.
    ///
    /// # Arguments
    /// * `val` - A boolean indicating whether to disable Nagle's algorithm.
    ///
    /// Disabling Nagle's algorithm can reduce latency for small packets by sending them immediately.
    ///
    /// サーバーソケットのNagleアルゴリズム無効化オプションを設定します。
    ///
    /// # 引数
    /// * `val` - Nagleアルゴリズムを無効にするかどうか。
    ///
    /// 無効化すると、小さなパケットを即時送信し、遅延を減らせます。
    pub fn nodelay(mut self, val: bool) -> Self {
        self.config.nodelay = val;
        self
    }

    /// Sets the backlog size for the server socket.
    ///
    /// # Arguments
    /// * `val` - The maximum number of pending connections.
    ///
    /// This sets the maximum number of connections that can be queued before the server starts rejecting new ones.
    ///
    /// サーバーソケットのバックログサイズを設定します。
    ///
    /// # 引数
    /// * `val` - 保留接続の最大数。
    ///
    /// サーバーが新しい接続を拒否するまでにキューできる最大接続数を指定します。
    pub fn backlog(mut self, val: usize) -> Self {
        self.config.backlog = val;
        self
    }

    /// Sets the size of the send buffer for the server socket.
    ///
    /// # Arguments
    /// * `val` - The size of the send buffer in bytes.
    ///
    /// This controls the amount of data that can be buffered for sending before the socket blocks.
    ///
    /// サーバーソケットの送信バッファサイズを設定します。
    ///
    /// # 引数
    /// * `val` - 送信バッファサイズ（バイト単位）。
    ///
    /// ソケットがブロックする前に送信できるデータ量を制御します。
    pub fn send_buffer_size(mut self, val: usize) -> Self {
        self.config.send_buffer_size = val;
        self
    }

    /// Sets the size of the receive buffer for the server socket.
    ///
    /// # Arguments
    /// * `val` - The size of the receive buffer in bytes.
    ///
    /// This controls the amount of data that can be buffered for receiving before the socket blocks.
    ///
    /// サーバーソケットの受信バッファサイズを設定します。
    ///
    /// # 引数
    /// * `val` - 受信バッファサイズ（バイト単位）。
    ///
    /// ソケットがブロックする前に受信できるデータ量を制御します。
    pub fn recv_buffer_size(mut self, val: usize) -> Self {
        self.config.recv_buffer_size = val;
        self
    }

    /// Enables or disables TCP keep-alive for the server socket.
    ///
    /// # Arguments
    /// * `val` - A boolean indicating whether to enable keep-alive.
    ///
    /// If enabled, the server will periodically send keep-alive packets to ensure the connection is still active.
    ///
    /// サーバーソケットのTCP Keep-Alive有効化オプションを設定します。
    ///
    /// # 引数
    /// * `val` - Keep-Aliveを有効にするかどうか。
    ///
    /// 有効にすると、サーバーは定期的にKeep-Aliveパケットを送信し、接続が生きているか確認します。
    pub fn keepalive_enabled(mut self, val: bool) -> Self {
        self.config.keepalive_enabled = val;
        self
    }

    /// Sets the idle time before the first keep-alive packet is sent.
    ///
    /// # Arguments
    /// * `val` - The duration of the idle time.
    ///
    /// This specifies how long the connection can remain idle before the first keep-alive packet is sent.
    ///
    /// 最初のKeep-Aliveパケット送信までのアイドル時間を設定します。
    ///
    /// # 引数
    /// * `val` - アイドル時間のDuration。
    ///
    /// 接続がアイドル状態になってから最初のKeep-Aliveパケットを送信するまでの時間を指定します。
    pub fn keepalive_time(mut self, val: Duration) -> Self {
        self.config.keepalive_time = val;
        self
    }

    /// Sets the interval between subsequent keep-alive packets.
    ///
    /// # Arguments
    /// * `val` - The duration of the interval.
    ///
    /// This specifies the time between keep-alive packets after the first one is sent.
    ///
    /// 2回目以降のKeep-Aliveパケット送信間隔を設定します。
    ///
    /// # 引数
    /// * `val` - 送信間隔のDuration。
    ///
    /// 最初の送信以降、Keep-Aliveパケットを送信する間隔を指定します。
    pub fn keepalive_interval(mut self, val: Duration) -> Self {
        self.config.keepalive_interval = val;
        self
    }

    /// Sets the number of accept threads.
    /// 未設定時はワーカースレッド数の半分を使用します。
    pub fn accept_threads(mut self, count: usize) -> Self {
        self.config.accept_threads = Some(count);
        self
    }

    /// build server
    /// new KurosabiServer instance
    /// 
    /// サーバーをビルドします
    /// 新しいKurosabiServerインスタンスを作成します
    pub fn build(self) -> KurosabiServer<E> {
        let thread_mum = self.config.thread;
        let queue_size = self.config.queue_size;
        let executor = Arc::clone(&self.prc_executor);
        // Create a new KurosabiServer instance with the provided configuration
        KurosabiServer {
            config: self.config,
            global_queue: Arc::new(ArrayQueue::new(queue_size)),
            workers_load: Arc::new(
                (0..thread_mum)
                    .map(|_| AtomicU64::new(0))
                    .collect::<Vec<_>>()
                    .into_boxed_slice()),
            workers: Vec::with_capacity(thread_mum),
            proc_executor: executor,
            _marker: std::marker::PhantomData, // Use PhantomData to indicate that E is used
        }
    }
}

impl<E> KurosabiServer<E> 
where E: Executor + Send + Sync + 'static {
    /// Starts the server and begins accepting connections.
    /// 
    /// サーバーを起動し、接続を受け付け始めます。
    pub fn run(&mut self) {
        let workers_load = Arc::clone(&self.workers_load); // Arcをクローン
        let proc_executor = Arc::clone(&self.proc_executor); // Arcをクローン
    
        for worker_id in 0..self.config.thread {
            let worker = DefaultWorker::new(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .thread_name(self.config.thread_name.clone())
                    .enable_all()
                    .build()
                    .expect("Failed to create Tokio runtime"),
                Arc::clone(&proc_executor), // クローンしたArcを渡す
                Arc::clone(&self.global_queue),
                workers_load.clone(), // クローンしたArcを渡す
                worker_id as u32, // worker_id
            );
            worker.run();
            self.workers.push(worker);
            info!("Worker {} started", worker_id);
        }

        let accept_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.accept_threads.unwrap_or(self.config.accept_threads.unwrap_or(1)))
            .thread_name("kurosabi-accept".to_string())
            .enable_all()
            .build()
            .expect("Failed to create accept runtime");

        info!(
            "Server starting on http://{}.{}.{}.{}:{}",
            self.config.host[0],
            self.config.host[1],
            self.config.host[2],
            self.config.host[3],
            self.config.port
        );
        // Show localhost URL if host is 0.0.0.0 or 127.x.x.x
        if self.config.host == [0, 0, 0, 0]
            || self.config.host[0] == 127
        {
            info!(
            "Server also accessible on http://localhost:{}",
            self.config.port
            );
        }

        accept_runtime.block_on(async move {
            let listener = self.create_configured_listener();
            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        info!("Accepted connection from {}", addr);
                        let connection = TcpConnection::new(socket);
                        let first_idle_worker = self.workers_load.iter().enumerate().find_map(|(worker_id, load)| {
                            if load.load(Relaxed) == 0 {
                                Some(worker_id)
                            } else {
                                None
                            }
                        });

                        if let Some(worker_id) = first_idle_worker {
                            self.workers[worker_id].execute(connection);
                        } else {
                            self.global_queue.push(connection).unwrap_or_else(|_| {
                                error!("Failed to push connection to global queue - queue is full");
                                println!("Failed to push connection to global queue - queue is full");
                            });
                        }
      
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });
    }

    fn create_configured_listener(&self) -> TcpListener {
        let addr = SocketAddr::from((self.config.host, self.config.port));
        let socket = Socket::new(Domain::IPV4, socket2::Type::STREAM, Some(Protocol::TCP)).unwrap();
        socket.set_reuse_address(self.config.reuse_address).unwrap();
        socket.set_nodelay(self.config.nodelay).unwrap();



        socket.bind(&addr.into()).unwrap();
        socket.set_reuse_address(self.config.reuse_address).unwrap();

        socket.set_send_buffer_size(self.config.send_buffer_size).unwrap(); // 送信バッファを設定
        socket.set_recv_buffer_size(self.config.recv_buffer_size).unwrap(); // 受信バッファを設定

        if self.config.keepalive_enabled {
            socket.set_tcp_keepalive(
                &TcpKeepalive::new()
                    .with_time(self.config.keepalive_time) // 最初のKeep-Alive送信までの時間
                    .with_interval(self.config.keepalive_interval) // Keep-Aliveパケットの間隔
            ).unwrap();
            socket.set_keepalive(true).unwrap();
        }
        socket.listen(self.config.backlog as i32).unwrap();
        let listener = tokio::net::TcpListener::from_std(socket.into()).unwrap();
        listener
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
