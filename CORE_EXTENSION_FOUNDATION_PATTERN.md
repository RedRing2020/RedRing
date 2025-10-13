# RedRing Core/Extension Foundation パターン

## 概要

RedRing CAD/CAM プラットフォームでは、幾何形状の機能を **Core（中核）** と **Extension（拡張）** に分離する設計パターンを採用しています。これにより、用途に応じて必要な機能のみを使用できる柔軟で効率的なアーキテクチャを実現しています。

## 設計思想

### 問題意識

従来の幾何ライブラリでは、すべての機能が一つのクラスに集約されるため、以下の問題が発生していました：

1. **肥大化した API**: 基本的な操作にも重い依存関係
2. **保守性の低下**: 250 行を超える巨大ファイル
3. **用途別最適化の困難**: レンダリング用と解析用で同じ重いインターフェース
4. **境界の曖昧さ**: どの機能が必須で、どれが拡張なのか不明確

### 解決方針

**Core/Extension 分離パターン** により、以下を実現します：

- **Core Foundation**: 必須機能のみ（軽量・高速・必須実装）
- **Extension Foundation**: 拡張機能群（オプション実装・機能豊富）

## アーキテクチャ

### Core Foundation（中核基盤）

**目的**: レンダリング・衝突判定・空間インデックスに必要な基本機能

**特徴**:

- ✅ 軽量・高速
- ✅ 必須実装
- ✅ 最小依存関係
- ✅ 型安全

**含まれる機能**:

- 構築メソッド（`new`, `from_center_radius`）
- アクセサメソッド（`center`, `radius`）
- 基本計量（`area`, `circumference`）
- 基本包含判定（`contains_point_inside`, `distance_to_point`）
- 基本パラメータ化（`point_at_parameter`, `tangent_at_parameter`）
- 境界ボックス（`bounding_box`）

### Extension Foundation（拡張基盤）

**目的**: 高度な操作・分析・変換機能

**特徴**:

- ✨ オプション実装
- ✨ 機能豊富
- ✨ 専門特化
- ✨ 段階的追加可能

**含まれる機能**:

- 高度な構築（`from_three_points`, `unit_circle`）
- 便利メソッド（`diameter`, `point_at_angle`）
- 変形操作（`scale`, `translate`, `move_to`）
- 空間関係（`intersects_circle`, `contains_circle`）
- 次元変換（`to_3d`, `to_3d_at_z`）

## ファイル構造

### Before（分離前）

```
circle_2d.rs        // 280行の巨大ファイル
├── Core 機能
├── Extension 機能
└── トレイト実装
```

### After（分離後）

```
circle_2d.rs              // 120行（Core実装）
├── 構築メソッド
├── アクセサメソッド
├── 基本計量メソッド
├── 基本包含メソッド
├── 基本パラメータメソッド
├── 境界ボックスメソッド
└── CoreFoundation トレイト実装

circle_2d_extensions.rs   // 130行（Extension実装）
├── 高度な構築メソッド
├── 便利メソッド
├── 変形メソッド
├── 空間関係メソッド
├── 次元変換メソッド
└── ExtensionFoundation トレイト実装
```

## トレイト階層

### Core Foundation トレイト

```rust
// 基盤トレイト（必須）
pub trait CoreFoundation<T: Scalar> {
    type Point;
    type Vector;
    type BBox;
    fn bounding_box(&self) -> Self::BBox;
}

// 基本機能トレイト（オプション）
pub trait BasicMetrics<T: Scalar> { ... }
pub trait BasicContainment<T: Scalar>: CoreFoundation<T> { ... }
pub trait BasicParametric<T: Scalar>: CoreFoundation<T> { ... }
pub trait BasicDirectional<T: Scalar>: CoreFoundation<T> { ... }
```

### Extension Foundation トレイト

```rust
// 拡張基盤トレイト
pub trait ExtensionFoundation<T: Scalar> {
    type BBox: AbstractBBox<T>;
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> Self::BBox;
    fn measure(&self) -> Option<T>;
}

// 拡張機能トレイト
pub trait TransformableExtension<T: Scalar>: ExtensionFoundation<T> { ... }
pub trait MeasurableExtension<T: Scalar>: ExtensionFoundation<T> { ... }
pub trait SpatialExtension<T: Scalar>: ExtensionFoundation<T> { ... }
pub trait CollectionExtension<T: Scalar> { ... }
```

## 実装例

### Circle2D Core 実装

