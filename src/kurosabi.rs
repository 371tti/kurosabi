use std::sync::Arc;

use crate::{context::Context, request::Req, response::Res, router::GenRouter, utils::header::Method};
    
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



