# 型分類 / CurveKind & SurfaceKind

RedRing では、幾何学要素の分類を明示的な enum 型で定義することで、型安全性と構造的明快さを両立しています。

## CurveKind

- `Line`：直線
- `Circle`：円
- `Ellipse`：楕円
- `Spline`：スプライン（将来的に NURBS 対応予定）

## SurfaceKind

- `Plane`：平面
- `Sphere`：球面
- `Torus`：トーラス
- `Ellipsoid`：楕円体
- `Cylinder`：円柱
- `Cone`：円錐

## 設計意図

- 型分類は API の明示性と拡張性を高めるために導入
- `SurfaceKind` は `Surface` トレイトと連携し、動的な型判定にも対応
