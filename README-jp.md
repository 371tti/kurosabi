# 🔥kurosabi🔥

kurosabi は、非常に軽量でシンプルなウェブフレームワークです。Rust の並列性と安全性を活かして設計されています。

## kurosabi とは？
「kurosabi」とは、日本語で「黒さび」を意味します。  
このフレームワークは TypeScript のウェブフレームワーク「hono」に触発されて作られました。  
黒さび「kurosabi」はさびを炎「hono」であぶったらできるわけですよ。。

## 特徴
- 非常に軽量で高性能
- Rust によるメモリ安全性とスレッド安全性
- Tokio を使用した非同期処理のサポート
- 簡単なルーティングとミドルウェア管理
- カスタマイズおよび拡張が容易

## インストール
Cargo.toml に以下の依存関係を追加してください:

```toml
[dependencies]
kurosabi = "0.1"  // 最新のバージョンを使用してください
```

## 使い方
以下は基本的な利用例です:

```rust
use kurosabi::kurosabi::Kurosabi;

#[tokio::main]
async fn main() {
    let mut kurosabi = Kurosabi::new();
    // ルートやミドルウェアの設定を行います
    kurosabi.run().await;
}
```

## コントリビューション
貢献は大歓迎です！  
プロジェクトのコーディングスタイルに従い、大きな変更については事前に issue を立てて議論してください。
