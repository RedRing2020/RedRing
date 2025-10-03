# geo-core クレート設計案

## トレラントモデリング基盤ライブラリ

### 目的

- 数値誤差に堅牢な幾何計算基盤
- 統一された許容誤差管理
- 2D/3D 統合ベクトル演算
- 抽象化された基本幾何要素

## モジュール構成

### 1. scalar/ - スカラー値管理

```rust
pub struct Scalar {
    value: f64,
    tolerance: f64,
}

pub trait TolerantEq<Rhs = Self> {
    fn tolerant_eq(&self, other: &Rhs) -> bool;
    fn tolerant_ne(&self, other: &Rhs) -> bool;
}

pub trait TolerantOrd<Rhs = Self> {
    fn tolerant_cmp(&self, other: &Rhs) -> Option<std::cmp::Ordering>;
    fn tolerant_lt(&self, other: &Rhs) -> bool;
    fn tolerant_le(&self, other: &Rhs) -> bool;
    fn tolerant_gt(&self, other: &Rhs) -> bool;
    fn tolerant_ge(&self, other: &Rhs) -> bool;
}
```

### 2. vector/ - 統合ベクトル演算

```rust
pub trait Vector<const D: usize>: Copy + Clone + PartialEq {
    type Scalar: TolerantEq + TolerantOrd;

    fn new(components: [Self::Scalar; D]) -> Self;
    fn components(&self) -> &[Self::Scalar; D];
    fn dimension() -> usize { D }

    // 基本演算
    fn dot(&self, other: &Self) -> Self::Scalar;
    fn norm(&self) -> Self::Scalar;
    fn normalize(&self) -> Option<Self>;

    // 許容誤差ベース比較
    fn tolerant_eq(&self, other: &Self) -> bool;
    fn tolerant_parallel(&self, other: &Self) -> bool;
    fn tolerant_perpendicular(&self, other: &Self) -> bool;
}

pub type Vector2D = Vector2<Scalar>;
pub type Vector3D = Vector3<Scalar>;

// 3D専用演算
pub trait Vector3D: Vector<3> {
    fn cross(&self, other: &Self) -> Self;
}
```

### 3. tolerance/ - 許容誤差管理

```rust
pub struct ToleranceContext {
    pub linear: f64,      // 長さの許容誤差
    pub angular: f64,     // 角度の許容誤差
    pub parametric: f64,  // パラメータ空間の許容誤差
}

pub trait ToleranceProvider {
    fn tolerance_context(&self) -> &ToleranceContext;
}

pub trait TolerantGeometry: ToleranceProvider {
    fn contains_point(&self, point: &Point, context: &ToleranceContext) -> bool;
    fn distance_to(&self, other: &dyn TolerantGeometry) -> f64;
    fn intersect_with(&self, other: &dyn TolerantGeometry) -> Vec<Point>;
}
```

### 4. robust/ - ロバスト計算

```rust
pub mod predicates {
    pub fn orient2d(a: &Point2D, b: &Point2D, c: &Point2D) -> Orientation;
    pub fn orient3d(a: &Point3D, b: &Point3D, c: &Point3D, d: &Point3D) -> Orientation;
    pub fn incircle(a: &Point2D, b: &Point2D, c: &Point2D, d: &Point2D) -> bool;
}

pub mod adaptive {
    pub fn adaptive_add(a: f64, b: f64) -> f64;
    pub fn adaptive_multiply(a: f64, b: f64) -> f64;
    pub fn exact_determinant(matrix: &[[f64; 3]; 3]) -> f64;
}

pub struct RobustSolver;
impl RobustSolver {
    pub fn newton_raphson_tolerant<F, G>(
        f: F, df: G,
        initial: f64,
        context: &ToleranceContext
    ) -> Option<f64>
    where
        F: Fn(f64) -> f64,
        G: Fn(f64) -> f64;
}
```

### 5. primitives/ - 幾何要素抽象化

```rust
pub trait Point<const D: usize>: Copy + Clone + ToleranceProvider {
    type Vector: Vector<D>;

    fn new(coords: [f64; D]) -> Self;
    fn coords(&self) -> &[f64; D];
    fn distance_to(&self, other: &Self) -> f64;
    fn translate(&self, vector: &Self::Vector) -> Self;
}

pub trait Curve<P: Point<D>, const D: usize>: TolerantGeometry {
    fn evaluate(&self, t: f64) -> P;
    fn derivative(&self, t: f64) -> P::Vector;
    fn tangent(&self, t: f64) -> Option<P::Vector>;
    fn curvature(&self, t: f64) -> f64;

    fn parameter_range(&self) -> (f64, f64);
    fn closest_point(&self, point: &P) -> (f64, P);
    fn length(&self) -> f64;
}

pub trait Surface<P: Point<3>>: TolerantGeometry {
    fn evaluate(&self, u: f64, v: f64) -> P;
    fn partial_u(&self, u: f64, v: f64) -> P::Vector;
    fn partial_v(&self, u: f64, v: f64) -> P::Vector;
    fn normal(&self, u: f64, v: f64) -> Option<P::Vector>;
}
```

## 利用例

```rust
use geo_core::*;

let ctx = ToleranceContext {
    linear: 1e-6,
    angular: 1e-8,
    parametric: 1e-10,
};

let p1 = Point3D::new([0.0, 0.0, 0.0]);
let p2 = Point3D::new([1.0, 0.0, 0.0]);
let v = Vector3D::from_points(&p1, &p2);

// 許容誤差ベース比較
if v.tolerant_parallel(&Vector3D::x_axis()) {
    println!("X軸と平行");
}

// ロバスト計算
let solver = RobustSolver;
if let Some(root) = solver.newton_raphson_tolerant(
    |x| x*x - 2.0,
    |x| 2.0*x,
    1.0,
    &ctx
) {
    println!("√2 ≈ {}", root);
}
```
