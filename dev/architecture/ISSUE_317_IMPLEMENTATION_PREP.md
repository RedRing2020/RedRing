# Issue #317 実施準備チェックリスト

対象Issue: [#317 geo_core依存逆転解消: geo_core -> geo_foundation を撤去](https://github.com/RedRing2020/RedRing/issues/317)

## 1. 目的

- `geo_core` を最下層基盤として成立させる
- `model/geo_core` の `geo_foundation` 依存を除去する
- 後続の `geo_foundation` 廃止（#318）を可能にする

## 2. 現状調査（geo_core -> geo_foundation 参照）

依存定義:

- `model/geo_core/Cargo.toml`
  - `geo_foundation = { path = "../geo_foundation" }`

主な参照箇所:

- 数値型再エクスポート経由
  - `Scalar`, `Angle`, `TolerantEq`, `TransformError`
- Foundation trait 参照
  - `AnalysisTransform2D`, `AnalysisTransform3D`
  - `geometry::core::vector_traits::*`
  - `commons::Aabb2DTrait`, `commons::Aabb3DTrait`

影響ファイル（初期棚卸し）:

- `model/geo_core/src/point_2d.rs`
- `model/geo_core/src/point_2d_transform.rs`
- `model/geo_core/src/point_3d.rs`
- `model/geo_core/src/point_3d_transform.rs`
- `model/geo_core/src/vector_2d.rs`
- `model/geo_core/src/vector_2d_transform.rs`
- `model/geo_core/src/vector_3d.rs`
- `model/geo_core/src/vector_3d_transform.rs`
- `model/geo_core/src/aabb_2d.rs`
- `model/geo_core/src/aabb_3d.rs`
- `model/geo_core/src/point_2d_tests.rs`
- `model/geo_core/src/vector_2d_tests.rs`
- `model/geo_core/src/vector_3d_tests.rs`
- `model/geo_core/src/lib.rs`（説明文更新）

## 3. 実施方針

### 3.1 切り分け原則

- 先に「型依存」を切る
  - `Scalar` / `Angle` / `TolerantEq` は `analysis` へ直接依存
- 次に「trait依存」を切る
  - Transform系 trait と AABB trait は `geo_core` 内に再定義
- 最後に `Cargo.toml` から `geo_foundation` を削除

### 3.2 後方互換ポリシー

- 破壊的変更を許容
- ただし初回PRでは公開シグネチャ変更を最小化し、コンパイルエラーの波及を抑える

## 4. 段階タスク（推奨）

### Phase A: 型依存の直接化（小PR）

- [x] `use geo_foundation::{Scalar, Angle, TolerantEq}` を `analysis` 直参照へ置換
- [x] 置換後に `cargo check -p geo_core`
- [x] `cargo test -p geo_core`

### Phase B: trait再配置（小PR）

- [x] Transform trait（`AnalysisTransform2D/3D` など）を `geo_core` 内へ移設
- [x] AABB trait を `geo_core` 内へ移設
- [x] `vector_traits` 依存を `geo_core` 側定義へ置換
- [x] `cargo check -p geo_core`
- [x] `cargo test -p geo_core`

### Phase C: 依存定義除去（小PR）

- [ ] `model/geo_core/Cargo.toml` から `geo_foundation` を削除
- [ ] 依存チェックスクリプトの `AllowedDependencies` / `ForbiddenDependencies` を更新
- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

## 5. リスクと対策

- リスク: trait定義移動で `geo_primitives` / `geo_nurbs` 側実装が崩れる
  - 対策: まず `geo_core` 側に同名・同契約 trait を用意し、差分を最小化

- リスク: TransformError の定義差分で挙動が変わる
  - 対策: 既存エラー種別とメッセージ互換を維持

- リスク: 依存ルール更新漏れで CI 失敗
  - 対策: Phase C で依存チェックスクリプトを同時更新

## 6. 初回PRスコープ（推奨）

最初のPRは Phase A のみ:

- `analysis` 直参照化
- テスト通過確認
- 公開API非変更

この分割ならレビューしやすく、失敗時のロールバック範囲も小さい。

## 7. 実施ログ

- 2026-03-16: Phase A を実施済み（`geo_core` の `Scalar/Angle/TolerantEq` を `analysis` 直参照へ置換）
- 検証結果:
  - `cargo check -p geo_core`: pass
  - `cargo test -p geo_core`: pass（110 unit tests + 12 doc tests）
- 2026-03-16: Phase B を実施済み（Transform/AABB/Vector trait と TransformError を `geo_core` 側へ再配置）
- 検証結果:
  - `cargo check -p geo_core`: pass
  - `cargo test -p geo_core`: pass（110 unit tests + 12 doc tests）
