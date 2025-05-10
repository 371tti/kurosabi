//! 高速 Radix ルータ (AHash + SmallVec + 2-phase param)

use std::{future::Future, pin::Pin, sync::Arc};

use ahash::AHashMap as Map;
use smallvec::SmallVec;

use crate::{
    error::HttpError, kurosabi::Context, request::Req, utils::method::Method,
};

/* ---------- Router trait -------------------------------------------- */
pub trait GenRouter<F>: Send + Sync
where
    F: Send + Sync + 'static,
{
    fn regist(&mut self, method: Method, pattern: &str, excuter: F);
    fn build(&mut self);
    fn route(&self, req: &mut Req) -> Option<F>;
}

/* ---------- Handler alias ------------------------------------------- */
pub type BoxedHandler<C> = Box<
    dyn Fn(Context<C>)
            -> Pin<
                Box<dyn Future<Output = Result<Context<C>, (Context<C>, HttpError)>> + Send>,
            > + Send
        + Sync,
>;

/* ---------- Radix tree --------------------------------------------- */
#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Static,
    Param,
    Wildcard,
}

type StaticKids<C> = SmallVec<[Box<Node<C>>; 4]>; // up to 4 without heap

struct Node<C> {
    #[allow(dead_code)]
    kind: Kind,
    label: Box<str>,                // compressed label
    param_name: Option<Box<str>>,   // :param name
    fixed: Map<u8, StaticKids<C>>,  // first-byte → siblings
    param_child: Option<Box<Node<C>>>,
    wild_child:  Option<Box<Node<C>>>,
    handler: Option<Arc<BoxedHandler<C>>>,
}

impl<C> Node<C> {
    fn new(kind: Kind, label: impl Into<Box<str>>) -> Self {
        Self {
            kind,
            label: label.into(),
            param_name: None,
            fixed: Map::default(),
            param_child: None,
            wild_child: None,
            handler: None,
        }
    }
}

/* ---------- Router struct ------------------------------------------ */
pub struct DefaultRouter<C> {
    trees: Map<Method, Box<Node<C>>>,
    sealed: bool,
}

impl<C> DefaultRouter<C> {
    pub fn new() -> Self {
        Self {
            trees: Map::default(),
            sealed: false,
        }
    }
}

