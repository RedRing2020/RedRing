# Issue #284 Phase5: Camera遷移責務分離

- 対象Issue: #284
- 作成日: 2026-02-26
- スコープ: 責務分割のみ（挙動変更なし）

## 背景

Phase1〜4 で `camera.rs` から数学・投影・ナビゲーション・プリセット責務を分離した。
残る主要責務はカメラ間遷移（`slerp_to`）であり、ここを独立させることで `camera.rs` は状態定義と薄い委譲層に整理できる。

## 目的

- `slerp_to` ロジックを `camera_transition` モジュールへ分離
- `Camera` の public API と戻り値仕様を維持

## 実施方針（Phase5）

- 新規モジュール `viewmodel/graphics/src/camera_transition.rs` を追加
- `Camera::slerp_to` 本体を移設し、`camera.rs` は内部委譲へ変更
- 補間に使う `lerp_f32` / `lerp_vector3` は既存 `camera_math` を再利用
- エラー型（`Result<Camera, String>`）は現行維持

## 非対象

- テストモジュールの別ファイル化
- `build_view_projection_matrix` の移設
- APIシグネチャ変更

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 挙動変更なしでビルド・Lint・fmt が通る
- `camera.rs` から遷移ロジック本体が分離されている
- 既存呼び出し側の修正が不要
