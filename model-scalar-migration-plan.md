# Model Geometry → Scalar 移植計画

## 🎯 統合方針：CAD 設計保持 + 数値堅牢性強化

### 基本思想

- ✅ **model の CAD 設計思想を 100%保持**（トレイト、分類システム、業務ロジック）
- ✅ **基礎データ型を f64 → Scalar 移植**（数値精度・トレラント比較対応）
- ✅ **geo_primitives 削除**（不要な重複レイヤー排除）

## 📋 段階的移植計画

### Phase 1: 基础型移植（Point, Vector）

#### Before（現在）

```rust
// model/geometry/geometry3d/point.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,     // ← f64型
    y: f64,
    z: f64,
}
```

#### After（移植後）

```rust
// model/geometry/geometry3d/point.rs
use geo_core::{Scalar, ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]  // Copy削除（Scalarの制約）
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

    // APIは100%維持
    pub fn x(&self) -> f64 { self.x.value() }
    pub fn y(&self) -> f64 { self.y.value() }
    pub fn z(&self) -> f64 { self.z.value() }

    // トレラント比較対応
    pub fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x.tolerant_eq(&other.x, context) &&
        self.y.tolerant_eq(&other.y, context) &&
        self.z.tolerant_eq(&other.z, context)
    }
}
```

### Phase 2: 高次構造移植（Circle, Line 等）

#### Before

```rust
// model/geometry/geometry3d/circle.rs
pub struct Circle {
    center: Point,
    radius: f64,      // ← f64型
    normal: Vector,
}
```

#### After

```rust
pub struct Circle {
    center: Point,    // ← 既にScalar化されたPoint
    radius: Scalar,   // ← Scalar型
    normal: Vector,   // ← 既にScalar化されたVector
}

impl Circle {
    pub fn new(center: Point, radius: f64, normal: Vector) -> Self {
        Self {
            center,
            radius: Scalar::new(radius),  // f64→Scalar変換
            normal,
        }
    }

    // APIは100%維持
    pub fn radius(&self) -> f64 { self.radius.value() }

    // 高精度計算に対応
    pub fn area(&self) -> f64 {
        let pi = Scalar::new(std::f64::consts::PI);
        (pi * self.radius.clone() * self.radius.clone()).value()
    }
}
```

## 🔧 実装上の考慮事項

### 1. **Copy trait 対応**

```rust
// 選択肢A: Clone使用（推奨）
impl Point {
    pub fn translate(&self, vector: &Vector) -> Self {
        Point::new(
            self.x().clone() + vector.x().clone(),  // Clone使用
            self.y().clone() + vector.y().clone(),
            self.z().clone() + vector.z().clone(),
        )
    }
}

// 選択肢B: Copy wrapper（軽量化）
#[derive(Debug, Clone, Copy)]
pub struct PointCoords { x: f64, y: f64, z: f64 }

impl Point {
    pub fn coords(&self) -> PointCoords {
        PointCoords {
            x: self.x.value(),
            y: self.y.value(),
            z: self.z.value(),
        }
    }
}
```

### 2. **API 互換性保証**

```rust
impl Point {
    // 既存APIは100%維持
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x.clone() - other.x.clone();
        let dy = self.y.clone() - other.y.clone();
        let dz = self.z.clone() - other.z.clone();
        (dx * dx + dy * dy + dz * dz).sqrt().value()
    }

    // トレラント比較の新機能追加
    pub fn is_near(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.tolerant_eq(other, context)
    }
}
```

### 3. **演算子オーバーロード維持**

```rust
impl std::ops::Add<Vector> for Point {
    type Output = Point;

    fn add(self, vector: Vector) -> Point {
        Point::new(
            (self.x + vector.x()).value(),
            (self.y + vector.y()).value(),
            (self.z + vector.z()).value(),
        )
    }
}
```

## 🎁 期待される効果

### 数値堅牢性

- ✅ **トレラント比較**: 浮動小数点誤差に対応
- ✅ **高精度演算**: Scalar 内部での精度管理
- ✅ **単位系管理**: mm 単位での一貫性

### CAD 機能性

- ✅ **既存 API100%維持**: 破壊的変更なし
- ✅ **トレイト設計保持**: Curve3D, PointOps 等
- ✅ **分類システム保持**: geometry_kind 階層

### アーキテクチャ整理

- ✅ **geo_primitives 削除**: 重複排除
- ✅ **責務明確化**: geo_core=数学基盤, model=CAD 業務ロジック
- ✅ **保守性向上**: 単一データフロー

## 📅 実装順序

1. **Point, Vector 移植**（基礎）
2. **Circle, Line 移植**（基本図形）
3. **NurbsCurve 移植**（高次要素）
4. **トレイト実装更新**（Curve3D 等）
5. **テスト・検証**（互換性確認）
6. **geo_primitives 削除**（重複排除）

## 🚨 注意事項

### 破壊的変更の最小化

```rust
// 段階的移行パターン
#[cfg(feature = "use_scalar")]
type CoordType = Scalar;
#[cfg(not(feature = "use_scalar"))]
type CoordType = f64;

pub struct Point {
    x: CoordType,
    y: CoordType,
    z: CoordType,
}
```

この方針により、**model の洗練された CAD 設計を保持**しながら、**geo_core の数値堅牢性**を完全に活用できます。
