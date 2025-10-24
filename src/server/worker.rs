use std::panic::AssertUnwindSafe;
use std::sync::{atomic::AtomicU64, Arc};

use crossbeam_queue::ArrayQueue;
use futures::FutureExt;
use log::{error, warn};
use tokio::runtime::Handle;
use tokio::sync::Notify;

use crate::context::ContextMiddleware;

use super::TcpConnection;

use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::Ordering::SeqCst;
pub trait Worker<E, C>: Send + Sync
where
    E: Executor<C>,
    C: Clone + Sync + Send + 'static + ContextMiddleware<C>,
{
    fn new(runtime: Handle, executor: Arc<E>, grobal_queue: Arc<ArrayQueue<TcpConnection>>, workers_load: Arc<Box<[AtomicU64]>>, my_worker_id: u32) -> Self;
    fn execute(&self, connection: TcpConnection);
    fn run(&self);
}

pub struct DefaultWorker<E, C>
where
    C: Clone + Sync + Send + 'static + ContextMiddleware<C>,
    E: Executor<C>,
{
    pub executor: Arc<E>,
    pub runtime: Handle,
    pub notify: Arc<Notify>,
    pub grobal_queue: Arc<ArrayQueue<TcpConnection>>,
    pub workers_load: Arc<Box<[AtomicU64]>>, // Fixed the syntax for Arc<Box<[AtomicU64]>
    pub worker_id: u32,
    phantom: std::marker::PhantomData<C>,
}

struct LoadGuard<'a>(&'a AtomicU64);
impl<'a> LoadGuard<'a> {
    #[inline]
    fn new(counter: &'a AtomicU64) -> Self {
        counter.fetch_add(1, SeqCst);
        Self(counter)
    }
}
impl<'a> Drop for LoadGuard<'a> {
    #[inline]
    fn drop(&mut self) {
        self.0.fetch_sub(1, SeqCst);
    }
}

impl<E, C> Worker<E, C> for DefaultWorker<E, C>
where 
    C: Clone + Sync + Send + 'static + ContextMiddleware<C>,
    E: Executor<C> + Send + Sync + 'static,
{
    fn new(runtime: Handle, executor: Arc<E>, grobal_queue: Arc<ArrayQueue<TcpConnection>>, workers_load: Arc<Box<[AtomicU64]>>, worker_id: u32) -> Self {
        DefaultWorker {
            runtime: runtime,
            executor,
            notify: Arc::new(Notify::new()),
            grobal_queue,
            workers_load,
            worker_id,
            phantom: std::marker::PhantomData,
        }
    }

    fn execute(&self, connection: TcpConnection) {
        self.notify.notify_one(); // 必ず通知
        let my_load = Arc::clone(&self.workers_load); // Arc<[AtomicU64]>をクローン
        let executor = Arc::clone(&self.executor);
        let worker_id = self.worker_id;
    
        self.runtime.spawn(async move {
            let _guard = LoadGuard::new(&my_load[worker_id as usize]);
            let res = AssertUnwindSafe(executor.execute(connection))
                .catch_unwind()
                .await;
            if let Err(panic_info) = res {
                error!("{:?} {}", panic_info, "handler panicked — connection closed");
            }
        });
    }

    fn run(&self) {
        let rt = self.runtime.clone();
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
                    if let Some(connection) = grobal_queue.pop() {
                        let cloned_workers_load = Arc::clone(&workers_load);
                        let executor_clone = Arc::clone(&executor);
                        rt.spawn(async move {
                            let _guard = LoadGuard::new(&cloned_workers_load[worker_id as usize]);
                            let res = AssertUnwindSafe(executor_clone.execute(connection))
                                .catch_unwind()
                                .await;
                            if let Err(panic_info) = res {
                                error!("{:?} {}", panic_info, "handler panicked — connection closed");
                            }
                        });
                    } else {
                        warn!("Failed to pop connection from global queue");
                    }
                }
            }
        });
    }
}

#[async_trait::async_trait]
pub trait Executor<C>: Send + Sync
where C: Clone + Sync + Send + 'static + ContextMiddleware<C> {
    async fn execute(&self, connection: TcpConnection);
    async fn init(&self);
}