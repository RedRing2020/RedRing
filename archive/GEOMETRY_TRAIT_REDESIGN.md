# RedRing 幾何形状トレイト設計 - 基本・拡張境界の再定義

## ✅ Phase 1 完了: 基本トレイト設計

### 実装済み構造

#### 共通基盤トレイト (`foundation.rs`)

```rust
// データアクセス層
GeometryFoundation<T>     // 基本属性アクセス（境界ボックス含む）
BasicMetrics<T>           // データ構造に直接関連する計算
BasicContainment<T>       // 基本的な包含判定
BasicParametric<T>        // パラメトリック形状の基本操作
BasicDirectional<T>       // 方向を持つ要素の基本操作
```

#### 形状別基本トレイト (`basic_shapes.rs`)

```rust
// 点・ベクトル
PointCore<T>, Point2DCore<T>, Point3DCore<T>
VectorCore<T>, Vector2DCore<T>, Vector3DCore<T>

// 線系
InfiniteLineCore<T>       // 無限直線
LineSegmentCore<T>        // 線分

// 曲線系
CircleCore<T>             // 円（BasicMetrics実装済み）
Circle3DCore<T>           // 3D円
EllipseCore<T>            // 楕円
ArcCore<T>                // 弧

// 境界系
BoundingBoxCore<T>        // 境界ボックス
BoundingBox2DCore<T>      // 2D境界ボックス
BoundingBox3DCore<T>      // 3D境界ボックス
```

#### ヘルパー関数 (`helpers.rs`)

```rust
normalize_angle()           // 角度正規化
angle_in_range()           // 角度範囲判定
approx_equal()             // 許容誤差比較
point_distance()           // 距離計算
ellipse_perimeter_approximation() // 楕円周長近似
arc_length()               // 弧長計算
bbox_*()                   // 境界ボックス計算群
```

### 設計原則の実現

✅ **明確な責務分離**: 基本=データ+基本解析のみ
✅ **型安全性**: コンパイル時の型チェック強化
✅ **段階的実装**: 基本トレイト → 拡張トレイトの段階的追加
✅ **デフォルト実装**: CircleCore + BasicMetrics で実証
✅ **競合回避**: blanket implementation の排除

## 次の Phase: 拡張トレイト設計

### Phase 2 計画: 空間関係・変換トレイト

#### 交差・衝突検出層

```rust
trait SpatialIntersection<T, Other = Self> {
    type IntersectionResult;
    fn intersects(&self, other: &Other) -> bool;
    fn intersection(&self, other: &Other) -> Self::IntersectionResult;
}

trait CollisionDetection<T> {
    fn bounding_box_test(&self, other: &Self) -> bool;
    fn separating_axis_test(&self, other: &Self) -> bool;
}
```

#### 変換・射影層

```rust
trait GeometryTransform<T> {
    type Transformed;
    fn translate(&self, offset: &Self::Vector) -> Self::Transformed;
    fn rotate(&self, angles: (T, T, T)) -> Self::Transformed;
    fn scale(&self, factor: T) -> Self::Transformed;
}

trait GeometryProjection<T> {
    type ProjectedGeometry;
    fn project_to_plane(&self, plane: &Plane<T>) -> Self::ProjectedGeometry;
    fn project_to_line(&self, line: &Line<T>) -> Self::ProjectedGeometry;
}
```

### Phase 3 計画: 具体実装との統合

1. `geo_primitives` の既存実装を基本トレイトに適合
2. 拡張トレイトの段階的実装
3. パフォーマンステストと最適化

## 利点の実証

- ✅ **ビルド成功**: 型競合なし、コンパイル時間短縮
- ✅ **可読性向上**: 明確な責務分離による理解しやすさ
- ✅ **拡張性**: 新形状の追加が標準化されたパターンで可能
- ✅ **保守性**: ヘルパー関数の分離による再利用性向上
