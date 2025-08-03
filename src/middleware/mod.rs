#[async_trait::async_trait]
pub trait Middleware<C> {
    /// リクエスト受信後処理（例: 認証、ロギング等）
    async fn after_rx(&mut self, ctx: &mut C) -> Result<(), String>;
    /// レスポンス前処理（例: レスポンスヘッダー追加等）
    async fn before_tx(&mut self, ctx: &mut C) -> Result<(), String>;
    /// レスポンス後処理（例: レスポンスヘッダー追加等）
    async fn after_tx(&mut self, ctx: &mut C) -> Result<(), String>;
    /// エラー時処理（例: エラーログ、カスタムエラーレスポンス等）
    async fn on_error(&mut self, ctx: &mut C);
}