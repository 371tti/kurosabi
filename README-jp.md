<div align="center">
<h1 style="font-size: 50px">🔥kurosabi🔥</h1>
</div>

kurosabiは、Rustの安全性と並列性を活かした、超軽量・高速・シンプルなWebバックエンドルーターです。

パフォーマンスと軽量さ、書きやすさ を大事にします

## ToDo
- 初期実装
  - [x] http_serverの実装
  - [x] ルーターの実装
  - [x] 基本的な構文の実装
- 機能追加 1
  - [x] keep_alive の実装
  - [x] サーバー設定の追加
  - [x] レスポンス関連の機能追加
- 最適化 1
  - [x] keep_aliveを修正
  - [x] htmlのフォーマットマクロを追加
  - [x] TCPストリームを直接操作できるように改良
- 破壊的変更 1
  - [x] 構文をより扱いやすくするため Contextにすべて集約
  - [x] http_serverをよりスールプットの高いように改良
- 最適化 2
  - [ ] linuxでTCP操作でport関連の改良
  - [ ] エラーハンドリングをもっと楽に
  - [ ] ミドルウェアへの対応
  - [ ] セキュリティの強化

## 特徴
- 超軽量・高速
- シンプルで表現力の高いルーティング
- 非同期ハンドラ対応
- パスパラメータ・ワイルドカード対応
- JSON・ファイルレスポンス
- カスタムコンテキスト対応
- 404やエラー処理が簡単
- 柔軟なサーバー設定


## インストール
`Cargo.toml`に以下を追加してください：

```toml
[dependencies]
kurosabi = "0.3" #最新のものを
```

## 試す
以下のコマンドでexamplesのデモを見れます。
```
cargo run --example start
```

## はじめかた

### 1. インポート
```rust
use kurosabi::{Kurosabi, kurosabi::Context};
```

### 2. サーバー作成とルート追加と実行
```rust
fn main() {
    // Kurosabiのインスタンスを作成します
    let mut kurosabi = Kurosabi::new();

    // ルートハンドラはこのように定義できます。
    kurosabi.get("/",  |mut c| async move {
        c.res.text("Hello, Kurosabi!");
        c
    });

    // method GETで"/field/:field/:value"にアクセスしたときのハンドラを定義します
    // このハンドラは、URLパスの:fieldと:value部分を取得し、"Field: {field}, Value: {value}"というテキストをレスポンスとして返します
    kurosabi.get("/field/:field/:value", |mut c| async move {
        let field = c.req.path.get_field("field").unwrap_or("unknown".to_string());
        let value = c.req.path.get_field("value").unwrap_or("unknown".to_string());
        c.res.text(&format!("Field: {}, Value: {}", field, value));
        c
    });

    // method GETで"/gurd/*"にアクセスしたときのハンドラを定義します
    // このハンドラは、URLパスの*部分を取得し、"Gurd: {path}"というテキストをレスポンスとして返します
    // *はワイルドカードで、任意の文字列を受け取ります
    kurosabi.get("/gurd/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("unknown".to_string());
        c.res.text(&format!("Gurd: {}", path));
        c
    });

    // method POSTで"/submit"にアクセスしたときのハンドラを定義します
    // これはレスポンスデータをそのまま返します
    kurosabi.post("/submit", |mut c| async move {
        let body = match c.req.body_form().await {
            Ok(data) => data,
            Err(e) => {
                println!("Error receiving POST data: {}", e);
                c.res.set_status(400);
                return c;
            }
        };
        c.res.html(&format!("Received: {:?}", body));
        c
    });

    // 404 notfound のときのハンドラを定義します
    kurosabi.not_found_handler(|mut c| async move {
        let html = html_format!(
            "<h1>404 Not Found</h1>
            <p>The page you are looking for does not exist.</p>
            <p>debug: {{data}}</p>",
            data = c.req.header.get_user_agent().unwrap_or("unknown")
        );
        c.res.html(&html);
        c.res.set_status(404);
        c
    });

    // サーバーを設定し組み立てます
    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8082)
        .build();

    // サーバーを実行します
    server.run();
}

```

## 提案
提案があればぜひissueへ  
プルリクもまってます

---

## ライセンス
MIT
