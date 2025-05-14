# kurosabi

kurosabiは、Rustの安全性と並列性を活かした、超軽量・高速・シンプルなWebフレームワークです。TypeScript製フレームワーク「hono」にインスパイアされ、Rustで快適なWeb開発体験を提供します。

---

## 特徴
- 超軽量・高速
- シンプルで表現力の高いルーティング
- 非同期ハンドラ対応
- パスパラメータ・ワイルドカード対応
- JSON・ファイルレスポンス
- カスタムコンテキスト対応
- 404やエラー処理が簡単
- 柔軟なサーバー設定

---

## インストール
`Cargo.toml`に以下を追加してください：

```toml
[dependencies]
kurosabi = "0.3.0"
```

---

## はじめかた

### 1. カスタムコンテキストの定義（任意）
```rust
pub struct MyContext {
    pub name: String,
}
impl MyContext {
    pub fn new(name: String) -> Self {
        MyContext { name }
    }
}
```

### 2. サーバー作成とルート追加
```rust
use std::{path::PathBuf, sync::Arc};
use kurosabi::{Kurosabi, kurosabi::Context};

#[tokio::main]
async fn main() {
    let arc_context = Arc::new(MyContext::new("Kurosabi".to_string()));
    let mut kurosabi = Kurosabi::with_context(arc_context);

    // シンプルなテキストレスポンス
    kurosabi.get("/hello", |mut c| async move {
        c.res.text("Hello, World!");
        c
    });

    // パスパラメータ
    kurosabi.get("/hello/:name", |mut c| async move {
        let name = c.req.path.get_field("name").unwrap_or("World".to_string());
        c.res.text(&format!("Hello, {}!", name));
        c
    });

    // ワイルドカード
    kurosabi.get("/wild/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("unknown".to_string());
        c.res.text(&format!("Wildcard: {}", path));
        c
    });

    // JSONレスポンス
    kurosabi.get("/json", |mut c| async move {
        let json_data = r#"{"name": "Kurosabi", "version": "0.1"}"#;
        c.res.json(json_data);
        c
    });

    // ファイルレスポンス
    kurosabi.get("/file", |mut c| async move {
        let _ = c.res.file(&c.req, PathBuf::from("README.md"), true).await.unwrap();
        c
    });

    // フォーム（GET/POST）
    kurosabi.get("/submit", |mut c| async move {
        c.res.html(r#"
        <form action=\"/submit\" method=\"post\">
            <input type=\"text\" name=\"data\" placeholder=\"データを入力してください\" />
            <button type=\"submit\">送信</button>
        </form>
        "#);
        c
    });
    kurosabi.post("/submit", |mut c| async move {
        let body = match c.req.body_form().await {
            Ok(data) => data,
            Err(_) => {
                c.res.set_status(400);
                return c;
            }
        };
        c.res.html(&format!("受信: {:?}", body));
        c
    });

    // 404ハンドラ
    kurosabi.not_found_handler(|mut c| async move {
        let html = format!(
            "<h1>404 Not Found</h1>\n<p>ページが見つかりません。</p>\n<p>debug: {}</p>",
            c.req.header.get_user_agent().unwrap_or("unknown")
        );
        c.res.html(&html);
        c.res.set_status(404);
        c
    });

    // サーバー設定
    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8080)
        .thread(8)
        .thread_name("kurosabi-worker".to_string())
        .queue_size(128)
        .build();

    server.run().await;
}
```

---

## 応用機能

### カスタムハンドラによるJSON API
```rust
use kurosabi::api::GETJsonAPI;
use serde::Serialize;

#[derive(Clone)]
pub struct MyAPI;
#[derive(Serialize)]
pub struct ResJsonSchemaVersion {
    pub name: String,
    pub version: String,
}
#[derive(Serialize)]
#[serde(untagged)]
pub enum ResJsonSchema {
    Version(ResJsonSchemaVersion),
    Error(String),
}

#[async_trait::async_trait]
impl GETJsonAPI<Context<Arc<MyContext>>, ResJsonSchema> for MyAPI {
    fn new() -> Self { MyAPI }
    async fn handler(self, c: &mut Context<Arc<MyContext>>) -> ResJsonSchema {
        let name = c.req.path.get_query("name").unwrap_or("Kurosabi".to_string());
        let version = c.req.path.get_query("version").unwrap_or("0.1".to_string());
        ResJsonSchema::Version(ResJsonSchemaVersion { name, version })
    }
}

// APIルートの登録
kurosabi.get_json_api("/jsonapi", MyAPI::new());
```

---

## ライセンス
MIT
