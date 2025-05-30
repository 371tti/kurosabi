use std::sync::{atomic::AtomicU64, Arc};

use crossbeam_queue::ArrayQueue;
use log::{debug, error};
use tokio::sync::Notify;

use super::TcpConnection;

pub struct WorkerPool<W> {
    workers: Arc<W>
}

impl<W: Worker> WorkerPool<W> {
    pub fn new(queue_size: usize, worker: Arc<W>) -> WorkerPool<W> {
        WorkerPool {
            task_queue: Arc::new(ArrayQueue::new(queue_size)),
            notifier: Arc::new(Notify::new()),
            worker: worker,
        }
    }

    #[inline]
    pub async fn main_loop(self: Arc<Self>) {
        loop {
            // タスクキューが空の場合は、ワーカーをスリープさせる
            self.notifier.notified().await;

            // タスクキューからコネクションを取り出して処理する
            while let Some(connection) = self.task_queue.pop() {
                debug!("Created new connection");
                self.handle_connection(connection).await;
            }
        }
    }

    #[inline]
    async fn handle_connection(&self, connection: TcpConnection) {
        // ここでリクエストを処理する
        self.worker.execute(connection).await;
    }

    #[inline]
    pub async fn assign_connection(&self, connection: TcpConnection) -> bool {
        if self.task_queue.push(connection).is_ok() {
            // 通知してワーカーを起こす
            self.notifier.notify_one();
            true
        } else {
            error!("Failed to assign connection to worker - queue is full");
            false
        }
    }

    #[inline]
    pub fn notifier(&self) -> Arc<Notify> {
        Arc::clone(&self.notifier)
    }
}

#[async_trait::async_trait]
pub trait Worker: Send + Sync {
    /// ワーカーにグローバルキューを渡す
    async fn set_global_queue(&self, queue: Arc<ArrayQueue<TcpConnection>>);
    // 各ワーカーの負荷情報を共有する
    async fn set_workers_load_info(&mut self, load_info: Arc<[AtomicU64]>);
}