# Foundation パターン実装ガイド

**最終更新日: 2025 年 11 月 11 日**

## 概要

Foundation パターンは、RedRing の幾何システムにおける統一トレイト実装方式です。`geo_core` をブリッジ役として、全ての幾何プリミティブが共通のインターフェースを持つことで、型安全性と一貫性を保証します。

## 重要な課題（2025年11月11日現在）

⚠️ **geo_nurbs Foundation パターン違反**: 現在 geo_nurbs は geo_primitives を直接インポートしており、Foundation パターンに違反しています。正しくは geo_core 経由でアクセスする必要があります。

## 基本構造

### 1. Foundation トレイトの定義

```rust
// geo_foundation/src/extension_foundation.rs
pub trait ExtensionFoundation<T: Scalar> {
    type BBox: AbstractBBox<T>;
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> Self::BBox;
    fn measure(&self) -> Option<T>;
}

// analysis/src/abstract_types/mod.rs
pub trait TolerantEq<T: Scalar> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool;
}
```

### 2. 各幾何プリミティブでの実装

各 3D 幾何プリミティブには `{shape}_3d_foundation.rs` ファイルが存在し、上記トレイトを実装します。

```rust
// 例: point_3d_foundation.rs
impl<T: Scalar> ExtensionFoundation<T> for Point3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Point
    }

    fn bounding_box(&self) -> Self::BBox {
        BBox3D::from_point(*self)
    }

    fn measure(&self) -> Option<T> {
        Some(T::ZERO) // 点の測度は0
    }
}

impl<T: Scalar> TolerantEq<T> for Point3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }
}
```

## 実装済み Foundation ファイル

### ✅ 完全実装済み（2025 年 10 月 26 日現在）

| 幾何プリミティブ   | Foundation ファイル              | 実装内容                         |
| ------------------ | -------------------------------- | -------------------------------- |
| **Arc3D**          | `arc_3d_foundation.rs`           | 弧の境界ボックス、弧長、誤差比較 |
| **BBox3D**         | `bbox_3d_foundation.rs`          | 境界ボックスの体積、包含判定     |
| **Circle3D**       | `circle_3d_foundation.rs`        | 円の境界ボックス、円周、誤差比較 |
| **Cylinder3D**     | `cylinder_3d_foundation.rs`      | 円柱の境界ボックス、表面積       |
| **Point3D**        | `point_3d_foundation.rs`         | 点の距離比較、測度 0             |
| **Ray3D**          | `ray_3d_foundation.rs`           | 無限レイ、測度無限大             |
| **Sphere3D**       | `sphere_3d_foundation.rs`        | 球の境界ボックス、表面積         |
| **Triangle3D**     | `triangle_3d_foundation.rs`      | 三角形の境界ボックス、面積       |
| **TriangleMesh3D** | `triangle_mesh_3d_foundation.rs` | メッシュの境界ボックス、総面積   |
| **Vector3D**       | `vector_3d_foundation.rs`        | ベクトルの長さ、誤差比較         |

### 📋 今後の実装予定（2D 系）

| 幾何プリミティブ | Foundation ファイル          | 優先度 |
| ---------------- | ---------------------------- | ------ |
| **Point2D**      | `point_2d_foundation.rs`     | 高     |
| **Vector2D**     | `vector_2d_foundation.rs`    | 高     |
| **Direction2D**  | `direction_2d_foundation.rs` | 高     |
| **Ray2D**        | `ray_2d_foundation.rs`       | 高     |
| **Circle2D**     | `circle_2d_foundation.rs`    | 中     |
| **Arc2D**        | `arc_2d_foundation.rs`       | 中     |
| **Ellipse2D**    | `ellipse_2d_foundation.rs`   | 低     |

## PrimitiveKind 列挙型

```rust
pub enum PrimitiveKind {
    // 基本要素
    Point,
    Vector,
    Direction,
    Ray,

    // 曲線
    Line,
    Circle,
    Ellipse,
    Arc,

    // 曲面
    Plane,
    Sphere,
    Cylinder,

    // 複合形状
    Triangle,
    Mesh,
    BBox,
}
```

## 実装パターン

### 1. measure() メソッドの実装指針

| 形状タイプ   | 測度の意味 | 実装例                |
| ------------ | ---------- | --------------------- |
| **点**       | 測度 0     | `T::ZERO`             |
| **曲線**     | 長さ       | `self.length()`       |
| **曲面**     | 面積       | `self.area()`         |
| **立体**     | 体積       | `self.volume()`       |
| **無限形状** | 無限大     | `None` または特別処理 |

