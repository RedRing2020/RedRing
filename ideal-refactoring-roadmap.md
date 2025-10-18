# RedRing 理想的リファクタリング手順計画

## 🎯 3 段階の理想的アーキテクチャ

### 最終目標構成

```
┌─────────────────┐
│     model       │ ← CAD業務ロジック・高次機能
│ (Scalar基礎)    │   - トレイト設計 (Curve3D, Surface)
└─────────────────┘   - 分類システム (geometry_kind)
         │             - CAD操作 (trim, extend, blend)
         ▼
┌─────────────────┐
│ geo_primitives  │ ← 基本プリミティブ形状
│ (Scalar基礎)    │   - Point, Line, Circle, Plane
└─────────────────┘   - Triangle, Polygon, Mesh
         │
         ▼
┌─────────────────┐
│   geo_nurbs     │ ← NURBS専門クレート
│ (Scalar基礎)    │   - NurbsCurve, NurbsSurface
└─────────────────┘   - Knot Vector, Control Points
         │             - NURBS専門アルゴリズム
         ▼
┌─────────────────┐
│   geo_core      │ ← 数学計算基盤
│                 │   - Scalar, Vector, ToleranceContext
└─────────────────┘   - ロバスト計算・高精度演算
```

## 📋 段階的実装手順

### 🔥 Phase 1: model(Scalar 基礎＋ CAD 設計)

#### 1.1 基礎型の Scalar 移植

```rust
// model/geometry/geometry3d/point.rs (Before)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,     // ← f64型
    y: f64,
    z: f64,
}

// model/geometry/geometry3d/point.rs (After)
use geo_core::{Scalar, ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]  // Copy削除
pub struct Point {
    x: Scalar,   // ← Scalar型
    y: Scalar,
    z: Scalar,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Scalar::new(x),
            y: Scalar::new(y),
            z: Scalar::new(z),
        }
    }

    // 既存API100%維持
    pub fn x(&self) -> f64 { self.x.value() }
    pub fn y(&self) -> f64 { self.y.value() }
    pub fn z(&self) -> f64 { self.z.value() }

    // トレラント比較対応（新機能）
    pub fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x.tolerant_eq(&other.x, context) &&
        self.y.tolerant_eq(&other.y, context) &&
        self.z.tolerant_eq(&other.z, context)
    }
}
```

#### 1.2 高次構造の Scalar 移植

```rust
// model/geometry/geometry3d/circle.rs
pub struct Circle {
    center: Point,    // ← 既にScalar化
    radius: Scalar,   // ← f64からScalar
    normal: Vector,   // ← 既にScalar化
}

// model/geometry/geometry3d/line.rs
pub struct Line {
    start: Point,     // ← 既にScalar化
    end: Point,       // ← 既にScalar化
    direction: Vector, // ← 既にScalar化
}
```

#### 1.3 トレイト実装の更新

```rust
// model/geometry_trait/curve3d.rs
impl Curve3D for Circle {
    fn evaluate(&self, t: f64) -> Point {
        // 内部でScalar演算使用、APIは維持
        let angle = Scalar::new(t * 2.0 * std::f64::consts::PI);
        // 高精度三角関数計算
    }

    fn length(&self) -> f64 {
        // Scalar精度でのPI計算
        (Scalar::new(2.0) * Scalar::new(std::f64::consts::PI) * self.radius.clone()).value()
    }
}
```

### 🚀 Phase 2: geo_primitives 移行（基本プリミティブ）

#### 2.1 プリミティブ形状を model→geo_primitives に移動

```rust
// geo_primitives/src/point.rs (modelから移植)
use geo_core::{Scalar, ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]
pub struct Point2D {
    x: Scalar,
    y: Scalar,
}

#[derive(Debug, Clone)]
pub struct Point3D {
    x: Scalar, y: Scalar, z: Scalar,
}

// geo_primitives/src/line.rs (modelから移植)
pub struct LineSegment2D { start: Point2D, end: Point2D }
pub struct LineSegment3D { start: Point3D, end: Point3D }

// geo_primitives/src/circle.rs (modelから移植)
pub struct Circle2D { center: Point2D, radius: Scalar }
pub struct Circle3D { center: Point3D, radius: Scalar, normal: Vector3D }
```

#### 2.2 統一インターフェース設計

```rust
// geo_primitives/src/lib.rs
pub trait GeometricPrimitive {
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> BoundingBox;
    fn measure(&self) -> Option<f64>;  // 長さ・面積・体積

    // トレラント比較対応
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveKind {
    Point, Line, Circle, Ellipse, Triangle,
    Polygon, Plane, Sphere, Cylinder,
    // NURBSは含まない（geo_nurbsが担当）
}
```

