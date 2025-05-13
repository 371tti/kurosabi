use serde::{Deserialize, Serialize};
pub trait POSTJsonAPI<C, Rqs, Rss>: Clone
where
    Rqs: crate::api::Rqs,
    Rss: Serialize,
{
    fn new() -> Self;
    fn handler(
        self,
        c: &mut C,
        req_json: Result<Rqs, serde_json::Error>,
    ) -> Rss;
    fn osa() -> Option<String> {
        None
    }
}

pub trait GETJsonAPI<C, Rss>: Clone
where
    Rss: Serialize,
{
    fn new() -> Self;
    fn handler(
        self,
        c: &mut C,
    ) -> Rss;
    fn osa() -> Option<String> {
        None
    }
}

pub trait Rqs: for<'a> Deserialize<'a> {
    fn def_err_req() -> Self;
}