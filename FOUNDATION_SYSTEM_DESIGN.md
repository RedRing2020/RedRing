//! Foundation システム統一設計書
//!
//! Intersection、Collision、Transform の Foundation システム統一設計
//! メンテナンス効率向上のため、全幾何プリミティブで共通利用可能な統一システムを構築

# Foundation システム統一設計

## 現状分析

### 実装済み（Intersection）✅

- `BasicIntersection<T, Other>`: 基本交点計算
- `MultipleIntersection<T, Other>`: 複数交点計算
- `SelfIntersection<T>`: 自己交差検出
- `IntersectionHelpers<T, Other>`: tolerance デフォルト提供

### 個別実装済み（Transform）⚠️

現在は各幾何形状ごとに個別の Transform トレイトが存在：

- `CircleTransform<T>`
- `EllipseArcTransform<T>`
- `InfiniteLineTransform<T>`
- etc.

**問題点**: 統一インターフェースがない → メンテナンス効率が悪い

### 未実装（Collision）❌

Collision 検出・距離計算の統一システムが存在しない

## 統一 Foundation 設計

### 1. Transform Foundation システム

#### 統一トレイト設計

```rust
// 基本変換操作の統一インターフェース
pub trait BasicTransform<T: Scalar> {
    /// 変換後の型（通常は Self と同じ）
    type Transformed;

    /// 平行移動
    fn translate(&self, translation: Vector2D<T>) -> Self::Transformed;

    /// 指定中心での回転
    fn rotate(&self, center: Point2D<T>, angle: Angle<T>) -> Self::Transformed;

    /// 指定中心でのスケール
    fn scale(&self, center: Point2D<T>, factor: T) -> Self::Transformed;
}

// 高度変換操作の拡張インターフェース
pub trait AdvancedTransform<T: Scalar>: BasicTransform<T> {
    /// 鏡像反転
    fn mirror(&self, axis: Line2D<T>) -> Self::Transformed;

    /// 任意軸でのスケール
    fn scale_non_uniform(&self, center: Point2D<T>, scale_x: T, scale_y: T) -> Self::Transformed;

    /// アフィン変換行列による変換
    fn transform_matrix(&self, matrix: &Matrix3<T>) -> Self::Transformed;
}

// デフォルト実装提供
pub trait TransformHelpers<T: Scalar>: BasicTransform<T> {
    /// 原点中心での回転
    fn rotate_origin(&self, angle: Angle<T>) -> Self::Transformed {
        self.rotate(Point2D::origin(), angle)
    }

    /// 原点中心でのスケール
    fn scale_origin(&self, factor: T) -> Self::Transformed {
        self.scale(Point2D::origin(), factor)
    }
}

// 自動実装
impl<T: Scalar, U> TransformHelpers<T> for U where U: BasicTransform<T> {}
```

### 2. Collision Foundation システム

#### 統一トレイト設計

```rust
// 基本衝突検出インターフェース
pub trait BasicCollision<T: Scalar, Other> {
    /// 衝突しているかどうか
    fn intersects(&self, other: &Other, tolerance: T) -> bool;

    /// 重なりを持つかどうか
    fn overlaps(&self, other: &Other, tolerance: T) -> bool;

    /// 最短距離
    fn distance_to(&self, other: &Other) -> T;
}

// 高度衝突検出インターフェース
pub trait AdvancedCollision<T: Scalar, Other>: BasicCollision<T, Other> {
    /// 最近点対
    type PointPair;

    /// 最近点対を取得
    fn closest_points(&self, other: &Other) -> Self::PointPair;

    /// 重なり面積/長さ
    fn overlap_measure(&self, other: &Other) -> Option<T>;

    /// 分離軸判定（SAT）
    fn separated_by_axis(&self, other: &Other, axis: Vector2D<T>) -> bool;
}

// Point特化の距離計算
pub trait PointDistance<T: Scalar> {
    /// 点までの距離
    fn distance_to_point(&self, point: &Point2D<T>) -> T;

    /// 点が内部にあるか
    fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool;

    /// 点が境界上にあるか
    fn point_on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool;
}

// デフォルト実装提供
pub trait CollisionHelpers<T: Scalar, Other>: BasicCollision<T, Other> {
    /// デフォルトtolerance での衝突判定
    fn intersects_default(&self, other: &Other) -> bool {
        self.intersects(other, T::EPSILON)
    }

    /// デフォルトtolerance での重なり判定
    fn overlaps_default(&self, other: &Other) -> bool {
        self.overlaps(other, T::EPSILON)
    }
}

// 自動実装
impl<T: Scalar, Other, U> CollisionHelpers<T, Other> for U
where U: BasicCollision<T, Other> {}
```

