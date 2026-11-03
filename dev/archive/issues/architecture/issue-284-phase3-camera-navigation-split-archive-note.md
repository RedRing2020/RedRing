# Issue #284 Phase3: Cameraナビゲーション分離

- 対象Issue: #284
- 作成日: 2026-02-26
- スコープ: 責務分割のみ（挙動変更なし）

## 背景

`camera.rs` には Arcball 回転・パン・ズームといったナビゲーション操作ロジックが集約されている。
Phase2で投影責務を分離したため、次段として操作系責務を独立モジュール化し、`Camera` の状態管理責務を明確化する。

## 目的

- 回転・Arcball・パン・ズームの実装を `camera_navigation` へ分離
- `Camera` の public API（メソッド名・引数・挙動）を維持

## 実施方針（Phase3）

- 新規モジュール `viewmodel/graphics/src/camera_navigation.rs` を追加
- 以下のロジックを移設
  - `project_on_sphere`
  - `compute_rotation_from_sphere_points`
  - `rotate`
  - `rotate_arcball`
  - `rotate_arcball_from_delta`
  - `pan`
  - `zoom`
  - `zoom_wheel`
- `camera.rs` 側は同名メソッドを維持しつつ内部委譲に変更
- ログ出力と定数値は現行を維持

## 非対象

- fit/reset/preset 系メソッドの分離
- 操作感度パラメータの調整
- テスト仕様の変更

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 挙動変更なしでビルド・Lint・fmt が通る
- `camera.rs` のナビゲーション実装本体が縮小されている
- 既存呼び出し側は変更不要
