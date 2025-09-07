use crate::kurosabi::Context;



#[derive(Clone)]
pub struct DefaultContext{
}

impl DefaultContext {
    pub fn new() -> DefaultContext {
        DefaultContext {
        }
    }
}

impl ContextMiddleware<Context<DefaultContext>> for DefaultContext {}

#[async_trait::async_trait]
pub trait ContextMiddleware<C> 
where
    C: Send + Sync + 'static,
{
    async fn before_handle(ctx: C) -> C
    {
        ctx
    }
    async fn after_handle(ctx: C) -> C
    {
        ctx
    }
}