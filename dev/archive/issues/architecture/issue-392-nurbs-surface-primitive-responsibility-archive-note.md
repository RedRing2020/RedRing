# Issue #392 実装準備メモ

対象Issue: [#392 [Phase C][#347] NurbsSurface3D × Primitive 混在 collision/intersection の責務整理](https://github.com/RedRing2020/RedRing/issues/392)

関連:
- 親Issue: #347
- 完了Issue: #349（NurbsCurve3D × Primitive）
- Design Freeze: #356

## 1. 方針（段階導入）

NurbsSurface3D は計算負荷と数値探索の難易度が高いため、最小セットから段階導入する。

### Stage 1（最小）
- 対象ペア: Point3D / Plane3D / Ray3D
- collision: `primitive_nurbs_surface.rs` に最小距離判定を実装
- intersection: tolerance ベースの最小 entry point を実装

### Stage 2（拡張）
- 対象ペア: LineSegment3D / InfiniteLine3D / Circle3D
- Stage 1 で抽出した共通ロジックを再利用

### Stage 3（残り）
- 対象ペア: SphericalSolid3D / EllipsoidalSolid3D / CylindricalSolid3D
- 必要に応じて pair_base 化を段階適用

## 2. 現状棚卸（要点）

- `geo_algorithms` 側に NurbsSurface3D 専用の collision/intersection 実装は未存在
- `geo_contracts` には NurbsSurface3D の Core trait定義は存在
- #349 で確立した `primitive_nurbs.rs` 構成（collision + intersection 対称）を流用可能

## 3. 実装対象ファイル（Stage 1）

- 追加:
  - `model/geo_algorithms/src/collision/primitive_nurbs_surface.rs`
  - `model/geo_algorithms/src/intersection/primitive_nurbs_surface.rs`
- 更新:
  - `model/geo_algorithms/src/collision/mod.rs`
  - `model/geo_algorithms/src/intersection/mod.rs`

## 4. 実装順序（Stage 1）

1. `collision/primitive_nurbs_surface.rs` 追加（Point3D / Plane3D / Ray3D）
2. `intersection/primitive_nurbs_surface.rs` 追加（同3ペア）
3. `mod.rs` 2箇所にモジュール追加・再エクスポート
4. テスト追加（最小対称ケース）
5. 検証: `cargo clippy` -> `cargo fmt` -> `cargo test --workspace`

## 5. 完了条件（Stage 1）

- NurbsSurface3D × (Point3D/Plane3D/Ray3D) の collision/intersection が `geo_algorithms` に実装される
- 対称 entry point と最小テストが存在する
- 上記検証が通過する