```rust
//! circle_2d.rs（Core実装）

impl<T: Scalar> Circle2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================
    pub fn new(center: Point2D<T>, radius: T) -> Option<Self> { ... }
    pub fn from_center_radius(center: Point2D<T>, radius: T) -> Option<Self> { ... }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================
    pub fn center(&self) -> Point2D<T> { ... }
    pub fn radius(&self) -> T { ... }

    // ========================================================================
    // Core Metrics Methods
    // ========================================================================
    pub fn circumference(&self) -> T { ... }
    pub fn area(&self) -> T { ... }

    // ========================================================================
    // Core Containment Methods
    // ========================================================================
    pub fn contains_point_inside(&self, point: &Point2D<T>) -> bool { ... }
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T { ... }

    // ========================================================================
    // Core Parametric Methods
    // ========================================================================
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> { ... }
    pub fn tangent_at_parameter(&self, t: T) -> Vector2D<T> { ... }

    // ========================================================================
    // Core Bounding Box Method
    // ========================================================================
    pub fn bounding_box(&self) -> crate::BBox2D<T> { ... }
}

// Core Foundation トレイト実装
impl<T: Scalar> CoreFoundation<T> for Circle2D<T> { ... }
impl<T: Scalar> BasicMetrics<T> for Circle2D<T> { ... }
impl<T: Scalar> BasicContainment<T> for Circle2D<T> { ... }
impl<T: Scalar> BasicParametric<T> for Circle2D<T> { ... }
```

### Circle2D Extension 実装

```rust
//! circle_2d_extensions.rs（Extension実装）

impl<T: Scalar> Circle2D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================
    pub fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self> { ... }
    pub fn unit_circle() -> Self { ... }

    // ========================================================================
    // Convenience Methods (Extension)
    // ========================================================================
    pub fn diameter(&self) -> T { ... }
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> { ... }
    pub fn contains_point_on_circle(&self, point: &Point2D<T>, tolerance: T) -> bool { ... }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================
    pub fn scale(&self, factor: T) -> Option<Self> { ... }
    pub fn translate(&self, offset: Vector2D<T>) -> Self { ... }
    pub fn move_to(&self, new_center: Point2D<T>) -> Self { ... }

    // ========================================================================
    // Spatial Relationship Methods (Extension)
    // ========================================================================
    pub fn intersects_circle(&self, other: &Self) -> bool { ... }
    pub fn contains_circle(&self, other: &Self) -> bool { ... }

    // ========================================================================
    // Dimension Extension Methods (Extension)
    // ========================================================================
    pub fn to_3d(&self) -> crate::Circle3D<T> { ... }
    pub fn to_3d_at_z(&self, z: T) -> crate::Circle3D<T> { ... }
}

// Extension Foundation トレイト実装（将来予定）
// impl<T: Scalar> ExtensionFoundation<T> for Circle2D<T> { ... }
// impl<T: Scalar> TransformableExtension<T> for Circle2D<T> { ... }
// impl<T: Scalar> SpatialExtension<T> for Circle2D<T> { ... }
```

### Ray2D実装例（2025年10月完了）

**完全な Core/Extension Foundation パターンの実装例**

#### Core Foundation実装（ray_2d.rs - 183行）
```rust
//! Ray2D - 2次元半無限直線の実装
//! パラメータ表現: point = origin + t * direction (t ≥ 0)

pub struct Ray2D<T: Scalar> {
    origin: Point2D<T>,      // 起点
    direction: Vector2D<T>,  // 方向ベクトル（正規化済み）
}

impl<T: Scalar> Ray2D<T> {
    // Core 作成メソッド
    pub fn new(origin: Point2D<T>, direction: Vector2D<T>) -> Option<Self> { ... }
    pub fn from_points(start: Point2D<T>, through: Point2D<T>) -> Option<Self> { ... }
    
    // Core アクセサ
    pub fn origin(&self) -> Point2D<T> { ... }
    pub fn direction(&self) -> Vector2D<T> { ... }
    
    // Core 基本操作
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool { ... }
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T { ... }
    pub fn to_infinite_line(&self) -> InfiniteLine2D<T> { ... }
}

// Core Foundation トレイト実装
impl<T: Scalar> CoreFoundation<T> for Ray2D<T> { ... }
impl<T: Scalar> BasicParametric<T> for Ray2D<T> { ... }
impl<T: Scalar> BasicDirectional<T> for Ray2D<T> { ... }
impl<T: Scalar> BasicContainment<T> for Ray2D<T> { ... }
```

#### Extension Foundation実装（ray_2d_extensions.rs - 174行）
```rust
//! Ray2D Extensions - 高度な幾何演算・変換操作

impl<T: Scalar> Ray2D<T> {
    // 特殊作成メソッド
    pub fn x_axis_ray(x: T, y: T) -> Self { ... }
    pub fn y_axis_ray(x: T, y: T) -> Self { ... }
    pub fn from_angle(angle: Angle<T>) -> Self { ... }
    
    // 交点計算
    pub fn intersection_with_ray(&self, other: &Self) -> Option<Point2D<T>> { ... }
    pub fn intersection_with_line_segment(&self, segment: &LineSegment2D<T>) -> Option<Point2D<T>> { ... }
    
    // 幾何関係判定
    pub fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool { ... }
    pub fn is_perpendicular_to(&self, other: &Self, tolerance: T) -> bool { ... }
    
    // 変換操作
    pub fn translate(&self, translation: Vector2D<T>) -> Self { ... }
    pub fn rotate_around_point(&self, angle: Angle<T>, center: Point2D<T>) -> Self { ... }
    
    // 分割・測定
    pub fn split_at_parameter(&self, t: T) -> Option<(LineSegment2D<T>, Self)> { ... }
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T { ... }
}
```

