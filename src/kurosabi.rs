use std::{pin::Pin, sync::Arc};
use log::{debug, error, info, warn};

use crate::api::{GETJsonAPI, POSTJsonAPI};
use crate::error::HttpError;
use crate::{context::DefaultContext, request::Req, response::Res, router::{BoxedHandler, DefaultRouter, GenRouter}, server::{worker::Worker, KurosabiServerBuilder, TcpConnection}};
use crate::utils::method::Method;
pub struct Kurosabi<C, R>
where
    C: Clone + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    router: R,
    context: C
}

impl Kurosabi<DefaultContext, DefaultRouter<DefaultContext>> {
    pub fn new() -> Kurosabi<DefaultContext, DefaultRouter<DefaultContext>> {
        Kurosabi {
            router: DefaultRouter::new(),
            context: DefaultContext::new(),
        }
    }
}

impl<C> Kurosabi<C, DefaultRouter<C>>
where
    C: Clone + Sync + Send + 'static,
{
    /// コンテキストを指定して初期化する
    pub fn with_context(context: C) -> Kurosabi<C, DefaultRouter<C>> {
        Kurosabi {
            router: DefaultRouter::new(),
            context,
        }
    }
}

impl<C, R> Kurosabi<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    /// コンテキストとルーターを指定して初期化する
    pub fn with_context_and_router(context: C, router: R) -> Kurosabi<C, R> {
        Kurosabi {
            router,
            context,
        }
    }

    #[inline]
    fn register_route<F, Fut>(&mut self, method: Method, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        let boxed_handler: Box<
            dyn Fn(
                Context<C>,
            ) -> Pin<Box<dyn std::future::Future<Output = Context<C>> + Send + 'static>>
                + Send
                + Sync,
        > = Box::new(move |c| Box::pin(handler(c)));
        self.router.regist(method, pattern, std::sync::Arc::new(boxed_handler));
    }

    pub fn get<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::GET, pattern, handler);
    }

    pub fn post<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::POST, pattern, handler);
    }

    pub fn put<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::PUT, pattern, handler);
    }

    pub fn delete<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::DELETE, pattern, handler);
    }

    pub fn patch<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::PATCH, pattern, handler);
    }

    pub fn options<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::OPTIONS, pattern, handler);
    }

    pub fn trace<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::TRACE, pattern, handler);
    }

    pub fn head<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::HEAD, pattern, handler);
    }

    pub fn connect<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::CONNECT, pattern, handler);
    }

    pub fn any<F, Fut>(&mut self, pattern: &str, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        self.register_route(Method::UNKNOWN("OTHER".to_string()), pattern, handler);
    }

    #[inline]
    pub fn not_found_handler<F, Fut>(&mut self, handler: F)
    where
        F: Fn(Context<C>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Context<C>> + Send + 'static,
    {
        let boxed_handler: Box<
            dyn Fn(
                Context<C>,
            ) -> Pin<Box<dyn std::future::Future<Output = Context<C>> + Send + 'static>>
                + Send
                + Sync,
        > = Box::new(move |c| Box::pin(handler(c)));
        self.router.regist_not_found(std::sync::Arc::new(boxed_handler));
    }

    #[inline]
    pub fn get_json_api<API, Rss>(&mut self, pattern: &str, api_struct: API)
    where
        C: Clone + Send + Sync + 'static,
        Rss: serde::Serialize,
        API: GETJsonAPI<Context<C>, Rss> + Send + Sync + 'static,
    {
        let api_struct = api_struct.clone();
        let handler = {
            let api_clone = api_struct.clone(); // 必要に応じてclone
            move |mut c: Context<C>| {
                // api_cloneを呼び出すためにここでもcloneする、または共有参照を使う
                let api = api_clone.clone(); 
                async move {
                    let res = api.handler(&mut c).await;
                    let serialized_res = serde_json::to_value(res).unwrap_or_default();
                    c.res.json_value(&serialized_res);
                    c
                }
            }
        };
        self.register_route(Method::GET, pattern, handler);
    }

    #[inline]
    pub fn post_json_api<API, Rqs, Rss>(&mut self, pattern: &str, api_struct: API)
    where
        C: Clone + Send + Sync + 'static,
        Rqs: for<'a> serde::Deserialize<'a>,
        Rss: serde::Serialize,
        API: POSTJsonAPI<Context<C>, Rqs, Rss> + Send + Sync + 'static,
    {
        let api_struct = api_struct.clone();
        let handler = {
            let api_clone = api_struct.clone(); // 必要に応じてclone
            move |mut c: Context<C>| {
                // api_cloneを呼び出すためにここでもcloneする、または共有参照を使う
                let api = api_clone.clone(); 
                async move {
                    let req_json_value = c.req.body_json().await.unwrap_or_default();
                    let req_json: Result<Rqs, serde_json::Error> = serde_json::from_value(req_json_value);
                    let res = api.handler(&mut c, req_json).await;
                    let serialized_res = serde_json::to_value(res).unwrap_or_default();
                    c.res.json_value(&serialized_res);
                    c
                }
            }
        };
        self.register_route(Method::POST, pattern, handler);
    }
    
    /// ルーターをビルドしてサーバーを生成する
    pub fn server(mut self) -> KurosabiServerBuilder<DefaultWorker<C, R>> {
        self.router.build();
        let worker = std::sync::Arc::new(DefaultWorker::new(std::sync::Arc::new(self.router), std::sync::Arc::new(self.context)));
        KurosabiServerBuilder::new(worker)
    }
}

