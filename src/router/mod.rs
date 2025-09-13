//! Simple & Fast router  ― Radix 無し版
//! 同じトレイト (GenRouter) で差し替え可能
//! ・完全一致は AHashMap
//! ・動的ルートは seg.length 毎の Vec で線形マッチ
//! ・ワイルドカード * は末尾限定

use std::{future::Future, pin::Pin, sync::Arc};

use ahash::AHashMap as Map;

use crate::{kurosabi::Context, request::Req, utils::method::Method};

pub trait GenRouter<F>: Send + Sync
where
    F: Send + Sync + 'static,
{
    fn regist(&mut self, method: Method, pattern: &str, excuter: F);
    fn regist_not_found(&mut self, excuter: F);
    fn build(&mut self);
    fn route(&self, req: &mut Req) -> Option<F>;
}

pub type BoxedHandler<C> = Box<
    dyn Fn(Context<C>) -> Pin<Box<dyn Future<Output = Context<C>> + Send>>
        + Send
        + Sync,
>;

#[derive(Clone)]
struct Pattern<C> {
    segs: Vec<Seg>,
    handler: Arc<BoxedHandler<C>>,
}
#[derive(Clone)]
enum Seg {
    Lit(String),
    Param(String),
    Wild,
}

pub struct DefaultRouter<C> {
    exact: Map<(Method, String), Arc<BoxedHandler<C>>>,
    fuzzy: Map<(Method, usize), Vec<Pattern<C>>>,
    // 末尾ワイルドカード用（可変長マッチ）
    wild: Map<Method, Vec<Pattern<C>>>,
    not_found: Option<Arc<BoxedHandler<C>>>,
}

impl<C> DefaultRouter<C> {
    pub fn new() -> Self {
        Self {
            exact: Map::default(),
            fuzzy: Map::default(),
            wild: Map::default(),
            not_found: None,
        }
    }
}

impl<C: 'static> GenRouter<Arc<BoxedHandler<C>>> for DefaultRouter<C> {
    fn regist_not_found(&mut self, ex: Arc<BoxedHandler<C>>) {
        self.not_found = Some(ex);
    }

    fn regist(&mut self, method: Method, pattern: &str, ex: Arc<BoxedHandler<C>>) {
        let path = pattern.trim_start_matches('/').to_string();

        if !path.contains([':', '*']) {
            if self.exact.insert((method.clone(), path), ex).is_some() {
                panic!("duplicate route: {method:?} {pattern}");
            }
            return;
        }

        let mut segs = Vec::new();
        let mut has_wild = false;
        for (i, s) in path.split('/').enumerate() {
            match s.as_bytes()[0] {
                b':' => segs.push(Seg::Param(s[1..].into())),
                b'*' => {
                    assert!(i == path.split('/').count() - 1,
                        "wildcard '*' must be terminal: {pattern}");
                    segs.push(Seg::Wild);
                    has_wild = true;
                }
                _ => segs.push(Seg::Lit(s.into())),
            }
        }

        if has_wild {
            self.wild.entry(method).or_default().push(Pattern { segs, handler: ex });
        } else {
            self.fuzzy.entry((method, segs.len()))
                .or_default()
                .push(Pattern { segs, handler: ex });
        }
    }

    fn build(&mut self) {
        // No-op for static structure
    }

    fn route(&self, req: &mut Req) -> Option<Arc<BoxedHandler<C>>> {
        let clean_path: String;
        {
            let tmp = &req.path.path;
            clean_path = tmp.split(&['?', '#']).next().unwrap()
                            .trim_start_matches('/')
                            .to_owned();
        }

        // 完全一致のルートをハンドル
        if let Some(h) = self.exact.get(&(req.method.clone(), clean_path.clone())) {
            return Some(h.clone());
        }

        // セグメントへ（空要素は除外して正規化）
        let segs: Vec<&str> = clean_path.split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // 動的ルート（ワイルドカード無し）
        if let Some(pats) = self.fuzzy.get(&(req.method.clone(), segs.len())) {
            'outer_fuzzy: for pat in pats {
                for (pseg, iseg) in pat.segs.iter().zip(&segs) {
                    match pseg {
                        Seg::Lit(l)   => if l != iseg { continue 'outer_fuzzy; },
                        Seg::Param(n) => req.path.set_field(n, iseg),
                        Seg::Wild     => unreachable!(),
                    }
                }
                return Some(pat.handler.clone());
            }
        }

        // 末尾ワイルドカード（可変長、0個以上）
        if let Some(pats) = self.wild.get(&req.method) {
            'outer_wild: for pat in pats {
                let prefix_len = pat.segs.len().saturating_sub(1);
                if segs.len() < prefix_len { continue 'outer_wild; }

                for (i, pseg) in pat.segs.iter().take(prefix_len).enumerate() {
                    match pseg {
                        Seg::Lit(l)   => if l != &segs[i] { continue 'outer_wild; },
                        Seg::Param(n) => req.path.set_field(n, segs[i]),
                        Seg::Wild     => unreachable!(),
                    }
                }
                // '*' には残りの全パスを結合
                let rest = if segs.len() > prefix_len {
                    segs[prefix_len..].join("/")
                } else {
                    String::new()
                };
                // 残りが空（/url や /url/）はマッチさせない
                if rest.is_empty() { continue 'outer_wild; }
                req.path.set_field("*", &rest);
                return Some(pat.handler.clone());
            }
        }

        self.not_found.clone()
    }
}
