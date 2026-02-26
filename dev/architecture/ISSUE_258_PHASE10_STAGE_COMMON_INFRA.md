# Issue #258 Phase10: Stage共通基盤の導入（低リスク）

- 対象Issue: #258 (Phase10準備)
- 作成日: 2026-02-26
- スコープ: `view/stage` 内の重複処理を共通基盤へ集約

## 1. 背景

`toolpath_stage` / `octree_stage` / `nurbs_curve_stage` / `nurbs_surface_stage` で、
以下の処理が重複している。

- view/proj から view_proj の生成
- uniform の model 行列初期化
- 深度テクスチャ生成ロジック（toolpath/octree）

## 2. 目的

- 重複を削減し、変更漏れを防止
- 挙動変更なしで `stage` 内実装の可読性を向上

## 3. 実施方針

- `view/stage/src` に共通モジュールを追加（例: `stage_common`）
- 共通化対象:
  - camera uniform 構築ヘルパ
  - depth texture 作成ヘルパ
- 各Stage側は差分ロジック（データ有無判定/描画分岐）に専念

## 4. 対象ファイル（予定）

- `view/stage/src/toolpath_stage.rs`
- `view/stage/src/octree_stage.rs`
- `view/stage/src/nurbs_curve_stage.rs`
- `view/stage/src/nurbs_surface_stage.rs`
- `view/stage/src/lib.rs`
- （新規）`view/stage/src/stage_common.rs`

## 5. DoD

- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- 上記重複ブロックが共通モジュールへ集約
- 描画結果が既存と同等

## 6. リスクと対策

- リスク: 共通化で stage 固有の差分が見えづらくなる
- 対策: ヘルパは最小責務に限定し、stage 固有分岐は呼び出し側に残す