### 2. bounding_box() メソッドの実装指針

| 形状タイプ   | 境界ボックス     | 実装例                 |
| ------------ | ---------------- | ---------------------- |
| **有限形状** | 最小包含直方体   | `BBox3D::from_...`     |
| **無限形状** | 微小境界ボックス | `原点周辺の微小BBox3D` |
| **退化形状** | 特別処理         | 条件分岐で適切に処理   |

### 3. tolerant_eq() メソッドの実装指針

```rust
impl<T: Scalar> TolerantEq<T> for GeometryType<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 距離ベースの比較（推奨）
        self.distance_to(other) <= tolerance

        // または成分ベースの比較
        // (self.x() - other.x()).abs() <= tolerance &&
        // (self.y() - other.y()).abs() <= tolerance &&
        // (self.z() - other.z()).abs() <= tolerance
    }
}
```

## テストパターン

各 Foundation ファイルには対応するテストを実装します：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::ExtensionFoundation;
    use analysis::TolerantEq;

    #[test]
    fn test_extension_foundation() {
        let geometry = GeometryType::new(/* パラメータ */);

        // primitive_kind のテスト
        assert_eq!(geometry.primitive_kind(), PrimitiveKind::ExpectedKind);

        // bounding_box のテスト
        let bbox = geometry.bounding_box();
        assert!(bbox.is_some()); // または is_none() for infinite shapes

        // measure のテスト
        let measure = geometry.measure();
        assert!(measure.is_some());
        assert!(measure.unwrap() >= 0.0); // 負の測度はない
    }

    #[test]
    fn test_tolerant_eq() {
        let geometry1 = GeometryType::new(/* パラメータ1 */);
        let geometry2 = GeometryType::new(/* パラメータ2 */);
        let tolerance = 0.01;

        // 自己比較
        assert!(geometry1.tolerant_eq(&geometry1, tolerance));

        // 対称性
        assert_eq!(
            geometry1.tolerant_eq(&geometry2, tolerance),
            geometry2.tolerant_eq(&geometry1, tolerance)
        );
    }
}
```

## 依存関係

### Foundation ファイルの依存関係

```rust
use crate::{GeometryType, BBox3D}; // 同クレート内の型
use geo_foundation::{ExtensionFoundation, PrimitiveKind}; // Foundation トレイト
use analysis::{TolerantEq, Scalar}; // 抽象型トレイト
```

### Cargo.toml での依存関係

```toml
[dependencies]
geo_foundation = { path = "../geo_foundation" }
analysis = { path = "../../analysis" }
```

## ファイル配置

```
model/
├── geo_foundation/          # Foundation トレイト定義
│   ├── src/
│   │   ├── extension_foundation.rs
│   │   └── core_foundation.rs
│   └── Cargo.toml
├── geo_primitives/          # 具体実装
│   ├── src/
│   │   ├── point_3d.rs                 # Core実装
│   │   ├── point_3d_foundation.rs      # Foundation実装 ← New!
│   │   ├── point_3d_tests.rs          # テスト
│   │   └── ...
│   └── Cargo.toml
└── analysis/                # 抽象型定義
    ├── src/
    │   └── abstract_types/
    │       └── mod.rs          # TolerantEq定義
    └── Cargo.toml
```

## 品質基準

### 1. ファイルサイズ

- Foundation ファイル: **50-100 行** を目標
- 簡潔で集中した実装を心がける

### 2. テストカバレッジ

- 各トレイトメソッドの基本動作テスト
- 境界条件（ゼロ、無限大、退化ケース）のテスト
- f32/f64 両対応のテスト

### 3. 一貫性

- 全ての Foundation ファイルで同じパターンを使用
- コメントとドキュメンテーションの統一
- エラーハンドリングの統一

## 今後の発展

### 1. 2D Foundation 対応

2D 幾何プリミティブにも同様の Foundation パターンを適用予定。

### 2. 高次元対応

将来的に 4D+ の幾何プリミティブでも同じパターンを適用可能。

### 3. 専門トレイトの追加

より専門的な幾何操作のための追加トレイトを Foundation に統合予定。

---

**最終更新**: 2025 年 10 月 26 日
**文書バージョン**: 1.0
**対応システム**: RedRing Foundation v1.0
