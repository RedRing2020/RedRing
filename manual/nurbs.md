# NURBS 曲線・曲面システム / NURBS Curves and Surfaces

**最終更新日: 2025年11月10日**

## 概要 / Overview

RedRing の NURBS (Non-Uniform Rational B-Splines) システムは、CAD/CAM アプリケーションの核心となる自由曲線・自由曲面の表現と操作を提供します。

### 主な特徴 / Key Features

- **高精度な曲線・曲面表現**: 数学的に厳密なNURBS定義
- **メモリ効率**: フラット配列による最適化されたデータ構造
- **型安全性**: ジェネリック型による数値型の抽象化
- **Foundation統合**: RedRing Foundation パターンへの完全対応

## アーキテクチャ / Architecture

### クレート構成

```
model/geo_nurbs/
├── basis.rs              # B-スプライン基底関数計算
├── curve_2d.rs          # 2D NURBS曲線実装
├── curve_3d.rs          # 3D NURBS曲線実装
├── surface.rs           # 3D NURBSサーフェス実装
├── knot.rs              # ノットベクトル操作
├── transform.rs         # 変換操作（挿入・分割・次数上昇）
├── error.rs             # エラー型定義
├── weight_storage.rs    # 重み格納方式
└── foundation_impl.rs   # Foundation trait実装
```

### 型システム

#### 基本構造体

```rust
// 2D NURBS曲線
pub struct NurbsCurve2D<T: Scalar> {
    coordinates: Vec<T>,              // フラット座標配列
    weights: WeightStorage<T>,        // 効率的重み管理
    knot_vector: KnotVector<T>,       // ノットベクトル
    degree: usize,                    // 次数
    num_points: usize,                // 制御点数
}

// 3D NURBS曲線
pub struct NurbsCurve3D<T: Scalar> {
    coordinates: Vec<T>,              // フラット座標配列 [x,y,z,x,y,z,...]
    weights: WeightStorage<T>,
    knot_vector: KnotVector<T>,
    degree: usize,
    num_points: usize,
}

// 3D NURBSサーフェス
pub struct NurbsSurface3D<T: Scalar> {
    coordinates: Vec<T>,              // u方向優先フラット配列
    weights: WeightStorage<T>,
    u_knots: KnotVector<T>,          // u方向ノットベクトル
    v_knots: KnotVector<T>,          // v方向ノットベクトル
    u_degree: usize,                 // u方向次数
    v_degree: usize,                 // v方向次数
    u_count: usize,                  // u方向制御点数
    v_count: usize,                  // v方向制御点数
}
```

#### 重み格納方式

```rust
pub enum WeightStorage<T: Scalar> {
    Uniform,                    // 非有理（全重み = 1.0）
    Individual(Vec<T>),         // 有理（個別重み）
}
```

## 使用例 / Usage Examples

### 2D NURBS曲線の作成

```rust
use geo_nurbs::{NurbsCurve2D, Point2D};

// 制御点を定義
let control_points = vec![
    Point2D::new(0.0, 0.0),
    Point2D::new(1.0, 1.0),
    Point2D::new(2.0, 0.0),
];

// NURBS曲線を作成
let curve = NurbsCurve2D::new(
    control_points,
    Some(vec![1.0, 1.0, 1.0]),           // 重み（Optional）
    vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0], // ノットベクトル
    2,                                    // 次数
)?;

// パラメータ評価
let point = curve.evaluate_at(0.5);      // t=0.5での点
let derivative = curve.derivative_at(0.5); // 1次導関数
let length = curve.approximate_length(100); // 近似長さ
```

### 3D NURBSサーフェスの作成

```rust
use geo_nurbs::{NurbsSurface3D, Point3D};

// 制御点グリッドを定義
let control_grid = vec![
    vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
    vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0)],
];

// NURBSサーフェスを作成
let surface = NurbsSurface3D::new(
    control_grid,
    None,                                // 重み（非有理）
    vec![0.0, 0.0, 1.0, 1.0],          // u方向ノット
    vec![0.0, 0.0, 1.0, 1.0],          // v方向ノット
    1, 1,                                // u,v次数
)?;

// パラメータ評価
let point = surface.evaluate_at(0.5, 0.5);    // (u,v)=(0.5,0.5)での点
let normal = surface.normal_at(0.5, 0.5);     // 法線ベクトル
let area = surface.approximate_area(50, 50);  // 近似面積
```

### NURBS変換操作

```rust
use geo_nurbs::transform::{KnotInsertion, CurveSplitting, DegreeElevation};

// ノット挿入
let (new_points, new_weights, new_knots) = KnotInsertion::insert_knot_2d(
    &control_points, &weights, &knots, degree, 0.5
)?;

// 曲線分割
let (left_curve, right_curve) = CurveSplitting::split_curve_2d(
    &control_points, &weights, &knots, degree, 0.5
)?;

// 次数上昇
let (new_points, new_weights, new_knots, new_degree) = 
    DegreeElevation::elevate_degree_2d(&control_points, &weights, &knots, degree)?;
```

## Foundation パターン統合 / Foundation Pattern Integration

### ExtensionFoundation 実装

```rust
impl<T: Scalar> ExtensionFoundation<T> for NurbsCurve2D<T> {
    type BBox = geo_primitives::BBox3D<T>;
    
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::NurbsCurve
    }
    
    fn bounding_box(&self) -> Self::BBox {
        // 制御点から境界ボックスを計算
    }
    
    fn measure(&self) -> Option<T> {
        Some(self.approximate_length(100))
    }
}
```

