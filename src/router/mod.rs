use std::sync::Arc;

use crate::context::Context;
use crate::error::HttpError;
use crate::{context::DefaultContext, request::Req, response::Res, utils::header::Method};
use regex::Regex;

pub trait GenRouter<C, F>: Send + Sync
where 
    C: Context + 'static,
    F: Send + Sync + 'static
{
    /// ルーティングテーブルに登録する
    /// 原則buildメソッド呼び出し後は呼び出さないように
    fn regist(&mut self, method: Method, pattern: &str, excuter: F) -> ();

    /// ルーティングテーブルを構築したり
    fn build(&mut self) -> ();

    /// ルーティングを行う
    fn route(&self, req: &mut Req, context: &mut C) -> Option<F>;
}


use std::future::Future;
use std::pin::Pin;

/// 非同期関数を返すハンドラの型エイリアス
pub type BoxedHandler = Box<dyn Fn(Req, Res, Box<dyn Context>) -> Pin<Box<dyn Future<Output = Result<Res, HttpError>> + Send>> + Send + Sync>;


pub struct DefaultRouter {
    table: Vec<(Regex, Arc<BoxedHandler>)>
}

impl DefaultRouter {
    pub fn new() -> DefaultRouter {
        DefaultRouter {
            table: Vec::new()
        }
    }
}

impl GenRouter<DefaultContext<String>, Arc<BoxedHandler>> for DefaultRouter {
    fn regist(&mut self, method: Method, pattern: &str, excuter: Arc<BoxedHandler>) {
        let method_str = method.to_str();
        let regex = Regex::new(pattern).unwrap();
        self.table.push((regex, excuter));
    }

    fn build(&mut self) -> () {
    }

    fn route(&self, req: &mut Req, context: &mut DefaultContext<String>) -> Option<Arc<BoxedHandler>> {
        let method = req.method.to_str();
        let path = req.path.path.clone();
        
        for (regex, excuter) in &self.table {
            if regex.is_match(&format!("{} {}", method, path)) {
                return Some(Arc::clone(excuter));
            }
        }
        
        None
    }
}