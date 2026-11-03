# Issue #284 Phase4: Cameraフィット/リセット責務分離

- 対象Issue: #284
- 作成日: 2026-02-26
- スコープ: 責務分割のみ（挙動変更なし）

## 背景

`camera.rs` には表示操作（navigation）に加え、メッシュ適応・視点リセット・緊急脱出などのプリセット制御が同居している。
Phase3までで navigation と projection を分離済みのため、残る preset 系責務を独立させて `Camera` 本体をさらに軽量化する。

## 目的

- fit/reset/preset ロジックを `camera_presets` モジュールへ分離
- `Camera` の public API と既存挙動を維持

## 実施方針（Phase4）

- 新規モジュール `viewmodel/graphics/src/camera_presets.rs` を追加
- 以下を移設
  - `fit_to_mesh`
  - `fit_to_small_mesh`
  - `reset`
  - `reset_to_standard_cad_view`
  - `reset_to_front_view`
  - `reset_to_safe_view`
  - `ensure_minimum_distance`
  - `emergency_camera_escape`
  - `log_state`
- `camera.rs` は上記メソッド本体を内部委譲に変更
- ログ文言・定数値・計算式は現行を維持

## 非対象

- `slerp_to` の移設
- 感度パラメータ・投影パラメータの調整
- テスト仕様の変更

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 挙動変更なしでビルド・Lint・fmt が通る
- `camera.rs` の preset 実装本体が縮小される
- 既存呼び出し側の修正が不要
