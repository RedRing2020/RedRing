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

## 7. 事前確認結果（2026-02-26）

- 深度テクスチャ生成の重複
  - `toolpath_stage.rs` に `create_depth_texture` 実装あり
  - `octree_stage.rs` に同等の `create_depth_texture` 実装あり
  - いずれも `Depth32Float` / `RENDER_ATTACHMENT | TEXTURE_BINDING` / `TextureView::default()` の同一パターン
- camera uniform 構築の重複
  - `toolpath_stage.rs` の `update_camera`
  - `octree_stage.rs` の `update_camera`
  - `nurbs_curve_stage.rs` の `update_camera`
  - `nurbs_surface_stage.rs` の `update_camera`
  - いずれも `view_proj` 構築 + 単位 `model` 行列の組み立てを実施

## 8. Phase10着手チェックリスト

- [ ] 共通化対象を `depth texture` と `camera uniform` の2点に限定（スコープ固定）
- [ ] `stage_common` のAPIは最小責務のみ（生成/構築のみ、描画分岐は持たない）
- [ ] `render_stage` トレイト境界に変更を入れない（互換維持）
- [ ] `octree_stage` のデバッグログ方針（現行 `info`）は Phase10では変更しない
- [ ] 検証順序を固定（`cargo build -p redring` → `cargo clippy -p redring -- -D warnings`）

## 9. 推奨実装順序（Phase10本体）

1. `view/stage/src/stage_common.rs` 新規作成
   - 深度テクスチャ生成ヘルパ
   - `view_proj + model(identity)` 構築ヘルパ
2. `view/stage/src/lib.rs` に `stage_common` を公開追加
3. `toolpath_stage.rs` の `create_depth_texture` を共通ヘルパ利用へ置換
4. `octree_stage.rs` の `create_depth_texture` を共通ヘルパ利用へ置換
5. `toolpath/octree/nurbs_curve/nurbs_surface` の camera uniform 構築を共通ヘルパ利用へ置換
6. `cargo build -p redring` / `cargo clippy -p redring -- -D warnings` で確認

## 10. 非対象（Phase10で扱わないもの）

- `render pass` 内の clear 色や描画順序
- `RenderStage` trait のシグネチャ変更
- `octree_stage` のログレベル調整（Phase12で扱う）
- stage 生成経路の一本化（Phase11で扱う）
