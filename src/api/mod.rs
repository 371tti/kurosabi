use serde::{Deserialize, Serialize};

#[async_trait::async_trait]
pub trait POSTJsonAPI<C, Rqs, Rss>: Clone
where
    Rqs: for<'a> Deserialize<'a>,
    Rss: Serialize,
{
    fn new() -> Self;

    async fn handler(
        self,
        c: &mut C,
        req_json: Result<Rqs, serde_json::Error>,
    ) -> Rss;

    fn osa() -> Option<String> {
        None
    }
}



/// GETJsonAPIは、GETリクエストにおいてJsonAPIを簡単に実装するためのトレイトです。
/// # example
/// ```
/// #[derive(Clone)]
/// pub struct MyAPI;
/// 
/// #[derive(Serialize)]
/// pub struct ResJsonSchemaVersion {
///     pub name: String,
///     pub version: String,
/// }
/// 
/// #[derive(Serialize)]
/// #[serde(untagged)]
/// pub enum ResJsonSchema {
///     Version(ResJsonSchemaVersion),
///     Error(String),
/// }
/// 
/// #[async_trait::async_trait]
/// impl GETJsonAPI<Context<Arc<MyContext>>, ResJsonSchema> for MyAPI {
///     fn new() -> Self {
///         MyAPI
///     }
/// 
///     async fn handler(
///             self,
///             c: &mut Context<Arc<MyContext>>,
///         ) -> ResJsonSchema {
///             let name = c.req.path.get_query("name").unwrap_or("Kurosabi".to_string());
///             let version = c.req.path.get_query("version").unwrap_or("0.1".to_string());
///             c.res.header.set("Connection", "keep-alive");
///             c.res.header.set("Keep-Alive", "timeout=60, max=100");
///             
///             ResJsonSchema::Version(
///                 ResJsonSchemaVersion {
///                     name: name,
///                     version: version,
///                 }
///             )
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait GETJsonAPI<C, Rss>: Clone
where
    Rss: Serialize,
{
    fn new() -> Self;

    async fn handler(
        self,
        c: &mut C,
    ) -> Rss;
    
    fn osa() -> Option<String> {
        None
    }
}