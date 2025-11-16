# Foundation Core/Extension分類システム再設計提案

**作成日**: 2025年11月16日  
**最終更新**: 2025年11月16日

## 概要

現在のFoundation分類システムの曖昧さを解決し、より明確で一貫性のある責務分離を実現する。

## 現在の問題点

### 1. 曖昧な分類
- Point2D/Point2DConstructorの分離
- Transform系がextensionにあるが、基本機能として扱うべき
- circle_core.rs と circle_traits.rs の責務重複

### 2. 一貫性の欠如
- 形状によって異なるtrait構成
- core/extensionの境界が不明確

## 新分類システム設計

### Core機能（4つの基本trait群）

#### 1. Constructor Traits - オブジェクト生成
```rust
// 基本コンストラクタ
pub trait BasicConstructor<T: Scalar> {
    fn new(...) -> Self;
    fn origin() -> Self; // 原点系オブジェクト用
}

// 複数点からの構築
pub trait FromPoints<T: Scalar> {
    type Point;
    fn from_points(points: &[Self::Point]) -> Option<Self>;
}

// パラメータからの構築
pub trait FromParameters<T: Scalar> {
    type Parameters;
    fn from_parameters(params: Self::Parameters) -> Option<Self>;
}
```

#### 2. Property Traits - 基本情報取得
```rust
// 座標・位置情報
pub trait PositionProperties<T: Scalar> {
    fn position(&self) -> Self::Point; // 代表点
    fn bounds(&self) -> Self::BBox;    // 境界ボックス
}

// 形状固有プロパティ
pub trait ShapeProperties<T: Scalar> {
    fn normal(&self) -> Option<Self::Vector>;  // 法線（平面等）
    fn radius(&self) -> Option<T>;             // 半径（円等）
    fn dimensions(&self) -> Self::Dimensions;  // 寸法情報
}
```

#### 3. Transform Traits - 座標変換（単一形状）
```rust
// Analysis Matrix/Vector基盤の統一変換（既に統合済み）
pub trait AnalysisTransform3D<T: Scalar> {
    type Matrix4x4;
    type Angle;
    type Output;
    
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError>;
    fn rotate_analysis(&self, center: &Self, axis: &Vector3<T>, angle: Self::Angle) -> Result<Self::Output, TransformError>;
    fn scale_analysis(&self, center: &Self, scale_x: T, scale_y: T, scale_z: T) -> Result<Self::Output, TransformError>;
    fn uniform_scale_analysis(&self, center: &Self, scale_factor: T) -> Result<Self::Output, TransformError>;
}

// エラーハンドリング版（SafeTransform）
pub trait SafeTransform<T: Scalar> {
    // 安全な変換操作（Result返却）
}
```

#### 4. Measure Traits - 計量
```rust
// 基本計量
pub trait BasicMeasure<T: Scalar> {
    fn area(&self) -> Option<T>;      // 面積
    fn volume(&self) -> Option<T>;    // 体積
    fn length(&self) -> Option<T>;    // 長さ
    fn perimeter(&self) -> Option<T>; // 周囲長
}

// 重心・慣性モーメント
pub trait CenterOfMass<T: Scalar> {
    fn centroid(&self) -> Self::Point;
    fn center_of_mass(&self) -> Self::Point;
    fn moment_of_inertia(&self) -> Option<T>;
}
```

### Extension機能（複数形状間の複雑な操作）

#### 1. Collision/Intersection - 複数形状間の関係
```rust
pub trait CollisionDetection<T: Scalar, Other> {
    fn intersects(&self, other: &Other, tolerance: T) -> bool;
    fn distance_to(&self, other: &Other) -> T;
    fn closest_point(&self, other: &Other) -> (Self::Point, Self::Point);
}

pub trait IntersectionCalculation<T: Scalar, Other> {
    type IntersectionResult;
    fn intersection(&self, other: &Other) -> Option<Self::IntersectionResult>;
}
```

#### 2. Boolean Operations - 集合演算
```rust
pub trait BooleanOperations<T: Scalar> {
    fn union(&self, other: &Self) -> Option<Self>;
    fn intersection(&self, other: &Self) -> Option<Self>;
    fn difference(&self, other: &Self) -> Option<Self>;
}
```

#### 3. Analysis Conversion - 外部ライブラリ変換
```rust
pub trait AnalysisConversion<T: Scalar> {
    type AnalysisType;
    fn to_analysis(&self) -> Self::AnalysisType;
    fn from_analysis(data: Self::AnalysisType) -> Option<Self>;
}
```

## 実装構造提案

### ファイル構造
```text
core/
├── constructor/
│   ├── basic_constructor.rs
│   ├── from_points.rs
│   └── from_parameters.rs
├── properties/
│   ├── position_properties.rs
│   ├── shape_properties.rs
│   └── dimension_properties.rs
├── transform/
│   ├── basic_transform.rs
│   ├── analysis_transform.rs
│   └── safe_transform.rs
├── measure/
│   ├── basic_measure.rs
│   ├── center_of_mass.rs
│   └── geometric_measure.rs
└── mod.rs

extensions/
├── collision/
│   ├── collision_detection.rs
│   ├── intersection_calculation.rs
│   └── spatial_query.rs
├── boolean/
│   ├── boolean_operations.rs
│   └── csg_operations.rs
├── analysis/
│   ├── analysis_conversion.rs
│   └── external_format.rs
└── mod.rs
```

## 移行計画

### Phase 1: 新構造の実装
1. 新しいtrait定義の作成
2. 既存traitの新構造へのマッピング

### Phase 2: 既存実装の移行
1. geo_primitivesの実装更新
2. 依存クレートの更新

### Phase 3: 旧構造の削除
1. 重複traitの削除
2. インポート文の整理

## 利点

1. **明確な責務分離**: 各traitの役割が明確
2. **一貫性**: 全ての形状で同じパターン
3. **拡張性**: 新しい形状や機能の追加が容易
4. **保守性**: どこに何があるかが分かりやすい

## 次のステップ

1. この提案の詳細レビュー
2. 具体的な実装スケジュール策定
3. 段階的移行の開始
