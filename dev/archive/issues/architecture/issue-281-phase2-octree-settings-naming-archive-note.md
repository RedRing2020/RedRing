# Issue #281 Phase2: Octree設定命名整合

- 対象Issue: #281
- 作成日: 2026-02-26
- スコープ: 命名修正のみ（挙動変更なし）

## 背景

`OctreeDebugVisualizationSettings` は `debug` 接頭辞を持つが、実際の責務は View 層から注入される可視化設定であり、デバッグ専用の一時操作ではない。

## 目的

- 設定型名を責務ベースへ整合し、`debug/sample/dev` 方針との不整合を解消する
- 既存挙動を維持したまま可読性を向上する

## 実施方針

- `OctreeDebugVisualizationSettings` → `OctreeVisualizationSettings`
- 追従対象
  - `viewmodel/converter/src/octree_converter.rs`
  - `viewmodel/converter/src/cam_sim_visualization_converter.rs`
  - `view/app/src/app_state.rs`
- ロジック変更は行わない

## 非対象

- `OctreeVisualizationOptions` / `VoxelVisualizationOptions` の仕様変更
- sample生成ロジック・描画挙動の変更

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 旧型名参照が残っていない
- 挙動不変でビルド・Lint・fmt が通る
