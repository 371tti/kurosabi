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
以下は詳細な利用例です:

```rust
// デフォルトのルーターとコンテキストで初期化されます
let mut kurosabi = Kurosabi::new();
// let mut custom_kurosabi = Kurosabi::with_contex(...);
```

### ルートの定義
`get` や `post` などのメソッドを使用してルートを定義できます。例:

```rust
kurosabi.get("/hello", |mut c| async move {
    c.res.text("Hello, World!");
    c.res.set_cookie("session_id", "value");
    c.res.set_header("X-Custom-Header", "MyValue");
    c.res.set_status(200);
    Ok(c)
});

kurosabi.get("/hello/:name", |mut c| async move {
    let name = c.req.path.get_field("name").unwrap_or("World".to_string());
    c.res.text(&format!("Hello, {}!", name));
    c.res.set_status(200);
    Ok(c)
});
```

### JSONレスポンス
簡単にJSONレスポンスを返すことができます:

```rust
kurosabi.get("/json", |mut c| async move {
    let json_data = r#"{"name": "Kurosabi", "version": "0.1"}"#;
    c.res.json(json_data);
    Ok(c)
});
```

### フォーム処理
HTMLフォームを提供し、フォーム送信を処理します:

```rust
kurosabi.get("/submit", |mut c| async move {
    c.res.html(r#"
    <form action="/submit" method="post">
        <input type="text" name="data" placeholder="データを入力してください" />
        <button type="submit">送信</button>
    </form>
    "#);
    Ok(c)
});

kurosabi.post("/submit", |mut c| async move {
    let body = c.req.body_string().await.unwrap_or_default();
    println!("受信したPOSTデータ: {}", body);
    c.res.html(&format!("受信: {}", body));
    Ok(c)
});
```

### サーバ設定
カスタム設定でサーバを構成します:

```rust
let mut server = kurosabi.server()
    .host([0, 0, 0, 0])
    .port(80)
    .thread(4)
    .thread_name("kurosabi-worker".to_string())
    .queue_size(128)
    .build();

server.run().await;
```

## コントリビューション
貢献は大歓迎です！  
プロジェクトのコーディングスタイルに従い、大きな変更については事前に issue を立てて議論してください。