pub struct DefaultWorker<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    router: Arc<R>,
    context: Arc<C>,
}

impl<C, R> DefaultWorker<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>> + 'static,
{
    pub fn new(router: Arc<R>, context: Arc<C>) -> DefaultWorker<C, R> {
        DefaultWorker {
            router,
            context,
        }
    }
}

pub struct Context<C> {
    pub req: Req,
    pub res: Res,
    pub c: Box<C>,
}

#[async_trait::async_trait]
impl<C, R> Worker for DefaultWorker<C, R>
where
    C: Clone + Sync + Send + 'static,
    R: GenRouter<Arc<BoxedHandler<C>>>,
{
    async fn execute(&self, connection: TcpConnection) {
        // Req は内部に TCP 接続を保持つ
        let mut req = Req::new(connection);
        
        loop {
            // 新しいリクエストが来るまで待機
            if let Err(e) = req.wait_request().await {
                error!("Failed to wait for request: {:?}", e);
                break;
            }
            // リクエストのタイミングを計測
            let rev_to_res_time = std::time::Instant::now();
            // HTTPリクエストのヘッダーをパース
            if let Err(e) = req.parse_headers().await {
                error!("Failed to parse headers: {:?}", e);
                break;
            }
            // リクエストのタイミングを計測
            let send_time = std::time::Instant::now();
            let ps_time = std::time::Instant::now();

            let method = req.method.to_str();
            let path = req.path.path.clone();
            let version = req.version.clone();
            let head_info = format!("{} {} {} ", method, path, version);
            
            // コンテキストをクローン
            let context_data: C = (*self.context).clone();
            // ルーティング
            if let Some(handler) = self.router.route(&mut req) {
                let res = Res::new();
                let mut context = Context {
                    req,
                    res,
                    c: Box::new(context_data),
                };
                
                // ハンドラを実行
                context = handler(context).await;

                let ps_time = ps_time.elapsed();
                
                // レスポンス送信
                if let Err(e) = context.res.flush(&mut context.req).await {
                    error!("Failed to flush response: {:?}", e);
                    break;
                }

                // レスポンスのタイミングを計測
                let send_time = send_time.elapsed();
                let rev_to_res_time = rev_to_res_time.elapsed();

                debug!("\ntime:\n\tall_time: {:?}\n\tsend_time: {:?}\n\tprocessing: {:?}", rev_to_res_time, send_time, ps_time);

                // ログ出力（レスポンスコードに応じて色分け）
                if context.res.code >= 500 {
                    error!("{}- \x1b[31m{}\x1b[0m", head_info, context.res.code);
                } else if context.res.code >= 400 {
                    warn!("{}- \x1b[33m{}\x1b[0m", head_info, context.res.code);
                } else if context.res.code >= 300 {
                    info!("{}- \x1b[34m{}\x1b[0m", head_info, context.res.code);
                } else if context.res.code >= 200 {
                    info!("{}- \x1b[32m{}\x1b[0m", head_info, context.res.code);
                } else {
                    info!("{}- \x1b[36m{}\x1b[0m", head_info, context.res.code);
                }
                
                // ヘッダーの内容から接続を閉じるべきか判断
                if should_close_connection(&context.req, &context.res) {
                    break;
                }
                
                // 次のリクエストを同じ接続で処理するため、Req を再利用
                req = context.req;
            } else {
                // ルーティングにヒットしなかった場合は 404 を返す
                let e = HttpError::NotFound;
                let mut res = e.err_res();
                res.text("404 Not Found (kurosabi router default err page)");
                if let Err(e) = res.flush(&mut req).await {
                    error!("Failed to flush 404 response: {:?}", e);
                }
                warn!("{}- \x1b[33m{}\x1b[0m\n{}", head_info, res.code, e);
            }
        }
        // すべてのリクエスト処理後、接続をクローズ
    }
}

/// 接続を閉じるか判断するための例
#[inline]
fn should_close_connection(req: &Req, res: &Res) -> bool {
    // HTTP/1.0の場合、明示的なKeep-Aliveがなければclose
    if req.version == "HTTP/1.0" && !req.header.get_connection().unwrap_or("close").eq_ignore_ascii_case("keep-alive") {
        return true;
    }
    // レスポンスで "Connection: close" が指定されている場合
    if res.header.get("Connection").unwrap_or("close").eq_ignore_ascii_case("close") {
        return true;
    }
    false
}