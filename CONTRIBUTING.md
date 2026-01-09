# Contributing Guide

## 開発フロー（推奨）

1. Issue を作る（または既存 Issue を確認）
2. 変更を最小に保って実装
3. `cargo fmt` / `cargo check` / `cargo test` を通す
4. PR を作る

### フォーマット

このリポジトリは `rustfmt.toml` を使っています。

```bash
cargo fmt
```

### ビルド/チェック

```bash
cargo check
```

feature を含めて確認したい場合：

```bash
cargo check --features "tokio-server"
cargo check --features "json"
cargo check --features "tokio-server json"
```

### テスト

```bash
cargo test
```

### examples

`Cargo.toml` の `required-features` により、examples は feature が必要

- hello:

```bash
cargo run --example hello --features "tokio-server"
```

- coffee:

```bash
cargo run --example coffee --features "tokio-server json"
```

## 変更の方針

- 依存追加は慎重に最小に できるだけ依存なしで 小さな依存ならutilsに作るように
- 公開 API の破壊的変更は原則避ける
- パフォーマンスに影響する変更は、理由や簡単な比較結果があると助かります
- ルータ/コネクションまわりは型状態（`Connection<..., S>`）の整合性を崩さないよう注意

## セキュリティ

脆弱性と思われる問題は、公開 Issue ではなくリポジトリ管理者へ非公開で連絡してください
