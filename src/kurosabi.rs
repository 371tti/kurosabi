use std::{io::{self, Result}, sync::Arc};
use tokio::io::{AsyncBufReadExt, BufReader};

use tokio::net::TcpStream;

use crate::{context::Context, error::KurosabiError, request::Req, response::Res, router::GenRouter, server::{worker::Worker, KurosabiServer, TcpConnection}, utils::header::Method};
    
pub struct Kurosabi<C, Router> {
    context: Arc<C>,
    router: Arc<Router>
}

/// 初期化およびインスタンス操作を行うためのメソッドぐん
/// 
impl<C, Router> Kurosabi<C, Router>
where C: Context,
    Router: GenRouter,
{
    pub fn new(context: Arc<C>, router: Arc<Router>) -> Kurosabi<C, Router> {
        Kurosabi {
            context,
            router
        }
    }

}

/// レジストリ操作メソッドたち
/// 
impl<C, Router> Kurosabi<C, Router> {
    /// httpのGETメソッドに対するルーティングを登録する
    pub fn get<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのPOSTメソッドに対するルーティングを登録する
    pub fn post<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのHEADメソッドに対するルーティングを登録する
    pub fn head<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのPUTメソッドに対するルーティングを登録する
    pub fn put<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのDELETEメソッドに対するルーティングを登録する
    pub fn delete<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのOPTIONSメソッドに対するルーティングを登録する
    pub fn options<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのTRACEメソッドに対するルーティングを登録する
    pub fn trace<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのCONNECTメソッドに対するルーティングを登録する
    pub fn connect<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    /// httpのPATCHメソッドに対するルーティングを登録する
    pub fn patch<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    pub fn some_method<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    pub fn before<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }

    pub fn after<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req, Arc<C>) -> Res
    {

    }
}

impl<C, Router> Kurosabi<C, Router>
where C: Context,
    Router: GenRouter,
{
    pub fn server(&mut self) -> KurosabiServer<DefaultWorker> {
        let worker = Arc::new(self.generate_worker());
        KurosabiServer::new(worker)
    }

    pub fn generate_worker(&mut self) -> DefaultWorker {
        DefaultWorker {}
    }
}


pub struct DefaultWorker {

}

impl DefaultWorker {
    async fn http_reader_head(reader: &mut BufReader<TcpStream>) -> std::result::Result<Req, KurosabiError> {
       let mut req = Req::new();

       let mut line_buf = String::new();
        
        reader.read_line(&mut line_buf).await.map_err(|e| KurosabiError::IoError(e))?;

        let parts: Vec<&str> = line_buf.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(KurosabiError::InvalidHttpHeader(line_buf));
        }
    
        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let http_version = parts[2].to_string();

        req.method = Method::from_str(&method).unwrap();
        req.path.path = path;

        loop {
            line_buf.clear();
            reader.read_line(&mut line_buf).await.map_err(|e| KurosabiError::IoError(e))?;
            let trimmed = line_buf.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some((key, value)) = trimmed.split_once(": ") {
                req.header.set(key, value);
            } else {
                return Err(KurosabiError::InvalidHttpHeader(line_buf));
            }
        }
        
        Ok(req)
    }
}

#[async_trait::async_trait]
impl Worker for DefaultWorker {
    async fn execute(&self, connection: TcpConnection) {
        let mut reader = BufReader::new(connection.socket);
        
    }
}
