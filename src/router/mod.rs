use std::sync::Arc;

use crate::error::HttpError;
use crate::kurosabi::Context;
use crate::{request::Req, utils::header::Method};
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
        
        // パスの先頭の "/" を除去した上でセグメント毎に処理
        let segments: Vec<&str> = pattern.split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let mut regex_str = String::new();
        // ルーティング対象は "METHOD /path" 形式なので、先頭にスペースと"/"を付加
        regex_str.push_str(" ");
        for segment in segments.iter() {
            // セグメントごとに先頭の "/" を付加
            regex_str.push('/');
            if segment.starts_with(':') {
                if segment.ends_with('?') && segment.len() > 2 {
                    // オプショナルパラメータ：例 "/:id?" → (?P<id>[^/]+)?
                    let name = &segment[1..segment.len()-1];
                    regex_str.push_str(&format!("(?P<{}>[^/]+)?", name));
                } else {
                    // 必須パラメータ：例 "/:id" → (?P<id>[^/]+)
                    let name = &segment[1..];
                    regex_str.push_str(&format!("(?P<{}>[^/]+)", name));
                }
            } else if *segment == "*" {
                // ワイルドカード：内部では "wildcard" としてキャプチャし、後で "*" として設定する
                regex_str.push_str("(?P<wildcard>.*)");
            } else {
                // 固定パスセグメントは正規表現用にエスケープ
                regex_str.push_str(&regex::escape(segment));
            }
        }
        
        // 全体の正規表現パターンは "METHOD" とスペース、その後にパスが続く形
        let full_regex_pattern = format!("^{}{}$", method_str, regex_str);
        let regex = Regex::new(&full_regex_pattern).unwrap();
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
                        if name == "wildcard" {
                            // ワイルドカードキャプチャはキー "*" として保存
                            req.path.set_field("*", value.as_str());
                        } else {
                            req.path.set_field(name, value.as_str());
                        }
                    }
                }
                return Some(Arc::clone(excuter));
            }
        }
        None
    }
}