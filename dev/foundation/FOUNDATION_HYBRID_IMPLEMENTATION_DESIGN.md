# ハイブリッド実装方針の詳細設計

**作成日**: 2025年11月16日
**最終更新**: 2025年11月16日

## 概要

Core機能は形状別統合、Extensions機能は機能別分離のハイブリッド方針を採用。

## 実装構造設計

### Core：形状別統合（安定・コンパクト）

#### ファイル構造
```text
core/
├── point_core_traits.rs      # Point用4core機能統合
├── vector_core_traits.rs     # Vector用4core機能統合
├── circle_core_traits.rs     # Circle用4core機能統合
├── line_core_traits.rs       # Line用4core機能統合
├── plane_core_traits.rs      # Plane用4core機能統合
├── arc_core_traits.rs        # Arc用4core機能統合
├── ellipse_core_traits.rs    # Ellipse用4core機能統合
├── triangle_core_traits.rs   # Triangle用4core機能統合
├── bbox_core_traits.rs       # BBox用4core機能統合
└── mod.rs
```

#### 実装テンプレート
```rust
// core/point_core_traits.rs
//! Point形状のCore機能統合トレイト定義

use crate::Scalar;

/// 1. Constructor - Point生成機能
pub trait PointConstructor<T: Scalar> {
    /// 基本コンストラクタ
    fn new(x: T, y: T) -> Self;
    /// 原点作成
    fn origin() -> Self;
    /// 別の点からコピー作成
    fn from_point(other: &Self) -> Self;
}

/// 2. Properties - Point基本情報取得
pub trait PointProperties<T: Scalar> {
    type Point;
    type BBox;

    /// X座標取得
    fn x(&self) -> T;
    /// Y座標取得
    fn y(&self) -> T;
    /// 位置（自分自身）
    fn position(&self) -> Self::Point;
    /// 境界ボックス（微小範囲）
    fn bounding_box(&self) -> Self::BBox;
    /// 次元情報
    fn dimension(&self) -> u32; // Pointは0次元
}

/// 3. Transform - Point座標変換
pub trait PointTransform<T: Scalar> {
    type Matrix4x4;
    type Vector;
    type Angle;
    type Output;

    /// Matrix変換
    fn transform_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;
    /// 平行移動
    fn translate(&self, offset: &Self::Vector) -> Self::Output;
    /// 回転（Pointでは無効だが、統一インターフェースのため定義）
    fn rotate(&self, center: &Self, axis: &Self::Vector, angle: Self::Angle) -> Self::Output;
    /// スケール（中心からの距離をスケール）
    fn scale(&self, center: &Self, factor: T) -> Self::Output;
}

/// 4. Measure - Point計量機能
pub trait PointMeasure<T: Scalar> {
    /// 他の点までの距離
    fn distance_to(&self, other: &Self) -> T;
    /// 原点からの距離
    fn distance_from_origin(&self) -> T;
    /// 面積（Pointは0）
    fn area(&self) -> Option<T> { None }
    /// 体積（Pointは0）
    fn volume(&self) -> Option<T> { None }
    /// 長さ（Pointは0）
    fn length(&self) -> Option<T> { None }
}
```

### Extensions：機能別分離（流動・拡張対応）

#### Extensionsファイル構造
```text
extensions/
├── collision/
│   ├── mod.rs
│   ├── basic_collision.rs      # 基本衝突検出
│   ├── spatial_query.rs        # 空間検索
│   ├── distance_calculation.rs # 距離計算
│   └── containment_test.rs     # 包含判定
├── intersection/
│   ├── mod.rs
│   ├── point_intersection.rs   # 点交差
│   ├── line_intersection.rs    # 線交差
│   ├── curve_intersection.rs   # 曲線交差
│   ├── surface_intersection.rs # サーフェス交差
│   └── volume_intersection.rs  # 体積交差
├── boolean/
│   ├── mod.rs
│   ├── union_operations.rs     # 和集合演算
│   ├── intersection_operations.rs # 積集合演算
│   ├── difference_operations.rs # 差集合演算
│   └── symmetric_difference.rs # 対称差集合
├── analysis/
│   ├── mod.rs
│   ├── analysis_conversion.rs  # Analysis型変換
│   ├── matrix_conversion.rs    # Matrix変換
│   └── external_format.rs      # 外部フォーマット変換
└── mod.rs
```

#### 実装例
```rust
// extensions/collision/basic_collision.rs
//! 基本衝突検出機能

use crate::Scalar;

/// 基本衝突検出トレイト
pub trait BasicCollision<T: Scalar, Other> {
    /// 衝突判定
    fn intersects(&self, other: &Other, tolerance: T) -> bool;
    /// 重なり判定
    fn overlaps(&self, other: &Other, tolerance: T) -> bool;
    /// 最短距離
    fn distance_to(&self, other: &Other) -> T;
    /// 最近点
    fn closest_points(&self, other: &Other) -> (Self::Point, Other::Point);
}

/// 高度な衝突検出トレイト（将来拡張用）
pub trait AdvancedCollision<T: Scalar, Other>: BasicCollision<T, Other> {
    /// 衝突時刻計算（動的衝突検出）
    fn collision_time(&self, velocity: &Self::Vector, other: &Other, other_velocity: &Other::Vector) -> Option<T>;
    /// 衝突応答計算
    fn collision_response(&self, other: &Other) -> (Self::Vector, Other::Vector);
}
```

## 実装上の利点

### Core（形状別統合）の利点
1. **開発効率**: 1つの形状の全機能が1ファイル
2. **理解しやすさ**: 依存関係が明確
3. **実装一貫性**: 4つのcore機能が統一された構造
4. **メンテナンス性**: 形状単位での変更管理

### Extensions（機能別分離）の利点
1. **拡張性**: 新機能を独立して追加
2. **専門性**: 複雑なアルゴリズムの分離
3. **並行開発**: チームでの分担作業が容易
4. **テスト独立性**: 機能単位でのテスト

## 移行戦略

### Phase 1: Core統合（2-3日）
1. point_core_traits.rs作成
2. vector_core_traits.rs作成
3. circle_core_traits.rs作成
4. 他の主要形状のcore統合

### Phase 2: Extensions分離（2-3日）
1. collision系の機能別分離
2. intersection系の機能別分離
3. analysis系の整理

### Phase 3: 実装更新（3-4日）
1. geo_primitivesの実装更新
2. テスト実行・修正
3. 依存関係の整理

## 成功指標

- [ ] 各形状でcore4機能が統合されている
- [ ] extensions機能が独立して拡張可能
- [ ] 全テストパス
- [ ] ビルド時間維持
- [ ] Clippy警告ゼロ
