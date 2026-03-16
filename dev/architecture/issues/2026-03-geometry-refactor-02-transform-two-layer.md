## 概要

Transform責務を2層化する。

- 共通変換核: `geo_core`
- 形状ごとのTransform API/実装: `geo_primitives` / `geo_nurbs`

## 背景

- Transform 実装が分散しており、重複と境界の曖昧さがある
- `geo_algorithms` はクロス形状演算に集中させ、形状固有Transform責務は持たせない

## タスク

### Phase A: NURBS先行切替

- [ ] `curve_2d_transform.rs` / `curve_3d_transform.rs` / `surface_3d_transform.rs` の `AnalysisTransform*` と `TransformError` を `geo_core` 参照へ切替
- [ ] `cargo check -p geo_nurbs`
- [ ] `cargo test -p geo_nurbs`

### Phase B: Primitives展開

- [ ] `model/geo_primitives/src/*_transform.rs`（32ファイル）の `AnalysisTransform*` / `AnalysisTransformSupport` / `TransformError` を `geo_core` 基準へ切替
- [ ] 既存テストの import 置換
- [ ] `cargo check -p geo_primitives`
- [ ] `cargo test -p geo_primitives`

### Phase C: 呼び出し側整理

- [ ] `geo_primitives` / `geo_nurbs` の `lib.rs` 再エクスポートを `geo_core` 基準へ更新
- [ ] README / examples の Transform import を更新
- [ ] 旧経路コメントや暫定注記を整理

### Phase D: 全体検証

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

## 受け入れ条件

- [ ] Transform共通核（trait/エラー/共通変換核）が `geo_core` に集約されている
- [ ] 形状固有Transformが `geo_primitives` / `geo_nurbs` から利用できる
- [ ] Transform実装35ファイル（Primitives 32 + NURBS 3）の参照が `geo_core` 基準に統一されている
- [ ] `cargo fmt --all -- --check` が通る
- [ ] `cargo check --workspace` が通る
- [ ] `cargo test --workspace` が通る
- [ ] 依存チェックスクリプトが通る

## 備考

- API変更は破壊的でよい（移行優先）
- `geo_foundation` 全廃自体は #318 のスコープで扱う
