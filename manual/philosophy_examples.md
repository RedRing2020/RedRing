# 設計思想のコード例 / Philosophy Examples

`philosophy.md` で説明している方針に対応するコード例をまとめます。

## 専用エラー型の使用

```rust
// ✅ 推奨: 専用エラー型
impl Ellipse<T> {
    pub fn from_radii(rx: T, ry: T) -> Result<Self, EllipseError> {
        // 楕円固有の検証とエラー報告
    }
}

// ❌ 非推奨: 汎用エラー型
impl Ellipse<T> {
    pub fn from_radii(rx: T, ry: T) -> Result<Self, GeometryError> {
        // エラーの詳細が不明確
    }
}
```

## 統合トレイトの活用

```rust
pub trait Normalizable<T> {
    type Output;
    type Error;
    fn normalize(&self) -> Result<Self::Output, Self::Error>;
}

pub trait DistanceCalculation<T, Target> {
    fn distance_to(&self, other: &Target) -> T;
}
```

## geo_io の例外パターン

```rust
use geo_primitives::{Point3D, TriangleMesh3D, Vector3D};

pub fn load_stl<T: Scalar>(path: &Path) -> Result<TriangleMesh3D<T>, IoError> {
    // ファイル形式との直接的な変換処理
}
```

## MVVM 準拠のアクセスパターン

```rust
// ❌ View層での直接I/Oアクセス（禁止）
// use geo_io::stl;

// ✅ ViewModel経由のアクセス（推奨）
use viewmodel::stl_loader::{load_stl_mesh, StlMeshData};
```

## analysis の汎用トレイト

```rust
pub trait Scalar: Copy + Clone + PartialEq + PartialOrd + ... {}
pub trait TolerantEq<T: Scalar> { ... }
```
