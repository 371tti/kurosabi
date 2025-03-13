use crate::utils::header::Method;

pub trait GenRouter {
    fn regist(&mut self, method: Method, pattern: &str, index: usize) -> ();
}