# geo_core vs model: 抽象化構造設計比較分析

## 🎯 設計哲学の根本的違い

### geo_core: 統合数学ライブラリ設計

```rust
// 型レベルでの次元区別
pub struct Point2D { x: Scalar, y: Scalar }
pub struct Point3D { x: Scalar, y: Scalar, z: Scalar }
pub struct Vector2D { x: Scalar, y: Scalar }
pub struct Vector3D { x: Scalar, y: Scalar, z: Scalar }

// 統合インターフェース
pub mod primitives {
    pub use crate::primitives2d::{Point2D, LineSegment2D, ...};
    pub use crate::primitives3d::{Point3D, LineSegment3D, ...};
}
```

**特徴**:

- ✅ **型名での次元表現**: Point2D, Point3D
- ✅ **フラットな構造**: primitives2d.rs, primitives3d.rs
- ✅ **統合再エクスポート**: primitives.rs
- ✅ **数値堅牢性中心**: Scalar ベース、TolerantEq 統一

### model: CAD 業務抽象化設計

```rust
// ネームスペースでの次元区別
pub mod geometry2d {
    pub struct Point { x: f64, y: f64 }
    pub struct Vector { x: f64, y: f64 }
}

pub mod geometry3d {
    pub struct Point { x: f64, y: f64, z: f64 }
    pub struct Vector { x: f64, y: f64, z: f64 }
}

// CAD抽象化レイヤー
pub mod geometry_trait {
    pub trait Curve3D { fn evaluate(&self, t: f64) -> Point; }
    pub trait Surface { fn evaluate(&self, u: f64, v: f64) -> Point; }
}
```

**特徴**:

- ✅ **ネームスペースでの次元表現**: geometry2d::Point, geometry3d::Point
- ✅ **階層化された構造**: geometry/geometry2d/, geometry/geometry3d/
- ✅ **CAD 業務ロジック**: Curve3D, Surface トレイト
- ✅ **意味論的分類**: geometry_kind, geometry_common

## 🚨 統合の互換性問題

### 問題 1: 命名規則の衝突

```rust
// geo_core期待
use geo_core::{Point2D, Point3D};

// model現状
use crate::geometry::{geometry2d::Point, geometry3d::Point};
//                    ^^^^^^^^^^^^^^^ 同じ名前Point！
```

### 問題 2: API シグネチャの違い

```rust
// geo_core設計
impl Point3D {
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self
    pub fn x(&self) -> &Scalar  // 参照返却
}

// model設計 (現在Scalar移植後)
impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self  // f64で入力
    pub fn x(&self) -> f64  // 値返却
}
```

### 問題 3: トレイト境界の違い

```rust
// geo_core: Copy削除、TolerantEq実装
#[derive(Debug, Clone)]
pub struct Point3D { x: Scalar, y: Scalar, z: Scalar }

// model: Copy期待、PartialEq実装
#[derive(Debug, Clone, Copy, PartialEq)]  // ← Copy削除でエラー
pub struct Point { x: Scalar, y: Scalar, z: Scalar }
```

## 🎯 統合戦略の選択肢

### 選択肢 A: geo_core 完全統合

```rust
// model/src/geometry/geometry3d/point.rs
pub use geo_core::Point3D as Point;

// 課題:
// - 27個のコンパイルエラー修正が必要
// - Copy期待箇所の大幅修正
// - APIシグネチャの変更影響
```

### 選択肢 B: model 独自実装継続 + geo_core 参照

```rust
// model独自実装は維持
pub struct Point { x: Scalar, y: Scalar, z: Scalar }

impl Point {
    // geo_coreとの相互変換
    pub fn from_geo_core(p: geo_core::Point3D) -> Self { ... }
    pub fn to_geo_core(&self) -> geo_core::Point3D { ... }
}
```

### 選択肢 C: ハイブリッド統合（推奨）

```rust
// 内部でgeo_coreを使用、APIは既存維持
pub struct Point {
    inner: geo_core::Point3D,  // 内部実装
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { inner: geo_core::Point3D::from_f64(x, y, z) }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }  // API維持

    // geo_coreの数値堅牢性活用
    pub fn tolerant_eq(&self, other: &Self, ctx: &ToleranceContext) -> bool {
        self.inner.tolerant_eq(&other.inner, ctx)
    }
}
```

## 📊 推奨: 選択肢 C (ハイブリッド統合)

### 利点

- ✅ **既存 API100%維持**: 破壊的変更なし
- ✅ **geo_core の数値堅牢性活用**: 内部で Scalar 使用
- ✅ **段階的移行可能**: 既存コードそのまま動作
- ✅ **Copy/PartialEq 解決**: 適切なトレイト実装

### 実装例

```rust
use geo_core::{Point3D as GeoPoint3D, Scalar, ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]
pub struct Point {
    inner: GeoPoint3D,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { inner: GeoPoint3D::from_f64(x, y, z) }
    }

    // 既存API完全維持
    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }
    pub fn z(&self) -> f64 { self.inner.z().value() }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.inner.distance_to(&other.inner).value()
    }

    // geo_coreの新機能活用
    pub fn tolerant_eq(&self, other: &Self, ctx: &ToleranceContext) -> bool {
        self.inner.tolerant_eq(&other.inner, ctx)
    }
}

// Copy実装（軽量ラッパー）
impl Copy for Point {}

// PartialEq実装
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        // デフォルト許容誤差での比較
        let default_ctx = ToleranceContext::default();
        self.inner.tolerant_eq(&other.inner, &default_ctx)
    }
}
```

この方式により、**既存の model の設計思想を完全保持**しながら、**geo_core の数値堅牢性を 100%活用**できます。
