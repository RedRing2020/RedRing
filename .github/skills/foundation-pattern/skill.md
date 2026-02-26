# Foundation パターン

## 最終更新日: 2026年2月13日

RedRingにおける Foundation Pattern の詳細仕様と実装ガイドを定義します。

---

## Foundation Pattern とは

全ての幾何プリミティブに統一されたインターフェースを提供する3層構造のデザインパターンです。

```
Core Traits (Constructor/Properties/Measure)
    ↓
Extension Traits (Bounded, Transformable, etc.)
    ↓
Transform Traits (AnalysisTransform2D/3D)
```

---

## 1. Core Traits（geo_foundation/src/core/）

### 定義場所

`geo_foundation/src/core/{shape}_core_traits.rs`

### 3つの基本トレイト

#### Constructor - 生成機能

```rust
pub trait {Shape}3DConstructor<T: Scalar> {
    /// 基本的なコンストラクタ
    fn new(...) -> Option<Self> where Self: Sized;
    
    /// 代替コンストラクタ
    fn from_point_direction(...) -> Option<Self> where Self: Sized;
    
    /// 単位形状生成
    fn unit_x() -> Self where Self: Sized;
}
```

#### Properties - プロパティ取得

```rust
pub trait {Shape}3DProperties<T: Scalar> {
    /// 基本情報取得
    fn start(&self) -> (T, T, T);
    fn end(&self) -> (T, T, T);
    fn length(&self) -> T;
    
    /// 形状の次元数
    fn dimension(&self) -> u32;
}
```

#### Measure - 計量・関係演算

```rust
pub trait {Shape}3DMeasure<T: Scalar> {
    /// 測度（長さ、面積、体積）
    fn measure(&self) -> T;
    
    /// 距離計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    
    /// 包含判定
    fn contains_point(&self, point: (T, T, T)) -> bool;
    
    /// パラメトリック評価
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}
```

### Core Traits の統合

```rust
/// 3つのCore機能統合トレイト
pub trait {Shape}3DCore<T: Scalar>:
    {Shape}3DConstructor<T> +
    {Shape}3DProperties<T> +
    {Shape}3DMeasure<T>
{
}

// Blanket implementation
impl<T: Scalar, S> {Shape}3DCore<T> for S
where
    S: {Shape}3DConstructor<T> +
       {Shape}3DProperties<T> +
       {Shape}3DMeasure<T>
{
}
```

---

## 2. Extension Traits（geo_foundation/src/extension_foundation.rs）

### ExtensionFoundation - 基本拡張

```rust
pub trait ExtensionFoundation<T: Scalar> {
    /// プリミティブの種類
    fn primitive_kind(&self) -> PrimitiveKind;
    
    /// 測度（Option版、無限形状対応）
    fn measure(&self) -> Option<T>;
}
```

### Bounded - 境界ボックス

```rust
pub trait Bounded<T: Scalar>: ExtensionFoundation<T> {
    type Aabb;
    
    /// 軸並行境界ボックス取得
    fn aabb(&self) -> Option<Self::Aabb>;
}
```

### その他の Extension Traits

```rust
pub trait MeasurableExtension<T: Scalar>: ExtensionFoundation<T> {
    fn area(&self) -> Option<T>;
    fn volume(&self) -> Option<T>;
}

pub trait SpatialExtension<T: Scalar>: ExtensionFoundation<T> {
    fn centroid(&self) -> Option<(T, T, T)>;
}
```

---

## 3. Transform Traits（geo_foundation/src/core/transform.rs）

### AnalysisTransform3D

```rust
pub trait AnalysisTransform3D<T: Scalar> {
    type Matrix4x4;
    type Angle;
    type Output;

    /// 平行移動
    fn translate_analysis(
        &self,
        offset: (T, T, T),
    ) -> Result<Self::Output, TransformError>;

    /// 回転（任意軸）
    fn rotate_analysis(
        &self,
        axis: (T, T, T),
        angle: Self::Angle,
    ) -> Result<Self::Output, TransformError>;

    /// スケール変換
    fn scale_analysis(
        &self,
        factors: (T, T, T),
    ) -> Result<Self::Output, TransformError>;

    /// 行列変換
    fn transform_point_matrix(
        &self,
        matrix: Self::Matrix4x4,
    ) -> Self::Output;
}
```