#### 2.3 model からの参照更新

```rust
// model/src/lib.rs
pub use geo_foundation::{
    Point2D, Point3D, LineSegment2D, LineSegment3D,
    Circle2D, Circle3D, Triangle2D, Triangle3D,
    GeometricPrimitive, PrimitiveKind
};

// model内の高次機能は継続
pub mod geometry_trait;  // Curve3D, Surface trait
pub mod geometry_kind;   // CurveKind3D, SurfaceKind
pub mod geometry_common; // IntersectionResult等
```

### 🌟 Phase 3: geo_nurbs クレート（NURBS 専門）

#### 3.1 geo_nurbs クレート作成

```toml
# geo_nurbs/Cargo.toml
[package]
name = "geo_nurbs"
version = "0.1.0"
edition = "2021"

[dependencies]
geo_core = { path = "../geo_core" }
geo_primitives = { path = "../geo_primitives" }

[features]
default = ["serde"]
serde = ["geo_core/serde"]
```

#### 3.2 NURBS 構造の移植

```rust
// geo_nurbs/src/nurbs_curve.rs (modelから移植)
use geo_core::{Scalar, Vector3D, ToleranceContext};
use geo_foundation::Point3D;

#[derive(Debug, Clone)]
pub struct NurbsCurve {
    degree: usize,
    knots: Vec<Scalar>,           // ← f64からScalar
    control_points: Vec<Point3D>, // ← geo_primitives::Point3D
    weights: Vec<Scalar>,         // ← f64からScalar
}

impl NurbsCurve {
    pub fn evaluate(&self, t: f64) -> Point3D {
        let param = Scalar::new(t);
        // 高精度NURBS演算
        self.evaluate_scalar(param)
    }

    fn evaluate_scalar(&self, t: Scalar) -> Point3D {
        // de Boor algorithm with Scalar precision
        // B-spline基底関数をScalar精度で計算
    }
}

// geo_nurbs/src/nurbs_surface.rs (modelから移植)
#[derive(Debug, Clone)]
pub struct NurbsSurface {
    u_degree: usize, v_degree: usize,
    u_knots: Vec<Scalar>, v_knots: Vec<Scalar>,
    control_points: Vec<Vec<Point3D>>,  // 2D配列
    weights: Vec<Vec<Scalar>>,
}
```

#### 3.3 NURBS 専門アルゴリズム

```rust
// geo_nurbs/src/algorithms/
pub mod basis_functions;      // B-spline基底関数
pub mod de_boor;             // de Boor algorithm
pub mod knot_insertion;      // ノット挿入
pub mod degree_elevation;    // 次数上昇
pub mod surface_tessellation; // サーフェス分割

// geo_nurbs/src/algorithms/basis_functions.rs
pub fn basis_function(i: usize, p: usize, knots: &[Scalar], u: Scalar) -> Scalar {
    // Cox-de Boor 再帰公式をScalar精度で実装
}

pub fn basis_function_derivatives(
    i: usize, p: usize, knots: &[Scalar], u: Scalar, n: usize
) -> Vec<Scalar> {
    // 基底関数の導関数をScalar精度で計算
}
```

## 🔧 実装上の考慮事項

### 3.1 API 互換性の保証

```rust
// model継続使用のためのAPI保持
// model/src/compatibility.rs
#[deprecated(note = "Use geo_foundation::Point3D")]
pub type Point = geo_foundation::Point3D;

#[deprecated(note = "Use geo_nurbs::NurbsCurve")]
pub type NurbsCurve = geo_nurbs::NurbsCurve;
```

### 3.2 段階的移行サポート

```rust
// 各Phaseでフィーチャーフラグ使用
#[cfg(feature = "phase1_scalar")]
type CoordType = Scalar;
#[cfg(not(feature = "phase1_scalar"))]
type CoordType = f64;
```

### 3.3 依存関係管理

```
geo_core (数学基盤)
    ↑
geo_primitives (基本形状)
    ↑
geo_nurbs (NURBS専門)
    ↑
model (CAD業務ロジック)
```

## 🎁 期待される効果

### 責務分離の明確化

- **geo_core**: 数学計算・精度管理
- **geo_primitives**: 基本幾何形状・統一 API
- **geo_nurbs**: NURBS 専門・高度アルゴリズム
- **model**: CAD 業務ロジック・トレイト設計

### 保守性の向上

- 各クレートが独立してテスト可能
- NURBS 機能の独立更新
- プリミティブ形状の再利用促進

### 性能の最適化

- Scalar 精度による数値安定性
- 専門クレートでの最適化実装
- 必要な機能のみの選択的利用

この手順により、理想的なアーキテクチャを段階的に構築できます！
