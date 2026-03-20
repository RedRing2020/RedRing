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

### 直近TODO（優先順）

- [x] `VERSIONING_POLICY.md` をルートに作成（SemVer + 0.x例外）
- [x] `CHANGELOG.md` を追加（Unreleased セクション開始）
- [x] リリースノートのテンプレートを `dev/` に追加（`dev/RELEASE_NOTE_TEMPLATE.md`）
- [ ] 次回 `develop -> main` 統合時に初回タグを運用開始

## 6. 補足

この文書は「初回リリース運用のベースライン（運用初版）」です。
運用実績に応じて `VERSIONING_POLICY.md` と整合を取りながら更新してください。

## 7. v0.1.0 スコープ決定（実装ベース）

### 7.1 v0.1.0 の目標（North Star）

`v0.1.0` は「研究開発向けの **技術プレビュー基盤** を安定配布する」ことを目標にする。

- 目的は「機能網羅」ではなく、**実装済みコア機能を壊さず再現可能に提供**すること
- 対象は CAD/CAM の本番運用ではなく、研究・検証・拡張の土台

### 7.2 v0.1.0 に含める機能（Must）

1. **ビルド/テスト/CI基盤の安定運用**
  - workspace build/test/clippy/fmt
  - architecture dependency check
  - mdbook build

2. **Foundation Pattern 準拠の幾何基盤**
  - `geo_contracts` / `geo_core` / `geo_primitives` の基礎API
  - 直接 import ルールを含む依存整合

3. **NURBS 実装（曲線・曲面 + 適応的テッセレーション）**
  - `geo_nurbs` の既存実装をリリース対象に含める

4. **Octree / Voxel を含む幾何アルゴリズム基盤**
  - `geo_algorithms` の空間分割・除去シミュレーション基盤

5. **Entity / ECS / Topology 基盤（技術プレビュー）**
  - `geo_entity`, `cam_entity` を含むエンティティ基盤の現行機能
  - ECS 的な最小フロー（登録・更新・参照）の安定動作
  - Topology データモデルの基盤機能（高度編集は除外）

6. **CAM基盤（Core/Sim）と可視化までの接続**
  - `cam_core`, `cam_sim` の現行機能
  - view/viewmodel 側の可視化導線（技術プレビュー範囲）

7. **ドキュメント運用基盤**
  - `manual/` + `mdbook` + `CHANGELOG.md` + `VERSIONING_POLICY.md` + `dev/RELEASE_NOTE_TEMPLATE.md`

### 7.3 v0.1.0 で見送る機能（Should Not）

- STEP/IGES 等の本格CAD I/O
- CAMパス自動生成の完成版
- WebAssembly 対応
- SpaceMouse 等の周辺デバイス最適化
- 商用利用前提の互換性保証/長期サポート

### 7.4 リリース受け入れ条件（Acceptance Criteria）

- [ ] `main` で以下がグリーン
  - [ ] `cargo build`
  - [ ] `cargo test --workspace`
  - [ ] `cargo clippy --workspace --all-targets -- -D warnings`
  - [ ] `cargo fmt --all -- --check`
  - [ ] `./scripts/check_architecture_dependencies_simple.ps1`
  - [ ] `mdbook build`
- [ ] `CHANGELOG.md` の `Unreleased` を `0.1.0` として確定
- [ ] `dev/RELEASE_NOTE_TEMPLATE.md` を使って `v0.1.0` ノートを作成
- [ ] Entity / ECS / Topology の最小統合シナリオ（作成→更新→表示）を確認
- [ ] `main` に `v0.1.0` タグを付与

### 7.5 直近の意思決定ポイント

- `v0.1.0` のリリース可否判断は「未実装機能の多寡」ではなく、**Must項目の安定性**で決める。
- 次バージョン（`0.1.1` / `0.2.0`）への繰越は、見送り機能を Issue/Milestone で明示する。

## 8. 承認時に合意すべき論点（恒久記録）

PRコメントのみで残すと参照されなくなるため、`v0.1.0` 判定に必要な合意事項を本文に固定する。

### 8.1 Experimental 扱いの範囲

- `Entity / ECS / Topology` は **技術プレビュー（Experimental）** として公開する。
- 互換性保証レベルは「次のマイナーまでで変更される可能性あり」を明記する。
- 高度編集機能・最適化・完全互換APIは `v0.1.0` の保証対象外とする。

### 8.2 v0.1.0 の保証レベル

- 保証対象は「ビルド可能」「主要フローが再現可能」「既定CIが安定してグリーン」の3点。
- 本番運用品質（商用SLA、長期互換、完全性能保証）はスコープ外。
- 破壊的変更が入り得る領域はリリースノートに明示する。

### 8.3 マージ判定ゲート（最終）

- [ ] 7.4 の Acceptance Criteria を満たす
- [ ] Experimental 範囲をリリースノートに明記
- [ ] `CHANGELOG.md` と `VERSIONING_POLICY.md` の記述に矛盾がない
- [ ] 次サイクルに送る項目を Issue/Milestone に登録済み

