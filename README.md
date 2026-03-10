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
- トップレベルのスールプット-安定レイテンシ-省メモリ
- 状態型を用いた安全なメソッド仕様

## パフォーマンス
`GET /hello`に対し`hello`を返す基本的な性能が測れるベンチマーク結果です。

- OS: Windows 11
- CPU: Intel Core i9-11900K
- Memory: 47GB DDR4-2800
- ベンチマーク時は、停止可能なプロセスを可能な限り停止した状態で測定
- 各測定は **2回連続で実行し、後に実行した結果を採用**

コードや環境、その他詳細は[ベンチマークリポジトリ](https://github.com/371tti/rust-http-server-bench)へ

| Server   | Requests/sec | Avg (ms) | p50 (ms) | p95 (ms) | p99 (ms) | p99.9 (ms) | Max (ms) |
| -------- | -----------: | -------: | -------: | -------: | -------: | ---------: | -------: |
| actix    |  46,623.4473 |   4.2790 |   4.1203 |   7.3056 |   8.9347 |    11.1651 |  17.6294 |
| axum     |  48,763.3611 |   4.0946 |   3.9687 |   6.4912 |   7.8943 |    10.2854 | 523.0516 |
| hyper    |  53,822.6072 |   3.7096 |   3.5986 |   5.8070 |   6.9984 |     8.9679 |  23.5547 |
| kurosabi |  55,413.6738 |   3.6022 |   3.4850 |   5.7590 |   7.0424 |     9.2435 |  29.0852 |

| Server   |  Idle | Under Load |
| -------- | ----: | ---------: |
| actix    | 6.4MB |      7.8MB |
| axum     | 4.6MB |      8.0MB |
| hyper    | 3.7MB |      5.8MB |
| kurosabi | 2.7MB |      5.2MB |

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
kurosabi = { version = "0.6", features = ["tokio-server"] }
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
