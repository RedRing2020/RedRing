# RedRing 命名規則統一ガイドライン

## 1. 基本原則

### 1.1 トレイト命名

- **基本トレイト**: `{Shape}Core<T>` （例: `PointCore<T>`, `CircleCore<T>`）
- **次元特化トレイト**: `{Shape}{2D|3D}Core<T>` （例: `Point2DCore<T>`, `Circle3DCore<T>`）
- **高度トレイト**: `{Shape}{Function}` （例: `CircleIntersection<T>`, `PointTransform<T>`）

### 1.2 型名統一

- **境界ボックス**: `BBox` （`BoundingBox`は使用しない）
- **円弧**: `Arc` （円弧の一般的な名称）
- **楕円弧**: `EllipseArc` （楕円の弧部分）
- **方向**: `Direction` （向きを持つ正規化ベクトル）
- **光線**: `Ray` （原点と方向を持つ半無限直線）
- **線分**: `Line` （InfiniteLine を基盤とし、パラメータ範囲で有効範囲を表現）

### 1.3 2D/3D 次元サフィックス規則

- **トレイト定義**: 次元サフィックスを付与 (`Point2DCore<T>`, `Point3DCore<T>`)
- **具象型実装**: 次元サフィックスを付与しない (`Point<T>` を 2D/3D で共用)
- **型エイリアス**: 互換性のため次元サフィックス付きエイリアスを提供

## 2. 形状別命名規則

### 2.1 基本形状

```rust
// 点 (Point)
trait PointCore<T: Scalar>           // 基本点トレイト
trait Point2DCore<T: Scalar>         // 2D点トレイト
trait Point3DCore<T: Scalar>         // 3D点トレイト

// ベクトル (Vector)
trait VectorCore<T: Scalar>          // 基本ベクトルトレイト
trait Vector2DCore<T: Scalar>        // 2Dベクトルトレイト
trait Vector3DCore<T: Scalar>        // 3Dベクトルトレイト

// 方向 (Direction) - 正規化ベクトル
trait DirectionCore<T: Scalar>       // 基本方向トレイト
trait Direction2DCore<T: Scalar>     // 2D方向トレイト
trait Direction3DCore<T: Scalar>     // 3D方向トレイト
```

### 2.2 曲線形状

```rust
// 円 (Circle)
trait CircleCore<T: Scalar>          // 基本円トレイト
trait Circle3DCore<T: Scalar>        // 3D円トレイト（平面円）

// 楕円 (Ellipse)
trait EllipseCore<T: Scalar>         // 基本楕円トレイト
trait Ellipse3DCore<T: Scalar>       // 3D楕円トレイト

// 円弧 (Arc)
trait ArcCore<T: Scalar>             // 基本円弧トレイト
trait Arc3DCore<T: Scalar>           // 3D円弧トレイト

// 楕円弧 (EllipseArc)
trait EllipseArcCore<T: Scalar>      // 基本楕円弧トレイト
trait EllipseArc3DCore<T: Scalar>    // 3D楕円弧トレイト
```

### 2.3 直線形状

```rust
// 無限直線 (InfiniteLine) - 基盤：無限に延びる直線
trait InfiniteLineCore<T: Scalar>    // 基本無限直線トレイト
trait InfiniteLine2DCore<T: Scalar>  // 2D無限直線トレイト
trait InfiniteLine3DCore<T: Scalar>  // 3D無限直線トレイト

// 光線 (Ray) - 半無限直線（始点あり、一方向に無限）
trait RayCore<T: Scalar>             // 基本光線トレイト
trait Ray2DCore<T: Scalar>           // 2D光線トレイト
trait Ray3DCore<T: Scalar>           // 3D光線トレイト

// 線分 (Line) - InfiniteLineを基盤とし、パラメータ範囲で有効範囲を表現
trait LineCore<T: Scalar>            // 基本線分トレイト
trait Line2DCore<T: Scalar>          // 2D線分トレイト
trait Line3DCore<T: Scalar>          // 3D線分トレイト
```

### 2.4 面・立体形状

```rust
// 境界ボックス (BBox) - BoundingBoxは使用しない
trait BBoxCore<T: Scalar>            // 基本境界ボックストレイト
trait BBox2DCore<T: Scalar>          // 2D境界ボックストレイト
trait BBox3DCore<T: Scalar>          // 3D境界ボックストレイト

// 球 (Sphere)
trait SphereCore<T: Scalar>          // 基本球トレイト
```

## 3. 実装型の命名規則

### 3.1 具象型名（geo_primitives）

- **基本原則**: 次元サフィックスを付与しない
- **ジェネリック**: `Point<T>`, `Circle<T>`, `Arc<T>`, `Line<T>`
- **特化型エイリアス**: 互換性のため提供

