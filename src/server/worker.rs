use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use log::error;
use tokio::sync::Notify;

use super::TcpConnection;

pub struct WorkerPool<W> {
    task_queue: Arc<ArrayQueue<TcpConnection>>, // ロックフリーのタスクキュー
    pub notifier: Arc<Notify>, // ワーカーを起こすための通知
    pub worker: Arc<W>,
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
            // タスクキューからコネクションを取り出して処理する
            while let Some(connection) = self.task_queue.pop() {
                self.handle_connection(connection).await;
                
            }

            // タスクキューが空の場合は、ワーカーをスリープさせる
            self.notifier.notified().await;
        }
    }

    #[inline]
    async fn handle_connection(&self, connection: TcpConnection) {
        // ここでリクエストを処理する
        self.worker.execute(connection).await;
    }

    #[inline]
    pub async fn assign_connection(&self, connection: TcpConnection) {
        if self.task_queue.push(connection).is_ok() {
            self.notifier.notify_one();
        } else {
            error!("Failed to assign connection to worker - queue is full");
        }
    }

    #[inline]
    pub fn notifier(&self) -> Arc<Notify> {
        Arc::clone(&self.notifier)
    }
}

#[async_trait::async_trait]
pub trait Worker: Send + Sync {
    async fn execute(&self, connection: TcpConnection);
}