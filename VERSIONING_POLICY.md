# Versioning Policy

## 目的

RedRing のリリース版数を一貫して運用するための基準を定義します。

## 基本方針

- バージョン体系は Semantic Versioning（SemVer）を採用する。
- 0.x 期間は API/仕様が変化しうる前提で、互換性破壊を許容する。
- リリースの判定単位は原則リポジトリ全体（統合）とする。

## 0.x 運用ルール

### タグ

- 初回タグは `v0.1.0` を基準とする。
- タグは `main` への統合後に付与する。

### 版数の上げ方

- 後方互換を維持した機能追加: `PATCH` を上げる（例: `0.1.0 -> 0.1.1`）。
- 互換性に影響する仕様変更・大規模整理: `MINOR` を上げる（例: `0.1.1 -> 0.2.0`）。
- ドキュメント修正のみ: 必要に応じて据え置き、または `PATCH`。

## クレート版数

- 変更があったクレートのみ版数更新する。
- 変更がないクレートの一律 bump は行わない。

## 開発時の必須チェック

**すべての push 前に実行（推奨）:**
```bash
pwsh scripts/check_all_before_push.ps1
```

このスクリプトは以下を実行します：
- `cargo fmt --all -- --check`（フォーマットチェック）
- `cargo clippy --all-targets --all-features --workspace -- -D warnings`
- `cargo test --workspace`

---

## リリース前チェック（リリース管理者向け）

1. `cargo build`
2. `cargo test --workspace`
3. `cargo clippy --all-targets --all-features --workspace -- -D warnings`
4. `cargo fmt --all -- --check`
5. `./scripts/check_architecture_dependencies_simple.ps1`
6. `mdbook build`

## 運用フロー

1. `develop` で対象PRを確定（リリース範囲固定）
2. `develop -> main` 統合PRを作成
3. CIグリーンとレビュー承認を確認
4. `main` マージ後にタグ付与
5. `CHANGELOG.md` の `Unreleased` を確定版に反映
6. GitHub Release を作成

## 改定

この文書は運用実績に基づいて更新する。
更新時は PR で変更理由を明記する。
