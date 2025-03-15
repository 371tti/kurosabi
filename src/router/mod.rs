use std::sync::Arc;

use crate::error::HttpError;
use crate::kurosabi::Context;
use crate::{context::DefaultContext, request::Req, response::Res, utils::header::Method};
use regex::Regex;

pub trait GenRouter<F>: Send + Sync
where 
    F: Send + Sync + 'static
{
    /// ルーティングテーブルに登録する
    /// 原則buildメソッド呼び出し後は呼び出さないように
    fn regist(&mut self, method: Method, pattern: &str, excuter: F) -> ();

    /// ルーティングテーブルを構築したり
    fn build(&mut self) -> ();

    /// ルーティングを行う
    fn route(&self, req: &mut Req) -> Option<F>;
}


use std::future::Future;
use std::pin::Pin;

/// 非同期関数を返すハンドラの型エイリアス
pub type BoxedHandler<C> = Box<dyn Fn(Context<C>) -> Pin<Box<dyn Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send>> + Send + Sync>;


pub struct DefaultRouter<C> {
    table: Vec<(Regex, Arc<BoxedHandler<C>>)>
}

impl<C> DefaultRouter<C> {
    pub fn new() -> DefaultRouter<C> {
        DefaultRouter {
            table: Vec::new()
        }
    }
}

impl<C: 'static> GenRouter<Arc<BoxedHandler<C>>> for DefaultRouter<C> {
    fn regist(&mut self, method: Method, pattern: &str, excuter: Arc<BoxedHandler<C>>) {
        let method_str = method.to_str();
        let paragraph = pattern.split('/').map(|s| {
            if s.starts_with(':') {
                format!("(?P<{}>[^/]+)", &s[1..])
            } else {
                s.to_string()
            }
        }).collect::<Vec<String>>().join("/");
        let regex_pattern = format!("^{} {}$", method_str, paragraph); // 末尾に $ を追加
        let regex = Regex::new(&regex_pattern).unwrap();
        self.table.push((regex, excuter));
    }

    fn build(&mut self) -> () {
        // ルーティングテーブルの構築処理を追加する場合はここに記述
    }

    fn route(&self, req: &mut Req) -> Option<Arc<BoxedHandler<C>>> {
        let method = req.method.to_str();
        let path = req.path.path.clone();
        let target = format!("{} {}", method, path);

        for (regex, excuter) in &self.table {
            if let Some(captures) = regex.captures(&target) {
                for name in regex.capture_names().flatten() {
                    if let Some(value) = captures.name(name) {
                        req.path.set_field(name, value.as_str());
                    }
                }
                return Some(Arc::clone(excuter));
            }
        }
        None
    }
}