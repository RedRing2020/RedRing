# 型分類 / Type Classification System

RedRing では、幾何学要素の分類を明示的な型システムで定義することで、型安全性と構造的明快さを両立しています。

## PrimitiveKind

基本幾何プリミティブの分類：

- `Point`：点
- `Vector`：ベクトル  
- `Direction`：正規化されたベクトル
- `Line`：直線
- `Ray`：光線

## CurveType

曲線要素の分類：

- `Line`：直線
- `Circle`：円
- `Ellipse`：楕円
- `Arc`：円弧
- `Spline`：スプライン（NURBS 対応予定）

## エラー型システム

各幾何要素は専用のエラー型を持ちます：

- `EllipseError`：楕円固有のエラー
- `CircleError`：円固有のエラー
- `NormalizationError`：正規化エラー
- `EllipseArcError`：楕円弧エラー

## トレイト統合システム

重複する操作は統合トレイトで抽象化：

### `Normalizable<T>`
```rust
pub trait Normalizable<T> {
    type Output;
    type Error;
    fn normalize(&self) -> Result<Self::Output, Self::Error>;
}
```

### `DistanceCalculation<T, Target>`
```rust
pub trait DistanceCalculation<T, Target> {
    fn distance_to(&self, other: &Target) -> T;
}
```

## 設計意図

- **型安全性**：コンパイル時の型チェックによるエラー防止
- **専用性**：各幾何要素に特化したエラー情報の提供
- **拡張性**：新しい幾何要素の追加に対応可能な構造
- **明示性**：API の意図と制約を型で表現
