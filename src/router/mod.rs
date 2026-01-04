use crate::connection::Connection;

pub type AsyncHandler<C, R, W> = 
    Box<dyn Fn(Connection<C, R, W>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Connection<C, R, W>> + Send>> + Send + 'static>;

pub struct KurosabiRouterBuilder<C> {
    context: C

}



pub struct KurosabiRouter<C> {
    context: C,
}

