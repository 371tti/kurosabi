<div align="center">
<h1 style="font-size: 50px">🔥kurosabi🔥</h1>
</div>

kurosabiは、Rustの安全性と並列性を活かした、超軽量・高速・シンプルなWebバックエンドルーターです。

パフォーマンスと軽量さ、書きやすさ、シンプルさを大事にします

## ToDo
- Rewrite
  - [x] 基本的な機能の実装
  - [x] server 抽象化の実装
  - [x] レスポンス種の充実
  - [x] パフォーマンスチューニング1
  - [x] streamingの最適化1
  - [ ] WebSocketの実装
  - [ ] middlewareの基盤構築
  - [ ] 翻訳作業1
- しばらく使って改善探す

## 特徴
- 超軽量・高速・小依存
- シンプルで表現力の高いルーティング
- 非同期ハンドラ対応
- JSON・ファイルレスポンス
- カスタムコンテキスト対応
- 404やエラー処理が簡単


## インストール
`Cargo.toml`に以下を追加してください：

```toml
[dependencies]
kurosabi = "0.6"
```

## 試す
以下のコマンドでexamplesのデモを見れます。
```
cargo run --example hello --features="tokio-server"
```

## はじめかた
tokioでの場合

### 1. Cargo.toml
```toml
[dependencies]
kurosabi = { version = "0.7", features = ["tokio-server"] }
```

### 2. サーバー作成とルート追加と実行
```rust
use std::io::Result;

use kurosabi::{http::HttpMethod, server::tokio::KurosabiTokioServerBuilder};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let server = KurosabiTokioServerBuilder::default()
        .bind([0, 0, 0, 0])
        .port(8080)
        .router_and_build(|conn| async move {
            match conn.req.method() {
                HttpMethod::GET => match conn.path_segs().as_ref() {
                    // GET /hello
                    ["hello"] => conn.text_body("Hello, World!"),

                    // GET /hello/:name
                    ["hello", name] => {
                        let body = format!("Hello, {}!", name);
                        conn.text_body(body)
                    },

                    // GET /anything/:anything...
                    ["anything", others @ ..] => {
                        let own: String = others.join("/");
                        conn.text_body(format!("You requested anything/{}!", own))
                    },

                    // GET /
                    [""] => conn.text_body("Welcome to the Kurosabi HTTP Server!"),

                    // その他は404
                    _ => conn.set_status_code(404u16).no_body(),
                },
                // GET以外を405
                _ => conn.set_status_code(405u16).no_body(),
            }
        },
    );
    server.run().await
}
```

## 提案
提案があればぜひissueへ  
プルリクもまってます

---

## ライセンス
MIT
