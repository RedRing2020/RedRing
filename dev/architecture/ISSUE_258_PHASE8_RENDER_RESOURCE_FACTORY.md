# Issue #258 Phase8: Render Resource 初期化の共通化

- 対象Issue: #258 (Phase8準備)
- 作成日: 2026-02-26
- スコープ: Uniform/BindGroup 初期化パターンの共通化

## 1. 背景

`mesh.rs` / `line.rs` / `toolpath.rs` / `nurbs_eval.rs` に、
Uniformバッファ生成、BindGroupLayout作成、BindGroup生成の定型処理が重複している。

## 2. 目的

- 初期化処理の責務を共通ヘルパに集約
- パラメータ差分のみを各リソース実装に残し、見通しを改善
- 挙動不変での構造改善に限定

## 3. 実施方針

- `view/render` 内に共通初期化ヘルパを追加（例: `uniform_factory`）
- 共通化対象:
  - Uniform buffer作成
  - bind group layout作成
  - bind group作成
- 適用対象:
  - `MeshResources`
  - `LineResources`
  - `ToolPathResources`
  - NURBS評価系リソース

## 4. 対象ファイル（予定）

- `view/render/src/mesh.rs`
- `view/render/src/line.rs`
- `view/render/src/toolpath.rs`
- `view/render/src/nurbs_eval.rs`
- （必要に応じて）`view/render/src/lib.rs`

## 5. DoD

- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- 重複していた初期化ブロックが共通化されている
- 既存描画の挙動に変更がない

## 6. リスクと対策

- リスク: 共通化により特定リソースの細かな設定差分が埋もれる
- 対策: ヘルパは最小責務に限定し、差分設定は呼び出し側で明示

## 7. Phase8 実施前準備（2026-02-26）

- 現在地:
  - Phase7（`render_2d`/`render_3d` 重複削減）は PR #267 でマージ済み
  - 共通化の次段として、Uniform/BindGroup 初期化の重複削減に着手可能
- 事前確認済み対象:
  - `view/render/src/mesh.rs`
  - `view/render/src/line.rs`
  - `view/render/src/toolpath.rs`
  - `view/render/src/nurbs_eval.rs`
- 着手手順（推奨）:
  1. Uniform/BindGroup 初期化の最小共通ヘルパを追加
  2. `mesh` → `line` → `toolpath` の順で段階適用
  3. 最後に `nurbs_eval` へ適用し、差分設定の保持を確認
  4. 各段階で `cargo build -p redring` を実行
- 非機能要件:
  - 公開API名は維持
  - 描画挙動は変更しない
  - ログ方針は既存の抑制方針を維持

## 8. 進捗更新（Phase8 2/4 着手準備）

- 1/4 完了:
  - PR #268 マージ済み（`mesh.rs` の Uniform初期化を共通化）
  - `uniform_factory` モジュール導入済み
- 2/4 着手対象:
  - `view/render/src/line.rs`
- 2/4 の実施方針:
  - `line.rs` の Uniform/BindGroup 初期化を `uniform_factory` へ移行
  - 公開APIと描画挙動は維持
  - 実施後に `cargo build -p redring` で検証

## 9. 進捗更新（Phase8 3/4 着手準備）

- 2/4 完了:
  - PR #269 マージ済み（`line.rs` の Uniform初期化を共通化）
- 3/4 着手対象:
  - `view/render/src/toolpath.rs`
- 3/4 の実施方針:
  - `toolpath.rs` の Uniform/BindGroup 初期化を `uniform_factory` へ移行
  - 公開APIと描画挙動は維持
  - 実施後に `cargo build -p redring` で検証
