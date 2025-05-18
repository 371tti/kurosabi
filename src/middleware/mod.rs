
#[async_trait::async_trait]
pub trait KurosabiMid<C> {
    async fn mid_before(&mut self, c: &mut C) -> Result<(), String>;
    async fn mid_after(&mut self, c: &mut C) -> Result<(), String>;
}