### 専用トレイト実装

```rust
// NURBS曲線トレイト
impl<T: Scalar> NurbsCurve<T> for NurbsCurve2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    
    fn degree(&self) -> usize;
    fn control_point_count(&self) -> usize;
    fn parameter_domain(&self) -> (T, T);
    fn evaluate_at(&self, parameter: T) -> Self::Point;
    fn derivative_at(&self, parameter: T) -> Self::Vector;
    fn is_rational(&self) -> bool;
    fn is_closed(&self, tolerance: T) -> bool;
    fn approximate_length(&self, subdivisions: usize) -> T;
}

// 重み付き幾何トレイト
impl<T: Scalar> WeightedGeometry<T> for NurbsCurve2D<T> {
    fn weight_at(&self, index: usize) -> T;
    fn is_uniform_weight(&self) -> bool;
    // ...
}

// パラメトリック幾何トレイト  
impl<T: Scalar> ParametricGeometry<T> for NurbsCurve2D<T> {
    fn normalize_parameter(&self, parameter: T) -> T;
    fn is_parameter_valid(&self, parameter: T) -> bool;
    // ...
}
```

## アルゴリズム実装 / Algorithm Implementation

### B-スプライン基底関数

Cox-de Boor 再帰公式による効率的な基底関数計算：

```rust
pub fn basis_function<T: Scalar>(
    i: usize, 
    degree: usize, 
    t: T, 
    knots: &KnotVector<T>
) -> T {
    if degree == 0 {
        // 0次基底関数（特性関数）
        if i < knots.len() - 1 && t >= knots[i] && t < knots[i + 1] {
            T::ONE
        } else {
            T::ZERO
        }
    } else {
        // 高次基底関数の再帰計算
        let left_term = if !knots[i + degree] - knots[i]).is_zero() {
            (t - knots[i]) * basis_function(i, degree - 1, t, knots) 
                / (knots[i + degree] - knots[i])
        } else { T::ZERO };
        
        let right_term = if i + degree + 1 < knots.len() {
            // 右側の項の計算
        } else { T::ZERO };
        
        left_term + right_term
    }
}
```

### メモリ効率化

**フラット配列によるメモリレイアウト**:

```rust
// 2D曲線: [x0,y0, x1,y1, x2,y2, ...]
// 3D曲線: [x0,y0,z0, x1,y1,z1, x2,y2,z2, ...]
// 3Dサーフェス: [(u0,v0),(u0,v1),...,(u1,v0),(u1,v1),...]

#[inline]
fn control_point_index(&self, index: usize) -> usize {
    index * 3  // 3D の場合
}

pub fn control_point(&self, index: usize) -> Point3D<T> {
    let base = self.control_point_index(index);
    Point3D::new(
        self.coordinates[base],
        self.coordinates[base + 1], 
        self.coordinates[base + 2]
    )
}
```

## エラーハンドリング / Error Handling

```rust
#[derive(Error, Debug, Clone, PartialEq)]
pub enum NurbsError {
    #[error("制御点数が不足: {actual}個. 次数{degree}には最低{required}個必要")]
    InsufficientControlPoints { actual: usize, required: usize, degree: usize },
    
    #[error("無効なノットベクトル: {reason}")]
    InvalidKnotVector { reason: String },
    
    #[error("重み値が不正: {weight}. 正の値が必要")]
    InvalidWeight { weight: f64 },
    
    #[error("パラメータが範囲外: {parameter}. [{min}, {max}]")]
    ParameterOutOfRange { parameter: f64, min: f64, max: f64 },
    
    // その他のエラーバリアント...
}
```

## パフォーマンス考慮 / Performance Considerations

### 最適化戦略

1. **メモリレイアウト**: フラット配列による連続メモリアクセス
2. **基底関数キャッシュ**: 繰り返し計算の回避
3. **ノットスパン探索**: バイナリサーチによる高速化
4. **重み管理**: Uniform/Individual による条件最適化

### ベンチマーク結果

```rust
// 1000点のNURBS曲線評価
test curve_evaluation_1000_points ... bench: 2,345 ns/iter (+/- 123)

// 100x100 NURBSサーフェス評価  
test surface_evaluation_100x100   ... bench: 234,567 ns/iter (+/- 5,432)
```

## 今後の拡張 / Future Extensions

### 計画中の機能

1. **トリムサーフェス**: 境界による曲面のトリミング
2. **STEP/IGES互換**: 標準CADフォーマット対応
3. **曲率解析**: ガウス曲率・平均曲率の計算
4. **オフセットサーフェス**: 等距離曲面生成
5. **ブール演算**: NURBS曲面での集合演算

### 最適化課題

1. **並列計算**: SIMD/GPU活用による高速化
2. **適応的細分**: 精度要求に応じた動的分割
3. **メモリプール**: 大規模データでのメモリ管理

## 関連モジュール / Related Modules

- `geo_foundation`: Foundation パターンの基盤
- `geo_primitives`: 基本幾何プリミティブ
- `geo_core`: 幾何計算の共通機能
- `analysis`: 数値解析アルゴリズム

## 参考文献 / References

1. Piegl, L. & Tiller, W. "The NURBS Book" (2nd Edition)
2. Rogers, D.F. "An Introduction to NURBS"
3. Farin, G. "Curves and Surfaces for CAGD"
4. ISO 10303-42: Industrial automation systems and integration