/* ================= GenRouter impl ================================== */
impl<C: 'static> GenRouter<Arc<BoxedHandler<C>>> for DefaultRouter<C> {
    /* ----- regist --------------------------------------------------- */
    fn regist(&mut self, method: Method, pattern: &str, excuter: Arc<BoxedHandler<C>>) {
        assert!(!self.sealed, "router is sealed");

        let mut path = pattern.trim_start_matches('/');

        let root = self
            .trees
            .entry(method.clone())
            .or_insert_with(|| Box::new(Node::new(Kind::Static, "")));
        let mut node = root.as_mut();

        loop {
            if path.is_empty() {
                if node.handler.is_some() {
                    panic!("duplicate route: {:?} {}", method, pattern);
                }
                node.handler = Some(excuter);
                return;
            }

            match path.as_bytes()[0] {
                b':' => {
                    /* :param ---------------------------------------- */
                    let end = path.find('/').unwrap_or(path.len());
                    let name = &path[1..end];
                    path = path[end..].trim_start_matches('/');

                    let child = node
                        .param_child
                        .get_or_insert_with(|| Box::new(Node::new(Kind::Param, "")));
                    child.param_name = Some(name.into());
                    node = child.as_mut();
                }
                b'*' => {
                    /* * --------------------------------------------- */
                    node = node
                        .wild_child
                        .get_or_insert_with(|| Box::new(Node::new(Kind::Wildcard, "")));
                    path = "";
                }
                _ => {
                    /* Static ---------------------------------------- */
                    let mut i = 0;
                    while i < path.len()
                        && !matches!(path.as_bytes()[i], b':' | b'*' | b'/')
                    {
                        i += 1;
                    }
                    let (seg, rest) = path.split_at(i);
                    path = rest.trim_start_matches('/');

                    let key = seg.as_bytes()[0];
                    let bucket = node.fixed.entry(key).or_default();

                    // 最大共有 prefix 探索
                    let mut idx_lcp = None;
                    for (n, child) in bucket.iter_mut().enumerate() {
                        let lcp = child
                            .label
                            .bytes()
                            .zip(seg.bytes())
                            .take_while(|(a, b)| a == b)
                            .count();
                        if lcp == child.label.len() {
                            idx_lcp = Some((n, lcp));
                            break;
                        } else if lcp > 0 {
                            // 部分一致 → 分割
                            let suffix_existing = child.label[lcp..].to_string();
                            let mut split = Node::new(Kind::Static, suffix_existing);
                            std::mem::swap(&mut split.fixed, &mut child.fixed);
                            split.handler = child.handler.take();

                            let mut truncated = child.label.to_string();
                            truncated.truncate(lcp);
                            child.label = truncated.into_boxed_str();
                            child
                                .fixed
                                .entry(split.label.as_bytes()[0])
                                .or_default()
                                .push(Box::new(split));
                            idx_lcp = Some((n, lcp));
                            break;
                        }
                    }

                    let child = if let Some((n, _)) = idx_lcp {
                        &mut bucket[n]
                    } else {
                        bucket.push(Box::new(Node::new(Kind::Static, seg)));
                        bucket.last_mut().unwrap()
                    };

                    // ★ BUGFIX: 正しく suffix 側へ進む
                    let lcp = child.label.len();
                    if seg.len() > lcp {
                        let suffix = &seg[lcp..];
                        node = child
                            .fixed
                            .get_mut(&suffix.as_bytes()[0])
                            .unwrap()
                            .last_mut()
                            .unwrap();
                    } else {
                        node = child.as_mut();
                    }
                }
            }
        }
    }

    /* ----- build ---------------------------------------------------- */
    fn build(&mut self) {
        if self.sealed {
            return;
        }
        fn dfs<C>(n: &mut Node<C>) {
            for kids in n.fixed.values_mut() {
                for c in kids.iter_mut() {
                    dfs(c);
                }
                kids.shrink_to_fit();
            }
            if let Some(c) = &mut n.param_child {
                dfs(c);
            }
            if let Some(c) = &mut n.wild_child {
                dfs(c);
            }
        }
        for root in self.trees.values_mut() {
            dfs(root);
        }
        self.sealed = true;
    }

    /* ----- route ---------------------------------------------------- */
    #[inline]
    fn route(&self, req: &mut Req) -> Option<Arc<BoxedHandler<C>>> {
        let mut node = self.trees.get(&req.method)?.as_ref();
        let path = req.path.path.clone().trim_start_matches('/').to_string();

        let mut i = 0;
        let mut params: SmallVec<[(&str, (usize, usize)); 4]> = SmallVec::new();

        /* phase-1 : 走査 & パラメータ位置記録 ------------------------ */
        loop {
            if i == path.len() {
                break;
            }
            /* Static ------------------------------------------------ */
            if let Some(bucket) = node.fixed.get(&path.as_bytes()[i]) {
                let mut matched = false;
                for child in bucket {
                    if path[i..].starts_with(&*child.label) {
                        i += child.label.len();
                        if i < path.len() && path.as_bytes()[i] == b'/' {
                            i += 1;
                        }
                        node = child;
                        matched = true;
                        break;
                    }
                }
                if matched { continue; }
            }
            /* Param ------------------------------------------------- */
            if let Some(child) = &node.param_child {
                let start = i;
                while i < path.len() && path.as_bytes()[i] != b'/' { i += 1; }
                if let Some(name) = &child.param_name {
                    params.push((name, (start, i)));
                }
                if i < path.len() && path.as_bytes()[i] == b'/' { i += 1; }
                node = child;
                continue;
            }
            /* Wildcard ---------------------------------------------- */
            if let Some(child) = &node.wild_child {
                params.push(("*", (i, path.len())));
                node = child;
                break;
            }
            return None; // mismatch
        }

        /* phase-2 : 可変借用してセット ------------------------------ */
        for (key, (s, e)) in params {
            req.path.set_field(key, &path[s..e]);
        }
        node.handler.clone()
    }
}