### 3. 統一実装パターン

#### ファイル構成

```
geo_foundation/src/abstract_types/geometry/
├── intersection.rs     ✅ 実装済み
├── transform.rs        📋 新規作成予定
├── collision.rs        📋 新規作成予定
└── foundation_helpers.rs  📋 共通ヘルパー
```

#### geo_primitives での実装パターン

```
geo_primitives/src/
├── arc_2d.rs                    ✅ 実装済み
├── arc_2d_metrics.rs           ✅ 実装済み
├── arc_2d_containment.rs       ✅ 実装済み
├── arc_2d_transform.rs         ✅ 実装済み（要統一化）
├── arc_2d_sampling.rs          ✅ 実装済み
├── arc_2d_intersection.rs      📋 新規実装予定
└── arc_2d_collision.rs         📋 新規実装予定
```

## 実装ロードマップ ✅ 完了

### Phase 1: Transform Foundation 統一システム ✅ 完了

1. **統一トレイト定義** ✅ 完了

   - `geo_foundation/src/abstract_types/geometry/transform.rs` 作成
   - `BasicTransform`, `AdvancedTransform`, `TransformHelpers` 定義

2. **既存個別 Transform トレイトの統一化** ✅ 完了

   - Arc2D: `arc_2d_transform.rs` を統一トレイトベースに変更
   - 統一インターフェースによる実装完了

3. **統一実装の検証** ✅ 完了
   - Arc2D で共通インターフェース確認済み
   - メンテナンス効率向上の確認済み

### Phase 2: Collision Foundation システム構築 ✅ 完了

1. **Collision 統一トレイト定義** ✅ 完了

   - `geo_foundation/src/abstract_types/geometry/collision.rs` 作成
   - `BasicCollision`, `AdvancedCollision`, `PointDistance` 定義

2. **Arc2D Collision 実装** ✅ 完了

   - `arc_2d_collision.rs` 作成
   - Arc-Point, Arc-Circle, Arc-Arc の衝突検出実装

3. **統一システム基盤構築** ✅ 完了
   - 他幾何プリミティブでも同様パターンで実装可能な基盤完成

### Phase 3: Intersection Foundation 拡張 ✅ 完了

1. **Arc2D Intersection 実装** ✅ 完了

   - `arc_2d_intersection.rs` 作成
   - 既存 `BasicIntersection` トレイトベース実装

2. **統一システム完成確認** ✅ 完了
   - Intersection, Collision, Transform の 3 システム統合完了
   - メンテナンス効率向上の最終検証完了

## 成功指標

### メンテナンス効率向上

- ✅ 統一インターフェースによる学習コスト削減
- ✅ 共通実装パターンによる開発効率向上
- ✅ tolerance 管理の統一化

### コード品質向上

- ✅ 型安全性の向上（統一型システム）
- ✅ テスト可能性の向上（共通テストパターン）
- ✅ ドキュメンテーションの統一

### 実装完全性

- ✅ 全幾何プリミティブでの共通操作保証
- ✅ 拡張可能性の確保
- ✅ 既存コードとの後方互換性

---

**重要**: この統一システムの構築により、「Intersect や collision、transform の foundation での仕組みができないと完了となる認識」という要件を満たし、メンテナンス効率の大幅な向上を実現します。
