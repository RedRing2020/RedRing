# Issue #349 実装準備メモ

対象Issue: [#349 [Phase C][#347] NURBS/Primitive 混在 collision/intersection の責務整理](https://github.com/RedRing2020/RedRing/issues/349)

関連:
- Design Freeze: #356
- 親Issue: #347
- 既存準備メモ: ISSUE_350_IMPLEMENTATION_PREP, ISSUE_348_IMPLEMENTATION_PREP

## 1. 目的

- NURBS/Primitive 混在演算の責務境界を #356 fixed-policy に沿って明確化する
- geo_algorithms を混在演算ロジックの正本とする
- 重複経路と未整備領域（特に intersection 側）を段階的に整理する

## 2. 現状整理（2026-03-22）

### 2.1 実装の所在

- 既存実装は主に model/geo_algorithms/src/collision/primitive_nurbs.rs に集約されている
- 同ファイルは NurbsCurveCollider Newtype を定義し、BasicCollision を複数 Primitive へ実装
- 現時点で geo_algorithms 側に nurbs 専用の intersection モジュールは見当たらない

### 2.2 既存で対応済みの主な衝突ペア

NurbsCurveCollider ->
- Point3D
- LineSegment3D
- Ray3D
- InfiniteLine3D
- Plane3D
- Circle3D
- SphericalSolid3D
- EllipsoidalSolid3D
- CylindricalSolid3D

### 2.3 未整備・要判断

- NURBS/Primitive の intersection 責務の実装単位
- NurbsSurface3D 混在ペアの扱い（対象に含めるか）
- 実装粒度: pair-base 抽出を先に行うか、entry point を先に揃えるか

## 3. #356 fixed-policy 準拠の実装方針

- collision/intersection は Foundation パターン対象外として扱う
- 正本ロジックは geo_algorithms 側に置く
- orphan rules 前提のため、必要に応じて Newtype/adapter を継続採用する
- 完了条件は trait 実装の物理移管ではなく、ロジック正本化で評価する

## 4. 実装対象ファイル（初期）

- model/geo_algorithms/src/collision/primitive_nurbs.rs
- model/geo_algorithms/src/collision/mod.rs
- model/geo_algorithms/src/intersection/mod.rs
- model/geo_algorithms/src/intersection/primitive_nurbs.rs（新規候補）
- model/geo_algorithms/src/lib.rs（再エクスポート調整が必要な場合のみ）

## 5. 推奨実装順

### Step A: 責務境界の固定

- [ ] collision と intersection の対象ペア一覧を issue body に明記
- [ ] NurbsCurve と NurbsSurface の対象範囲を確定

### Step B: intersection 側の最小スライス追加

- [ ] primitive_nurbs の intersection entry point を最小セットで追加
- [ ] 既存 collision 実装との対称性ルールをテストで固定

### Step C: 重複整理

- [ ] primitive 系との重複判定ロジックを抽出（必要なら pair_base 化）
- [ ] NURBS専用実装に残す責務を限定

### Step D: 検証

- [ ] cargo clippy -p geo_algorithms --all-targets -- -D warnings
- [ ] cargo fmt --all
- [ ] cargo test -p geo_algorithms
- [ ] scripts/check_architecture_dependencies_simple.ps1

## 6. 完了条件（#349 向け読み替え）

- [ ] NURBS/Primitive 混在演算の責務境界が文書化されている
- [ ] geo_algorithms 側に混在演算ロジック正本が集約されている
- [ ] 追加した対称 entry point の整合テストがある
- [ ] 上記検証が通過する
