use std::{collections::VecDeque, sync::{atomic::AtomicU64, Arc}};

use crossbeam_queue::ArrayQueue;
use futures::executor;
use tokio::runtime::Runtime;

use crate::server::worker::{self, Executor, Worker};

use super::TcpConnection;

use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::Ordering::SeqCst;

pub struct DefaultWorker<E>
where E: Executor {
    pub executor: Arc<E>,
    pub runtime: Arc<Runtime>,
    pub grobal_queue: Arc<ArrayQueue<TcpConnection>>,
    pub workers_load: &'static [AtomicU64],
    pub worker_id: u32,
}

impl<E> Worker<E> for DefaultWorker<E>
where E: Executor + Send + Sync + 'static {
    fn new(runtime: Runtime, executor: Arc<E>, grobal_queue: Arc<ArrayQueue<TcpConnection>>, workers_load: &'static [AtomicU64], worker_id: u32) -> Self {
        DefaultWorker {
            runtime: Arc::new(runtime),
            executor,
            grobal_queue,
            workers_load,
            worker_id,
        }
    }

    fn execute(&self,connection:TcpConnection) {
        let my_load = &self.workers_load[self.worker_id as usize];
        let executor = Arc::clone(&self.executor);
        self.runtime.spawn(async move {
            my_load.fetch_add(1, SeqCst);
            executor.execute(connection).await;
            my_load.fetch_sub(1, SeqCst);
        });
    }

    fn run(&self) {
        let rt = Arc::clone(&self.runtime);
        let worker_num = self.workers_load.len() as u64;
        let workers_load = self.workers_load;
        let grobal_queue = Arc::clone(&self.grobal_queue);
        let my_load = self.workers_load[self.worker_id as usize].load(Relaxed);
        let executor = Arc::clone(&self.executor);
        self.runtime.spawn(async move {
            loop {
                // タスクキューが空の場合は、ワーカーをスリープさせる
                if grobal_queue.is_empty() {
                    tokio::task::yield_now().await;
                    continue;
                }
    
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
                let connection = grobal_queue.pop().expect("Failed to pop connection from global queue");
                rt.spawn(async move {
                    executor_clone.execute(connection).await;
                });
            }
        });
    }
}