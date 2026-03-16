# Issue #320 実施準備チェックリスト

対象Issue: [#320 geo_commons廃止: analysis/geo_algorithms への分解移管](https://github.com/RedRing2020/RedRing/issues/320)

## 1. 目的

- `geo_commons` を廃止して中間層を削減する
- `analysis` を「形状を含まない純粋数値計算」に限定する
- 形状を含む計算（面積/体積/距離/近似）を `geo_*` 側へ移管する

## 2. 現状調査（2026-03-16）

依存:

- `model/geo_foundation/Cargo.toml` のみが `geo_commons` に依存

参照元:

- 実コード参照は `geo_foundation` 経由が中心
- 主な参照箇所:
  - `model/geo_foundation/src/commons/mod.rs`
  - `model/geo_foundation/src/commons/ellipse_calculation_traits.rs`
  - `model/geo_foundation/src/geometry/core/linesegment_traits.rs`

公開関数（24件）:

- `approximations/ellipse.rs` : 8件
- `approximations/curves.rs` : 3件
- `metrics/area_volume.rs` : 8件
- `metrics/distance.rs` : 5件

## 3. 移管方針（更新）

`analysis` の責務（形状を含まない）:

- 線形代数
- 数値積分
- 方程式ソルバー
- 汎用数学ユーティリティ

`geo_*` 側へ移管（形状を含む計算）:

- `metrics::area_volume::*`
- `approximations::ellipse::*`
- `approximations::curves::*`
- `metrics::distance::*`
  - `ellipse_2d_distance_to_point`
  - `ellipse_3d_distance_to_point`
  - `sphere_to_infinite_line_distance`
  - `sphere_to_ray_distance`
  - `sphere_to_line_segment_distance`
  - `line_segment_to_aabb_distance`

## 4. 段階タスク（推奨）

### Phase A: API棚卸し確定（小PR）

- [x] `geo_commons` 公開関数一覧と依存参照を確定
- [x] `analysis` / `geo_algorithms` への分類方針を確定

### Phase B: analysis移管（中PR）

- [ ] `metrics::area_volume::*` を `geo_algorithms` へ移管
- [ ] `approximations::*` を `geo_algorithms` へ移管
- [ ] `geo_foundation` 側ブリッジを `geo_algorithms` 基準へ切替
- [ ] `cargo check -p geo_algorithms -p geo_foundation`
- [ ] `cargo test -p geo_algorithms -p geo_foundation`

### Phase C: analysis責務の整理（小PR）

- [ ] 形状を含む関数が `analysis` に混入していないことを確認
- [ ] 必要なら `geo_algorithms` で数値基盤（積分/solver）を利用する薄いヘルパーを整理
- [ ] `cargo check -p analysis -p geo_algorithms`

### Phase D: geo_commons撤去（小PR）

- [ ] `geo_foundation` の `geo_commons` 依存を削除
- [ ] `model/geo_commons` クレートを削除
- [ ] ワークスペース参照と依存ルールを更新
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

## 5. リスクと対策

- リスク: `geo_foundation` の再エクスポート互換が壊れる
  - 対策: Phase Bで呼び出し側importを先に置換し、最後に `geo_commons` を削除

- リスク: `analysis` にドメイン依存コードが混入する
  - 対策: `analysis` には形状意味を持つ関数を置かない

- リスク: `geo_algorithms` へ移管後に依存境界違反
  - 対策: 移管PRごとに依存チェックスクリプトを実行する

## 6. 完了条件（#320）

- [ ] ワークスペースから `geo_commons` 依存が消える
- [ ] `analysis` は形状を含まない純粋数値計算のみを保持
- [ ] 面積/体積/近似/距離など形状計算は `geo_*` 側（`geo_algorithms`中心）に集約
- [ ] ワークスペース全体の `fmt/check/clippy/test` と依存チェックが通る
