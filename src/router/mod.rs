//! radix_router.rs
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
};

use crate::{error::HttpError, kurosabi::Context, request::Req, utils::method::Method};

/// ---------- 共通トrait ----------
pub trait GenRouter<F>: Send + Sync
where
    F: Send + Sync + 'static,
{
    fn regist(&mut self, method: Method, pattern: &str, excuter: F);
    fn build(&mut self);
    fn route(&self, req: &mut Req) -> Option<F>;
}

/// ---------- ハンドラ型 ----------
pub type BoxedHandler<C> = Box<
    dyn Fn(Context<C>)
        -> Pin<
            Box<dyn Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send>,
        > + Send
        + Sync,
>;

/// ------------- Radix tree 実装 -------------
#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Static,   // /foo
    Param,    // :id
    Wildcard, // *
}

/// 圧縮ノード
struct Node<C> {
    kind: Kind,
    label: String,                       // Static または Param 部分
    param_name: Option<String>,          // Param 名
    fixed: HashMap<u8, Box<Node<C>>>,    // 先頭バイト→子
    param_child: Option<Box<Node<C>>>,   // :param
    wild_child: Option<Box<Node<C>>>,    // *
    handler: Option<Arc<BoxedHandler<C>>>,
}

impl<C> Node<C> {
    fn new(kind: Kind, label: String) -> Self {
        Node {
            kind,
            label,
            param_name: None,
            fixed: HashMap::new(),
            param_child: None,
            wild_child: None,
            handler: None,
        }
    }
}

/// ルーター本体
pub struct DefaultRouter<C> {
    trees: HashMap<Method, Box<Node<C>>>,
    sealed: bool,
}

impl<C> DefaultRouter<C> {
    pub fn new() -> Self {
        DefaultRouter {
            trees: HashMap::new(),
            sealed: false,
        }
    }
}

// ----------- GenRouter 実装 -----------
impl<C: 'static> GenRouter<Arc<BoxedHandler<C>>> for DefaultRouter<C> {
    /// ① 登録
    fn regist(&mut self, method: Method, pattern: &str, excuter: Arc<BoxedHandler<C>>) {
        assert!(
            !self.sealed,
            "`build()` 後には regist できません。"
        );

        // 先頭 '/' を除去して走査用スライスを得る
        let mut path = pattern.trim_start_matches('/');
        
        let panic_method = method.clone();
        // ルートノード
        let root = self
            .trees
            .entry(method)
            .or_insert_with(|| Box::new(Node::new(Kind::Static, String::new())));
        let mut node = root.as_mut();

        loop {
            // 末尾に到達
            if path.is_empty() {
                if node.handler.is_some() {
                    panic!("duplicate route: {panic_method} {pattern}");
                }
                node.handler = Some(excuter);
                return;
            }

            match path.as_bytes()[0] {
                b':' => {
                    // :param
                    let end = path.find('/').unwrap_or(path.len());
                    let name = &path[1..end];
                    path = &path[end..].trim_start_matches('/');

                    let child = node
                        .param_child
                        .get_or_insert_with(|| Box::new(Node::new(Kind::Param, String::new())));
                    child.param_name = Some(name.to_string());
                    node = child.as_mut();
                }
                b'*' => {
                    // * (末尾専用とする)
                    node = node
                        .wild_child
                        .get_or_insert_with(|| Box::new(Node::new(Kind::Wildcard, String::new())));
                    // 残り全部を * が飲み込む
                    path = "";
                }
                _ => {
                    // Static セグメントを抜き出す (':' '/' '*' 手前まで)
                    let mut i = 0;
                    while i < path.len() {
                        match path.as_bytes()[i] {
                            b':' | b'*' | b'/' => break,
                            _ => i += 1,
                        }
                    }
                    let (seg, rest) = path.split_at(i);
                    path = rest.trim_start_matches('/');

                    // 先頭バイトで子検索（Radix node 接頭辞圧縮）
                    let key = seg.as_bytes()[0];
                    let child = node.fixed.entry(key).or_insert_with(|| {
                        Box::new(Node::new(Kind::Static, seg.to_string()))
                    });

                    // 共有 prefix 長を比較
                    let lcp = child
                        .label
                        .bytes()
                        .zip(seg.bytes())
                        .take_while(|(a, b)| a == b)
                        .count();
                    if lcp < child.label.len() {
                        // 既存 child を分割
                        let suffix = child.label[lcp..].to_string();
                        let mut split = Node::new(Kind::Static, suffix);
                        std::mem::swap(&mut split.fixed, &mut child.fixed);
                        split.handler = child.handler.take();
                        child.label.truncate(lcp);
                        child
                            .fixed
                            .insert(split.label.as_bytes()[0], Box::new(split));
                    }
                    // 未消化部分を続ける
                    if seg.len() > lcp {
                        // 分割後の新 suffix を child の先に付ける
                        let suffix = &seg[lcp..];
                        let new_child = child
                            .fixed
                            .entry(suffix.as_bytes()[0])
                            .or_insert_with(|| {
                                Box::new(Node::new(Kind::Static, suffix.to_string()))
                            });
                        node = new_child.as_mut();
                    } else {
                        node = child.as_mut();
                    }
                }
            }
        }
    }

    /// ② ビルド（木の凍結）
    fn build(&mut self) {
        if self.sealed {
            return;
        }
        fn dfs<C>(n: &mut Node<C>) {
            n.fixed.shrink_to_fit();
            if let Some(c) = &mut n.param_child {
                dfs(c);
            }
            if let Some(c) = &mut n.wild_child {
                dfs(c);
            }
            for c in n.fixed.values_mut() {
                dfs(c);
            }
        }
        for root in self.trees.values_mut() {
            dfs(root);
        }
        self.sealed = true;
    }

    /// ③ ルーティング
    fn route(&self, req: &mut Req) -> Option<Arc<BoxedHandler<C>>> {
        // ① path を自前の String として確保
        let full_path = req.path.path.clone();
        let mut i = 0;
        let mut node = match self.trees.get(&req.method) {
            Some(n) => n.as_ref(),
            None => return None,
        };
    
        // &str は full_path 由来なので req.path とは独立
        let path = full_path.trim_start_matches('/');

        while i <= path.len() {
            // 終端判定
            if i == path.len() {
                return node.handler.clone();
            }

            // Static マッチ
            if let Some(child) = node.fixed.get(&path.as_bytes()[i]) {
                if path[i..].starts_with(&child.label) {
                    i += child.label.len();
                    if i < path.len() && path.as_bytes()[i] == b'/' {
                        i += 1;
                    }
                    node = child;
                    continue;
                }
            }

            // Param マッチ
            if let Some(child) = &node.param_child {
                let start = i;
                while i < path.len() && path.as_bytes()[i] != b'/' {
                    i += 1;
                }
                if let Some(pn) = &child.param_name {
                    req.path
                        .set_field(pn, &path[start..i]); // パラメータ格納
                }
                if i < path.len() && path.as_bytes()[i] == b'/' {
                    i += 1;
                }
                node = child;
                continue;
            }

            // Wildcard マッチ
            if let Some(child) = &node.wild_child {
                req.path.set_field("*", &path[i..]);
                node = child;
                i = path.len();
                continue;
            }

            // 失敗
            return None;
        }
        None
    }
}
