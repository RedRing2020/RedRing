# Foundation パターン実装ガイド

**最終更新日: 2025年11月11日**

## 概要

Foundation パターンは、RedRing の幾何システムにおける統一トレイト実装方式です。全ての幾何プリミティブが共通のインターフェースを持つことで、型安全性と一貫性を保証します。

## 重要な課題（2025年11月11日現在）

⚠️ **geo_nurbs Foundation パターン違反**: 現在 geo_nurbs は geo_primitives を直接インポートしており、Foundation パターンに違反しています。正しくは geo_core 経由でアクセスする必要があります。

## Foundation トレイトの実装

### 1. ExtensionFoundation トレイト

```rust
// geo_foundation/src/extension_foundation.rs
pub trait ExtensionFoundation<T: Scalar> {
    type BBox: AbstractBBox<T>;
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> Self::BBox;
    fn measure(&self) -> Option<T>;
}
```

### 2. 実装例

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

## 実装指針

### 1. measure() メソッドの実装

| 形状タイプ   | 測度の意味 | 実装例                |
| ------------ | ---------- | --------------------- |
| **点**       | 測度 0     | `T::ZERO`             |
| **曲線**     | 長さ       | `self.length()`       |
| **曲面**     | 面積       | `self.area()`         |
| **立体**     | 体積       | `self.volume()`       |
| **無限形状** | 無限大     | `None` または特別処理 |

### 2. bounding_box() メソッドの実装

| 形状タイプ   | 境界ボックス     | 実装例                 |
| ------------ | ---------------- | ---------------------- |
| **有限形状** | 最小包含直方体   | `BBox3D::from_...`     |
| **無限形状** | 微小境界ボックス | `原点周辺の微小BBox3D` |
| **退化形状** | 特別処理         | 条件分岐で適切に処理   |

### 3. tolerant_eq() メソッドの実装

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
        assert!(measure.unwrap() >= T::ZERO); // 負の測度はない
    }

    #[test]
    fn test_tolerant_eq() {
        let geometry1 = GeometryType::new(/* パラメータ1 */);
        let geometry2 = GeometryType::new(/* パラメータ2 */);
        let tolerance = T::from(0.01).unwrap();

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
analysis = { path = "../../foundation/analysis" }
```

---

**このガイドの目的**: Foundation パターンを実装する開発者向けの技術的指針を提供