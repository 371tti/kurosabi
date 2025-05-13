use serde::{Deserialize, Serialize};

pub trait POSTJsonAPI<C, Rqs, Rss>: Clone
where
    Rqs: for<'a> Deserialize<'a>,
    Rss: Serialize,
{
    fn new() -> Self;
    fn handler(
        self,
        c: &mut C,
        req_json: &Rqs,
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