```rust
// geo_primitives/src/geometry2d/point.rs
pub struct Point<T: Scalar> { ... }
pub type Point2D<T> = Point<T>;     // 互換性エイリアス
pub type Point2DF64 = Point<f64>;   // 特化型エイリアス

// geo_primitives/src/geometry3d/point.rs
pub struct Point<T: Scalar> { ... }
pub type Point3D<T> = Point<T>;     // 互換性エイリアス
pub type Point3DF64 = Point<f64>;   // 特化型エイリアス

// geo_primitives/src/geometry2d/arc.rs
pub struct Arc<T: Scalar> { ... }
pub type Arc2D<T> = Arc<T>;         // 互換性エイリアス

// geo_primitives/src/geometry2d/line.rs
pub struct Line<T: Scalar> { ... }
pub type Line2D<T> = Line<T>;       // 互換性エイリアス
```

### 3.2 エラー型命名

````rust
```rust
// エラー型は明確な接尾辞を使用
VectorNormalizationError             // ベクトル正規化エラー
CircleConstructionError              // 円構築エラー
EllipseDeformationError              // 楕円変形エラー
ArcAngleError                        // 円弧角度エラー
LineValidationError                  // 線分検証エラー
````

## 4. 親子関係の明確化

### 4.1 Circle → Arc 関係

- **Circle**: 完全な円（360 度）
- **Arc**: 円の一部（角度範囲指定）
- **実装**: `ArcCore<T>` は `CircleCore<T>` を継承または包含

```rust
trait ArcCore<T: Scalar>: CircleCore<T> {
    fn start_angle(&self) -> Angle<T>;
    fn end_angle(&self) -> Angle<T>;
    fn is_full_circle(&self) -> bool {
        self.end_angle() - self.start_angle() >= Angle::full_rotation()
    }
}
```

### 4.2 Ellipse → EllipseArc 関係

- **Ellipse**: 完全な楕円
- **EllipseArc**: 楕円の一部（角度範囲指定）

### 4.3 InfiniteLine → Line 関係（RedRing 設計方針）

- **InfiniteLine**: 無限に延びる直線（基盤）
- **Line**: InfiniteLine を基盤とし、開始・終了パラメータで有効範囲を表現
- **設計理由**: トリムや移動で線分が傾くことを回避する重要な設計方針
- **実装**: Line は InfiniteLine への参照と 2 つのパラメータ値を持つ

````

## 4. 親子関係の明確化

### 4.1 Circle → Arc 関係

- **Circle**: 完全な円（360 度）
- **Arc**: 円の一部（角度範囲指定）
- **実装**: `ArcCore<T>` は `CircleCore<T>` を継承または包含

```rust
trait ArcCore<T: Scalar>: CircleCore<T> {
    fn start_angle(&self) -> Angle<T>;
    fn end_angle(&self) -> Angle<T>;
    fn is_full_circle(&self) -> bool {
        self.end_angle() - self.start_angle() >= Angle::full_rotation()
    }
}
````

### 4.2 Ellipse → EllipseArc 関係

- **Ellipse**: 完全な楕円
- **EllipseArc**: 楕円の一部（角度範囲指定）

## 5. 移行戦略

### 5.1 段階的移行

1. **Phase 1**: 新しい命名規則でトレイト定義を統一
2. **Phase 2**: geo_primitives の具象型を新規則に適合
3. **Phase 3**: 旧名前との互換性エイリアス提供
4. **Phase 4**: 旧名前の段階的削除

### 5.2 破壊的変更の最小化

- 型エイリアスで互換性維持
- 段階的な警告と deprecation
- 明確な移行ガイド提供

## 6. 品質保証

### 6.1 命名チェックリスト

- [ ] トレイト名に`Core`サフィックス付与
- [ ] 次元特化に`2D`/`3D`サフィックス付与
- [ ] 境界ボックスは`BBox`を使用
- [ ] 円弧は`Arc`を使用（`Circle`の子）
- [ ] 線分は`Line`を使用（InfiniteLine を基盤とする RedRing 設計）
- [ ] Line 設計: InfiniteLine + パラメータ範囲による有効範囲表現
- [ ] エラー型に明確なサフィックス付与

### 6.2 コンパイル時検証

- 命名規則違反の自動検出
- トレイト継承関係の検証
- 型エイリアスの一貫性チェック

## 7. ドキュメント要件

### 7.1 トレイト文書化

- 各トレイトの責務を明確に記述
- 親子関係と継承構造を図示
- 使用例とベストプラクティス提供

### 7.2 移行ガイド

- 旧名前から新名前への対応表
- 段階的な移行手順
- 互換性維持期間の明示

---

この命名規則に従うことで、geo_foundation と geo_primitives の間での名前衝突を回避し、
一貫性のある保守しやすいコードベースを構築できます。