### TransformError - エラー型

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    ZeroVector,           // ゼロベクトル
    InvalidScaleFactor,   // 無効なスケール倍率
    InvalidRotation,      // 無効な回転パラメータ
    InvalidGeometry,      // 変換後の幾何的無効性
}
```

---

## 実装チェックリスト

新規幾何プリミティブ実装時の必要ファイル：

### geo_foundation（トレイト定義）

- [ ] `geo_foundation/src/core/{shape}_core_traits.rs`
  - [ ] `{Shape}Constructor<T>` trait
  - [ ] `{Shape}Properties<T>` trait
  - [ ] `{Shape}Measure<T>` trait
  - [ ] `{Shape}Core<T>` trait（統合）
  - [ ] Blanket implementation

- [ ] `geo_foundation/src/lib.rs` に再エクスポート追加
  ```rust
  pub use core::{shape}_traits::{
      {Shape}Constructor, {Shape}Properties,
      {Shape}Measure, {Shape}Core,
  };
  ```

### geo_primitives（具体実装）

- [ ] `geo_primitives/src/{shape}_3d.rs`（基本実装）
  - [ ] 構造体定義
  - [ ] 基本メソッド実装

- [ ] `geo_primitives/src/{shape}_3d_foundation.rs`
  - [ ] Core Traits 実装
  - [ ] Extension Traits 実装

- [ ] `geo_primitives/src/{shape}_3d_transform.rs`
  - [ ] AnalysisTransform3D 実装

- [ ] `geo_primitives/src/{shape}_3d_tests.rs`
  - [ ] Core 機能テスト
  - [ ] Extension 機能テスト
  - [ ] Transform 機能テスト

---

## デフォルト実装パターン

### トレイトにデフォルト実装を提供

```rust
// geo_foundation/src/core/linesegment_traits.rs
pub trait LineSegment3DCollisionDetection<T: Scalar>: 
    LineSegment3DProperties<T> 
{
    fn distance_to_aabb(
        &self,
        aabb_min: (T, T, T),
        aabb_max: (T, T, T),
    ) -> T {
        // デフォルト実装
        let start = self.start();
        let end = self.end();
        geo_commons::metrics::distance::line_segment_to_aabb_distance(
            start, end, aabb_min, aabb_max,
        )
    }
}
```

### Blanket Implementation

```rust
// geo_primitives/src/line_segment_3d_collision.rs
impl<T: Scalar> LineSegment3DCollisionDetection<T> for LineSegment3D<T> {}
```

この方法により、トレイト境界を満たす全ての型に自動的に実装が適用されます。

---

## 実装例：LineSegment3D

### 1. Core Traits 定義

```rust
// geo_foundation/src/core/linesegment_traits.rs
pub trait LineSegment3DConstructor<T: Scalar> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self>
    where Self: Sized;
}

pub trait LineSegment3DProperties<T: Scalar> {
    fn start(&self) -> (T, T, T);
    fn end(&self) -> (T, T, T);
    fn length(&self) -> T;
}

pub trait LineSegment3DMeasure<T: Scalar> {
    fn measure(&self) -> T;
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}
```

### 2. 具体実装

```rust
// geo_primitives/src/line_segment_3d_foundation.rs
impl<T: Scalar> LineSegment3DConstructor<T> for LineSegment3D<T> {
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self> {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let end_point = Point3D::new(end.0, end.1, end.2);
        Self::new(start_point, end_point)
    }
}

impl<T: Scalar> LineSegment3DProperties<T> for LineSegment3D<T> {
    fn start(&self) -> (T, T, T) {
        (self.start.x(), self.start.y(), self.start.z())
    }
    
    fn end(&self) -> (T, T, T) {
        (self.end.x(), self.end.y(), self.end.z())
    }
    
    fn length(&self) -> T {
        self.start.distance_to(&self.end)
    }
}
```

---

## 参照文書

- **Phase 1 計画**: `dev/foundation/PHASE1_DETAILED_PLAN.md`
- **Foundation 再設計**: `dev/foundation/FOUNDATION_CORE_EXTENSION_REDESIGN_PROPOSAL.md`
- **NURBS Foundation**: `dev/architecture/NURBS_FOUNDATION_PATTERN.md`
- **Issue #222 実装例**: PR #223（LineSegment3DCollisionDetection）

---

## 使用例コメントの配置方針（共通）

- Foundation 実装ファイルでは、責務説明を優先し、詳細な使用例は `manual/` に集約する
- import を含む実践サンプルはコメントで長文化せず、`manual/<topic>_examples.md` 参照へ置換する
- 参照先ドキュメントを追加した場合は `manual/SUMMARY.md` に登録する
- 詳細ルールは `.github/skills/implementation/skill.md` の
    「使用例コメントの配置ルール（CI整合）」に従う
