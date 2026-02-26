# RELEASE VERSION PLAN (2026 Q1)

## 最終更新日: 2026-02-26

## 1. 現状スナップショット

- `PR #286`（`develop -> main`）は **merged**。
- `PR #287`（Foundation import準拠 + ドキュメント整理）は **merged**（base: `develop`）。
- Open PR（base=`main` / base=`develop`）は現時点で **0件**。
- リポジトリの Git tag は現時点で **未作成**。
- `origin/main` と `origin/develop` は乖離あり（`develop` 側に未統合コミットが存在）。

## 2. 版数ポリシー（運用初版）

### 2.1 リポジトリ全体（アプリ/統合）

- 初回リリースタグを `v0.1.0` とする（タグ未運用のため）。
- 以降は SemVer 準拠で運用：
  - 破壊的変更: `MINOR` を上げる（0.x運用中）
  - 後方互換の機能追加: `PATCH` を上げる
  - 不具合修正のみ: `PATCH` を上げる

### 2.2 各 crate バージョン

- 現在 `0.1.0` のクレートは、次リリース時に実差分で更新。
- 変更なしクレートは据え置き（全クレート一律 bump はしない）。

## 3. リリース前ゲート（必須）

1. `cargo build`（workspace）
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo fmt --all -- --check`
5. `./scripts/check_architecture_dependencies_simple.ps1`
6. `mdbook build`

## 4. リリース運用フロー（運用初版）

1. `develop` のリリース対象期間を確定（Issue/PRを凍結）
2. `develop -> main` 統合PRを作成
3. 上記ゲート通過と最終レビュー
4. `main` マージ後にタグ作成（例: `v0.1.0`）
5. GitHub Release（変更概要、互換性、既知制約）公開
6. 次サイクル用の `release-next` マイルストーン作成

## 5. 今回の不足と次アクション

### 未実施

- 公式なタグ戦略の合意（初回タグ名・バンプ規則）
- CHANGELOG 運用方式（Keep a Changelog 形式など）の確定
- リリースノートテンプレートの固定化

### 直近TODO（優先順）

- [x] `VERSIONING_POLICY.md` をルートに作成（SemVer + 0.x例外）
- [x] `CHANGELOG.md` を追加（Unreleased セクション開始）
- [ ] リリースノートのテンプレートを `dev/` に追加
- [ ] 次回 `develop -> main` 統合時に初回タグを運用開始

## 6. 補足

この文書は「初回リリース運用のベースライン（運用初版）」です。
運用実績に応じて `VERSIONING_POLICY.md` と整合を取りながら更新してください。
