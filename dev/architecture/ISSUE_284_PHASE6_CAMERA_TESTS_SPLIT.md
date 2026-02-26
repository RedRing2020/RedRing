# Issue #284 Phase6: Camera単体テスト分離

- 対象Issue: #284
- 作成日: 2026-02-26
- スコープ: テスト配置整理のみ（挙動変更なし）

## 背景

`viewmodel/graphics/src/camera.rs` は、実装本体の責務分離（Phase1〜5）を完了した一方で、`#[cfg(test)] mod tests` が同ファイル末尾に残っている。
可読性・保守性向上のため、単体テストは同一モジュール配下で別ファイル化する。

## 目的

- テストを別ファイルへ移し、`camera.rs` の実装可読性を向上
- private API テスト可能性を維持したまま構成整理

## 実施方針（Phase6）

- 新規ファイル `viewmodel/graphics/src/camera_tests.rs` を追加
- 既存 `#[cfg(test)] mod tests { ... }` の内容を移設
- `camera.rs` は次の宣言へ変更
  - `#[cfg(test)]`
  - `#[path = "camera_tests.rs"]`
  - `mod tests;`
- テストロジック・アサーション・テスト名は変更しない

## 非対象

- 統合テスト（`tests/`）への移行
- テストケースの増減や期待値変更
- 公開API/内部実装の変更

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`
- `cargo test --workspace`

## DoD

- 実装とテストが別ファイルに整理されている
- 既存テストが通過し、挙動変更がない
- 追加警告なしでフォーマット・Lint・ビルドが通る