**Ray2D実装の特徴**:
- ✅ **tolerance引数による厳密制御**: 全ての判定メソッドでtolerance指定可能
- ✅ **半無限直線の完全実装**: t ≥ 0のパラメータ範囲制約
- ✅ **Core/Extension完全分離**: 基本機能183行、拡張機能174行
- ✅ **包括的テストスイート**: ray_2d_tests.rsで全機能検証済み

## 適用済み幾何プリミティブ

| プリミティブ | Core実装 | Extension実装 | テスト | 状態 |
|-------------|---------|---------------|-------|------|
| Circle2D    | ✅      | ✅            | ✅    | 完了 |
| **Ray2D**   | ✅      | ✅            | ✅    | **完了** |
| Arc2D       | ✅      | ✅            | ✅    | 完了 |
| LineSegment2D | ✅    | ✅            | ✅    | 完了 |
| Ellipse2D   | ✅      | ✅            | ✅    | 完了 |
| Point2D/3D  | ✅      | ✅            | ✅    | 完了 |
| Vector2D/3D | ✅      | ✅            | ✅    | 完了 |

## 利用パターン

### パターン 1: Core のみ使用（軽量・高速）

```rust
use geo_primitives::Circle2D;

// 基本的な円の操作
let circle = Circle2D::new(center, radius)?;

// Core機能のみ
let area = circle.area();
let bbox = circle.bounding_box();
let point = circle.point_at_parameter(0.5);
let contains = circle.contains_point_inside(&test_point);
```

**特徴**:

- ✅ 軽量（最小依存関係）
- ✅ 高速（基本機能のみ）
- ✅ レンダリング・衝突判定に最適

### パターン 2: Extension を含む使用（高機能）

```rust
use geo_primitives::Circle2D;
// Extensions は自動的にインポートされる

// 高度な円の操作
let unit_circle = Circle2D::unit_circle();                    // Extension
let from_points = Circle2D::from_three_points(p1, p2, p3)?;  // Extension
let diameter = circle.diameter();                             // Extension
let scaled = circle.scale(2.0)?;                              // Extension
let moved = circle.translate(Vector2D::new(1.0, 2.0));       // Extension
let intersects = circle.intersects_circle(&other);           // Extension
let circle_3d = circle.to_3d();                              // Extension
```

**特徴**:

- ✨ 高機能（拡張メソッド利用可能）
- ✨ 変形・解析・変換操作
- ✨ CAD/CAM 専用機能に最適

## メリット

### 1. 段階的実装

- 最小限の Core から開始
- 必要に応じて Extension を追加
- プロジェクトの成熟に合わせて拡張

### 2. 用途別最適化

- **レンダリング用**: Core のみ（軽量・高速）
- **解析用**: Core + Extension（高機能）
- **CAM 用**: Core + 専用 Extension

### 3. 保守性向上

- 責務分離により理解が容易
- ファイルサイズの削減（280 行 → 120+130 行）
- 機能別の修正・テストが可能

### 4. 拡張性

- 新しい Extension を後から追加可能
- 既存コードに影響を与えない拡張
- プラグイン的な機能追加

## 将来拡張

このパターンにより、以下のような新しい Extension を容易に追加できます：

### CAD 専用拡張

- `CADExtension<T>`: CAD 固有の操作
- `DraftingExtension<T>`: 製図支援機能
- `ConstraintExtension<T>`: 拘束システム

### CAM 専用拡張

- `CAMExtension<T>`: 切削パス生成
- `ToolpathExtension<T>`: 工具経路最適化
- `MachiningExtension<T>`: 加工シミュレーション

### 解析専用拡張

- `MeshExtension<T>`: メッシュ操作
- `SimulationExtension<T>`: 物理シミュレーション
- `OptimizationExtension<T>`: 最適化アルゴリズム

## 実装ガイドライン

### 1. Core 実装のガイドライン

- **最小機能**: 必須機能のみ実装
- **高速性**: パフォーマンスを重視
- **安定性**: API の変更を最小限に
- **依存関係**: 最小限の依存関係

### 2. Extension 実装のガイドライン

- **選択的実装**: 必要な Extension のみ
- **機能豊富**: 専門的な機能を提供
- **型安全性**: ジェネリック型の活用
- **エラー処理**: 適切なエラーハンドリング

### 3. ファイル命名規則

```
{geometry_name}.rs              // Core実装
{geometry_name}_extensions.rs   // Extension実装
{geometry_name}_tests.rs        // テスト
```

### 4. モジュール宣言

```rust
// lib.rs
pub mod circle_2d;              // Core
pub mod circle_2d_extensions;   // Extension
```

## 結論

Core/Extension Foundation パターンは、RedRing CAD/CAM プラットフォームの柔軟性と効率性を支える重要な設計原則です。このパターンにより、用途に応じた最適化と段階的な機能拡張を実現し、保守性の高いコードベースを維持できます。

今後の幾何プリミティブ実装においても、このパターンを一貫して適用することで、統一性のある高品質なライブラリを構築していきます。
