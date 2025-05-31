use std::sync::{atomic::AtomicU64, Arc};

use crossbeam_queue::ArrayQueue;
use log::warn;
use tokio::{runtime::Runtime, sync::Notify};

use crate::server::worker::{Executor, Worker};

use super::TcpConnection;

use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::Ordering::SeqCst;

pub struct DefaultWorker<E>
where E: Executor {
    pub executor: Arc<E>,
    pub runtime: Arc<Runtime>,
    pub notify: Arc<Notify>,
    pub grobal_queue: Arc<ArrayQueue<TcpConnection>>,
    pub workers_load: Arc<Box<[AtomicU64]>>, // Fixed the syntax for Arc<Box<[AtomicU64]>
    pub worker_id: u32,
}

impl<E> Worker<E> for DefaultWorker<E>
where E: Executor + Send + Sync + 'static {
    fn new(runtime: Runtime, executor: Arc<E>, grobal_queue: Arc<ArrayQueue<TcpConnection>>, workers_load: Arc<Box<[AtomicU64]>>, worker_id: u32) -> Self {
        DefaultWorker {
            runtime: Arc::new(runtime),
            executor,
            notify: Arc::new(Notify::new()),
            grobal_queue,
            workers_load,
            worker_id,
        }
    }

    fn execute(&self, connection: TcpConnection) {
        self.notify.notify_one(); // 必ず通知
        let my_load = Arc::clone(&self.workers_load); // Arc<[AtomicU64]>をクローン
        let executor = Arc::clone(&self.executor);
        let runtime = Arc::clone(&self.runtime);
        let worker_id = self.worker_id;
    
        runtime.spawn(async move {
            let my_load = &my_load[worker_id as usize]; // AtomicU64への参照を取得
            my_load.fetch_add(1, SeqCst);
            executor.execute(connection).await;
            my_load.fetch_sub(1, SeqCst);
        });
    }

    fn run(&self) {
        let rt = Arc::clone(&self.runtime);
        let worker_num = self.workers_load.len() as u64;
        let workers_load = self.workers_load.clone();
        let grobal_queue = Arc::clone(&self.grobal_queue);
        let worker_id = self.worker_id;
        let executor = Arc::clone(&self.executor);
        let notify = Arc::clone(&self.notify);

        self.runtime.spawn(async move {
            loop {
                notify.notified().await;
                while workers_load[worker_id as usize].load(Relaxed) > 0 {
                    // タスクキューが空の場合は、ランタイムに帰還
                    if grobal_queue.is_empty() {
                        tokio::task::yield_now().await;
                        continue;
                    }

                    let my_load = workers_load[worker_id as usize].load(Relaxed);
        
                    // 自分の負荷が他のワーカーの合計負荷を超えている場合は、スリープする
                    let mut load_sum = 1;
                    for load in workers_load.iter() {
                        load_sum += load.load(Relaxed);
                    }
                    if my_load * worker_num > load_sum {
                        tokio::task::yield_now().await;
                        continue;
                    }
        
                    // タスクキューからコネクションを取り出して処理する
                    let executor_clone = Arc::clone(&executor); // 非同期タスク内でクローン
                    if let Some(connection) = grobal_queue.pop() {
                        rt.spawn(async move {
                            executor_clone.execute(connection).await;
                        });
                    } else {
                        warn!("Failed to pop connection from global queue");
                    }
                }
            }
        });
    }
}