# Issue #284 Phase1: Camera数学ヘルパー抽出

- 対象Issue: #284
- 作成日: 2026-02-26
- スコープ: 責務分割のみ（挙動変更なし）

## 背景

`viewmodel/graphics/src/camera.rs` は、カメラ状態・操作ロジック・数学補助関数・テストを1ファイルに保持しており、責務が過密になっている。

Phase1では最小リスクで分割を開始するため、外部APIに影響しない数学補助関数を先行抽出する。

## 目的

- `camera.rs` の責務を段階的に軽量化する
- `Camera` 公開APIと既存挙動を維持したまま、内部構造の見通しを改善する

## 実施方針（Phase1）

- 新規モジュール `viewmodel/graphics/src/camera_math.rs` を追加
- 以下の内部関数を `camera.rs` から移設
  - `quaternion_to_matrix`
  - `matrix_transform_vector`
  - `lerp_vector3`
  - `lerp_f32`
- `camera.rs` は新モジュールを `crate::camera_math` 経由で利用
- `viewmodel/graphics/src/lib.rs` に内部モジュール宣言を追加
- ロジック・シグネチャ・公開APIは変更しない

## 非対象

- `Camera` の public メソッド分割（projection/navigation/presets）
- App層・View層の呼び出し経路変更
- カメラ挙動パラメータの調整

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 挙動変更なしでビルド・Lint・fmt が通る
- `camera.rs` から数学補助関数定義が分離されている
- 既存の `Camera` 利用側に変更が不要
