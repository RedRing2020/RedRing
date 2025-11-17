# 既存Trait → 新分類システム マッピング表

**作成日**: 2025年11月16日
**最終更新**: 2025年11月16日

## 概要

現在のgeo_foundation traitを新しい4+α分類システムにマッピングし、移行計画を策定する。

## 既存Trait分析と新分類マッピング

### 🔵 Core Traits (現在のcore/)

#### Point系 Traits
```rust
// 現在
core/point_traits.rs:
- Point2D<T>              → properties/PositionProperties
- Point3D<T>              → properties/PositionProperties
- Point2DConstructor<T>   → constructor/BasicConstructor
- Point3DConstructor<T>   → constructor/BasicConstructor
```

#### Vector系 Traits
```rust
// 現在
core/vector_traits.rs:
- Vector2D<T>             → properties/PositionProperties
- Vector3D<T>             → properties/PositionProperties
- VectorConstructor<T>    → constructor/BasicConstructor
- VectorOperations<T>     → measure/BasicMeasure (norm, length等)
```

#### Circle系 Traits
```rust
// 現在
core/circle_traits.rs + core/circle_core.rs:
- Circle2DTrait<T>        → properties/ShapeProperties (radius等)
- CircleConstructor<T>    → constructor/BasicConstructor
- CircleOperations<T>     → measure/BasicMeasure (area, circumference)
```

#### Direction系 Traits
```rust
// 現在
core/direction_traits.rs:
- Direction2D<T>          → properties/ShapeProperties
- Direction3D<T>          → properties/ShapeProperties
- DirectionConstructor<T> → constructor/BasicConstructor
```

#### NURBS系 Traits
```rust
// 現在
core/nurbs_traits.rs:
- NurbsCurve<T>           → properties/ShapeProperties
- NurbsSurface<T>         → properties/ShapeProperties
- ParametricGeometry<T>   → properties/ShapeProperties
- WeightedGeometry<T>     → properties/ShapeProperties
- BasisFunction<T>        → measure/GeometricMeasure (評価関数)
```

### 🟠 Extension Traits (現在のextensions/)

#### Transform系 → **Coreに移動**
```rust
// 現在 (extensions/transform.rs) - 既にAnalysis系に統合済み
- AnalysisTransform3D<T>        → core/transform/AnalysisTransform3D
- AnalysisTransform2D<T>        → core/transform/AnalysisTransform2D
- AnalysisTransformVector3D<T>  → core/transform/AnalysisTransformVector3D
- SafeTransform<T>              → core/transform/SafeTransform
// 注：BasicTransformは既にAnalysisTransformに統合済み
```

#### Collision系 → **Extensions維持**
```rust
// 現在 (extensions/collision.rs)
- BasicCollision<T, Other>   → extensions/collision/CollisionDetection
- DistanceCalculation<T>     → extensions/collision/CollisionDetection
```

#### Intersection系 → **Extensions維持**
```rust
// 現在 (extensions/intersection.rs)
- IntersectionCalculation<T> → extensions/collision/IntersectionCalculation
- LineIntersection<T>        → extensions/collision/IntersectionCalculation
```

#### Analysis Conversion → **Extensions維持**
```rust
// 現在 (extensions/analysis_conversion.rs)
- AnalysisConversion<T>      → extensions/analysis/AnalysisConversion
- MatrixConversion<T>        → extensions/analysis/AnalysisConversion
```

## 移行アクションプラン

### Phase 1: 新Core構造の実装 (1-2日)

#### 1.1 Constructor Traits作成
- [ ] `core/constructor/basic_constructor.rs`
- [ ] `core/constructor/from_points.rs`
- [ ] `core/constructor/from_parameters.rs`

#### 1.2 Properties Traits作成
- [ ] `core/properties/position_properties.rs`
- [ ] `core/properties/shape_properties.rs`
- [ ] `core/properties/dimension_properties.rs`

#### 1.3 Transform Traits移行 ⚠️
- [ ] `extensions/transform.rs` → `core/transform/`に移動
- [ ] `core/transform/analysis_transform_3d.rs` (既存AnalysisTransform3D)
- [ ] `core/transform/analysis_transform_2d.rs` (既存AnalysisTransform2D)
- [ ] `core/transform/analysis_transform_vector.rs` (既存AnalysisTransformVector3D)
- [ ] `core/transform/safe_transform.rs` (エラーハンドリング版)

#### 1.4 Measure Traits作成
- [ ] `core/measure/basic_measure.rs`
- [ ] `core/measure/center_of_mass.rs`
- [ ] `core/measure/geometric_measure.rs`

### Phase 2: Extensions構造の整理 (1日)

#### 2.1 Collision系の統合
- [ ] `extensions/collision/collision_detection.rs`
- [ ] `extensions/collision/intersection_calculation.rs`
- [ ] `extensions/collision/spatial_query.rs`

#### 2.2 Boolean Operations追加（将来用）
- [ ] `extensions/boolean/boolean_operations.rs`
- [ ] `extensions/boolean/csg_operations.rs`

#### 2.3 Analysis系の整理
- [ ] `extensions/analysis/analysis_conversion.rs`
- [ ] `extensions/analysis/external_format.rs`

### Phase 3: geo_primitivesの実装更新 (2-3日)

#### 3.1 Point系の更新
- [ ] Point2D/Point3Dの新trait実装
- [ ] コンストラクタの統合

#### 3.2 Vector系の更新
- [ ] Vector2D/Vector3Dの新trait実装
- [ ] Operations → Measureの移行

#### 3.3 Circle系の更新
- [ ] circle_core.rs + circle_traits.rs統合
- [ ] Properties/Measure分離

#### 3.4 Transform実装の移行 ⚠️
- [ ] 全プリミティブの transform メソッド更新
- [ ] extensions → core参照の変更

### Phase 4: 依存クレート更新 (1日)

#### 4.1 geo_coreの更新
- [ ] import文の更新
- [ ] trait bound の更新

#### 4.2 geo_algorithmsの更新
- [ ] 新trait参照への変更

#### 4.3 geo_nurbsの更新
- [ ] NURBS特有trait実装

### Phase 5: 旧構造削除・クリーンアップ (1日)

#### 5.1 重複trait削除
- [ ] 旧Point2DConstructor等の削除
- [ ] circle_core.rsの削除

#### 5.2 Import整理
- [ ] lib.rsのre-export更新
- [ ] モジュール構造の最終確認

## 重要な考慮事項

### ⚠️ 破壊的変更
- **Transform系のcore移行**: 大量のimport文変更が必要 (extensions → core)
- **Constructor統合**: 既存の分離されたコンストラクタの統合
- **注**: Transform系は既にAnalysis系に統合済みで、BasicTransformは存在しない

### 🔄 互換性維持
- 移行期間中は旧traitも並行維持
- Deprecation warningで段階的移行

### 🧪 テスト戦略
- 各Phase完了時にビルド確認
- geo_primitives更新時に全テスト実行

## 想定工数

- **Phase 1-2**: 3-4日 (新構造実装)
- **Phase 3**: 2-3日 (primitives更新)
- **Phase 4**: 1日 (依存更新)
- **Phase 5**: 1日 (クリーンアップ)

**合計**: 7-9日

## 成功指標

- [ ] 全テストパス (471+テスト)
- [ ] ビルド時間維持 (0.20s程度)
- [ ] Clippy警告ゼロ
- [ ] 明確な責務分離の実現
