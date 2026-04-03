# Issue #284 Phase2: Camera投影ロジック分離

- 対象Issue: #284
- 作成日: 2026-02-26
- スコープ: 責務分割のみ（挙動変更なし）

## 背景

`viewmodel/graphics/src/camera.rs` には、カメラ状態と操作に加えて投影行列生成ロジックが同居している。
Phase1で数学ヘルパーを分離したため、次段として投影責務を独立モジュール化し、`Camera` 本体の責務をさらに明確化する。

## 目的

- 投影方式ごとの行列生成責務を `camera_projection` へ分離
- `Camera` の公開APIを維持したまま内部構造を整理

## 実施方針（Phase2）

- 新規モジュール `viewmodel/graphics/src/camera_projection.rs` を追加
- `Camera::projection_matrix` のロジックを移設し、`camera.rs` 側は委譲のみとする
- `orthographic_rh_01` の内部実装も新モジュールへ移設
- 既存ログ文言・パラメータ計算・返却行列形式は維持
- `viewmodel/graphics/src/lib.rs` に内部モジュール宣言を追加

## 非対象

- ProjectionMode enum の仕様変更
- near/far/zoom の挙動調整
- Arcball / navigation / preset 系メソッドの分離

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 挙動変更なしでビルド・Lint・fmt が通る
- `camera.rs` に投影ロジック本体が残っていない
- 外部呼び出し側の修正が不要
