# Issue #258 Phase5-2: `debug_scene` 用途種別分離設計

- 日付: 2026-02-26
- 対象Issue: #258 (Phase5-2準備)
- スコープ: `view/app/src/app_state/debug_scene.rs`

## 背景

`debug_scene.rs` は octree/toolpath/svg/nurbs まで複数用途を単一ファイルで扱っており、
責務境界が読み取りづらい状態になっている。

## 目的

- 挙動を変えずに用途ごとにコードを分離する
- `AppState` 側から「どのデバッグ用途か」が追いやすい構造にする
- 将来の追加（debug形状・可視化）時の競合を減らす

## 方針

### 1) サブモジュール分割

`debug_scene` を以下の用途別モジュールに分割する。

- `debug_scene/camera_fit.rs`
  - `CameraFit` 構築・適用の共通処理
- `debug_scene/octree_debug.rs`
  - octree 可視化の build/apply/load
- `debug_scene/toolpath_debug.rs`
  - CAM snapshot/toolpath 関連
- `debug_scene/shape_debug.rs`
  - SVG読み込みの line/circle/triangle/arc 系
- `debug_scene/nurbs_debug.rs`
  - NURBS曲線/曲面のデバッグ表示

### 2) 互換性維持

- 既存の `AppState` 公開メソッド名（`load_debug_*`）は維持
- 呼び出し側変更は最小限（`mod` と `impl AppState` の分割のみ）

### 3) 併せて行う最小整理

- 同型処理（SVGロード→stage設定→uniform更新）はヘルパ化候補を定義
- ただし実際の共通化は小PRに分け、挙動変更を避ける

## 受け入れ条件

- `cargo fmt` 成功
- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- 既存ショートカットのデバッグ表示挙動が変わらない

## 非対象

- 新規デバッグ機能追加
- 可視化アルゴリズム変更
- 命名変更（Phase6で実施）

## Phase番号整合

- **Phase5-1**: `mouse_input` / `mouse_actions` 責務分離方針（既存ドキュメント）
- **Phase5-2**: 本ドキュメント（`debug_scene` 用途分離）
- **Phase6**: 命名整理（リネーム専用）
