# NURBS-Primitives Collision/Intersection アーキテクチャ検討

**作成日**: 2025年12月23日  
**目的**: NURBS と Primitives 間の collision/intersection 実装の実現性検証

## 📊 現状分析

### 依存関係の制約

```text
現在のアーキテクチャ（✅ 正常）:
analysis → geo_foundation → geo_commons
                ↓              ↓
            geo_core ──────────┘
                ↓        ↓
        geo_primitives  geo_nurbs
                ↓           ↓
          geo_algorithms  geo_io
```

**重要な制約**:
- ✅ `geo_nurbs` は `geo_primitives` に依存しない（Foundation パターン違反解消済み）
- ✅ 両者は `geo_core`, `geo_foundation`, `analysis` のみに依存
- ❌ `geo_primitives` → `geo_nurbs` の依存は**絶対禁止**（循環依存になる）
- ❌ `geo_nurbs` → `geo_primitives` の依存も**禁止**（Foundation パターン違反）

### 現在の BasicCollision/BasicIntersection トレイト

```rust
// geo_foundation/src/extensions/collision.rs
pub trait BasicCollision<T: Scalar, Other> {
    type Point2D;
    fn intersects(&self, other: &Other, tolerance: T) -> bool;
    fn overlaps(&self, other: &Other, tolerance: T) -> bool;
    fn distance_to(&self, other: &Other) -> T;
}

// geo_foundation/src/extensions/intersection.rs
pub trait BasicIntersection<T: Scalar, Other> {
    type Point;
    fn intersection_with(&self, other: &Other, tolerance: T) -> Option<Self::Point>;
}
```

**現在の実装パターン（geo_primitives）**:
```rust
// 各 primitive 形状ごとに実装
impl<T: Scalar> BasicCollision<T, Point3D<T>> for Circle3D<T> { ... }
impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for Circle3D<T> { ... }
impl<T: Scalar> BasicCollision<T, NurbsCurve3D<T>> for Circle3D<T> { ... } // ← これができない！
```

## 🚨 重大な問題点

### 問題1: 型の相互参照不可能

**シナリオ**: `Circle3D` と `NurbsCurve3D` の衝突判定

```rust
// ❌ geo_primitives 内では NurbsCurve3D を参照できない
impl<T: Scalar> BasicCollision<T, NurbsCurve3D<T>> for Circle3D<T> {
    // コンパイルエラー: NurbsCurve3D is not in scope
}

// ❌ geo_nurbs 内では Circle3D を参照できない（Foundation パターン違反）
impl<T: Scalar> BasicCollision<T, Circle3D<T>> for NurbsCurve3D<T> {
    // アーキテクチャ違反
}
```

### 問題2: トレイトの対称性問題

`BasicCollision<T, Other>` は非対称的：
- `A.intersects(&B)` と `B.intersects(&A)` は別の実装
- 両方向の実装が必要だが、クレート間では実現不可能

### 問題3: 動的ディスパッチの限界

```rust
// ❌ トレイトオブジェクトでは Other が決定できない
fn collides(a: &dyn ???, b: &dyn ???) -> bool {
    // どのトレイトを使うべきか決定できない
}
```

## 💡 解決策の検討

### 案1: geo_algorithms クレートに実装（✅ 推奨）

**現状確認**:
```toml
# model/geo_algorithms/Cargo.toml（既存）
[dependencies]
analysis = { path = "../../foundation/analysis" }
geo_foundation = { path = "../geo_foundation" }
geo_primitives = { path = "../geo_primitives" }  # ← 既に依存
```

**追加が必要な依存関係**:
```toml
geo_nurbs = { path = "../geo_nurbs" }  # ← これを追加するだけ
```

**実装方法**:
```rust
// model/geo_algorithms/src/collision.rs (新規)
use geo_primitives::Circle3D;
use geo_nurbs::NurbsCurve3D;
use geo_foundation::{BasicCollision, Scalar};

impl<T: Scalar> BasicCollision<T, NurbsCurve3D<T>> for Circle3D<T> {
    fn intersects(&self, other: &NurbsCurve3D<T>, tolerance: T) -> bool {
        // 実装
    }
}

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for NurbsCurve3D<T> {
    fn intersects(&self, other: &Circle3D<T>, tolerance: T) -> bool {
        // 実装（対称性のため、上記と同じロジック）
    }
}
```

**利点**:
- ✅ 新規クレート不要（既存の geo_algorithms を活用）
- ✅ 責務が一致（高レベル幾何アルゴリズム）
- ✅ Foundation パターン遵守
- ✅ 全ての組み合わせを一箇所で管理
- ✅ ビルド時間増加なし

**欠点**:
- なし（最適解）

### 案2: 中間クレート作成（geo_collision）

```text
analysis → geo_foundation
                ↓
           geo_core
            ↓    ↓
   geo_primitives  geo_nurbs
            ↓         ↓
        geo_collision (新規)
                ↓
        geo_algorithms
```

**利点**:
- ✅ 責務分離が明確

**欠点**:
- ❌ 新しいクレートが必要（過剰設計）
- ❌ geo_algorithms と責務が重複
- ❌ ビルド時間増加

### 案3: 動的ディスパッチ + Visitor パターン

```rust
// geo_foundation にヘルパートレイト追加
pub trait CollisionVisitor<T: Scalar> {
    fn visit_point(&self, point: &Point3D<T>, tolerance: T) -> bool;
    fn visit_line_segment(&self, seg: &LineSegment3D<T>, tolerance: T) -> bool;
    fn visit_nurbs_curve(&self, curve: &dyn NurbsCurveCollision<T>, tolerance: T) -> bool;
    // ... 他の形状
}

pub trait CollisionAcceptor<T: Scalar> {
    fn accept(&self, visitor: &dyn CollisionVisitor<T>, tolerance: T) -> bool;
}

// geo_primitives での実装
impl<T: Scalar> CollisionAcceptor<T> for Circle3D<T> {
    f4 accept(&self, visitor: &dyn CollisionVisitor<T>, tolerance: T) -> bool {
        visitor.visit_circle(self, tolerance)
    }
}

// geo_nurbs での実装
impl<T: Scalar> CollisionAcceptor<T> for NurbsCurve3D<T> {
    fn accept(&self, visitor: &dyn CollisionVisitor<T>, tolerance: T) -> bool {
        visitor.visit_nurbs_curve(self, tolerance)
    }
}
```

**利点**:
- ✅ 依存関係変更不要
- ✅ 動的な形状の組み合わせに対応

**欠点**:
- ❌ 複雑な実装
- ❌ パフォーマンスオーバーヘッド（vtable 経由の呼び出し）
- ❌ 全ての形状を事前に Visitor に定義する必要がある

### 案3: enum による形状の統合（簡易版）

```rust
// geo_core または geo_foundation に定義
pub enum AnyGeometry<T: Scalar> {
    Point3D(Point3D<T>),
    Circle3D(Circle3D<T>),
    NurbsCurve3D(Box<NurbsCurve3D<T>>),
    // ... 他の形状
}

impl<T: Scalar> AnyGeometry<T> {
    pub fn collides_with(&self, other: &AnyGeometry<T>, tolerance: T) -> bool {
        match (self, other) {
            (AnyGeometry::Circle3D(c), AnyGeometry::Point3D(p)) => c.intersects(p, tolerance),
            (AnyGeometry::Circle3D(c), AnyGeometry::NurbsCurve3D(n)) => {
                // 専用実装
            }
            // ... 全ての組み合わせ
        }
    }
}
```

**利点**:
- ✅ シンプルな API
- ✅ 型安全

**欠点**:
- ❌ enum のサイズが大きくなる
- ❌ 新しい形状を追加するたびに enum を更新
- ❌ Foundation パターンの美しさを損なう

### 案5: マクロベースの実装生成

```rust
// geo_foundation でマクロ定義
#[macro_export]
macro_rules! impl_cross_crate_collision {
    ($shape_a:ty, $shape_b:ty) => {
        impl<T: Scalar> BasicCollision<T, $shape_b> for $shape_a {
            fn intersects(&self, other: &$shape_b, tolerance: T) -> bool {
                // デフォルト実装または専用実装
                self.distance_to(other) <= tolerance
            }
            // ...
        }
    };
}

// geo_collision や geo_algorithms で使用
impl_cross_crate_collision!(Circle3D<T>, NurbsCurve3D<T>);
```

**利点**:
- ✅ ボイラープレート削減
- ✅ 一貫性のある実装

**欠点**:
- ❌ 依然として中間クレートが必要
- ❌ マクロのデバッグが困難

## 🎯 推奨アーキテクチャ

### **推奨: 案1（geo_algorithms に実装）**

既存の `geo_algorithms` クレートを活用します。新規クレート作成は不要です。

```text
Phase 1: Primitives のみ（現在）
  geo_primitives 内で BasicCollision 実装

Phase 2: geo_algorithms に NURBS collision 追加
  geo_primitives ↘
  geo_nurbs     → geo_algorithms
  
Phase 3: （オプション）既存の collision を geo_algorithms に移動
  geo_primitives と geo_nurbs は Foundation のみ実装
```

### 実装手順

**ステップ1: geo_algorithms に geo_nurbs 依存追加**
```toml
# model/geo_algorithms/Cargo.toml
[dependencies]
analysis = { path = "../../foundation/analysis" }
geo_foundation = { path = "../geo_foundation" }
geo_primitives = { path = "../geo_primitives" }
geo_nurbs = { path = "../geo_nurbs" }  # ← 追加
```

**ステップ2: collision モジュール作成**
```rust
// model/geo_algorithms/src/collision.rs (新規)
use geo_primitives::*;
use geo_nurbs::*;
use geo_foundation::{BasicCollision, Scalar};

pub mod primitive_nurbs;  // NURBS × Primitives collision
pub mod nurbs_nurbs;      // NURBS × NURBS collision
```

**ステップ3: トレイト実装**
```rust
// model/geo_algorithms/src/collision/primitive_nurbs.rs
use geo_primitives::Circle3D;
use geo_nurbs::NurbsCurve3D;
use geo_foundation::{BasicCollision, Scalar};

impl<T: Scalar> BasicCollision<T, NurbsCurve3D<T>> for Circle3D<T> {
    fn intersects(&self, nurbs: &NurbsCurve3D<T>, tolerance: T) -> bool {
        // Circle と NURBS の衝突判定アルゴリズム
        // 1. NURBS を適応的にサンプリング
        // 2. 各セグメントと Circle の距離チェック
        // 3. tolerance 以下なら true
    }
    
    fn distance_to(&self, nurbs: &NurbsCurve3D<T>) -> T {
        // 最小距離計算（数値的手法）
    }
}

// 対称性のための実装
impl<T: Scalar> BasicCollision<T, Circle3D<T>> for NurbsCurve3D<T> {
    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        circle.intersects(self, tolerance) // 対称性を利用
    }
    
    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        circle.distance_to(self)
    }
}
```

## ✅ アーキテクチャの健全性チェック

### 依存関algorithms → geo_primitives
   geo_algorithms → geo_nurbs
   geo_primitives → geo_foundation
   geo_nurbs → geo_foundation

✅ Foundation パターン遵守:
   geo_primitives 内に NURBS の知識なし
   geo_nurbs 内に primitives の知識なし

✅ 拡張性:
   新しい形状を追加しても geo_algorithms に実装を追加するだけ

✅ 責務の一致:
   geo_algorithms = 高レベル幾何アルゴリズム
   collision/intersection は幾何アルゴリズムの一種
   geo_nurbs 内に primitives の知識なし

✅ 拡張性:
   新しい形状を追加しても geo_collision に実装を追加するだけ
```

### パフォーマンス考慮

1. **コンパイル時間**: 新クレート追加でビルド時間増加（許容範囲）
2. **実行時**: トレイトの静的ディスパッチで overhead なし
3. **バイナリサイズ**: 各組み合わせで専用コード生成

## 🔄 移行戦algorithms/Cargo.toml に geo_nurbs 依存追加
- 🔨 geo_algorithms/src/collision.rs モジュール作成
- 🔨 NURBS × Primitives collision 実装開始

### 長期（Phase 5以降）
- 🔨 （オプション）geo_primitives の collision を geo_algorithms に移動

### 中期（Phase 4）
- 🔨 geo_collision クレート作成
- 🔨 NURBS × Primitives collision 実装開始
- 🔨 段階的に既存実装を移動検討

### 長期（Phase 5以降）
- 🔨 全ての collision を geo_collision に統合
- 🔨 AdvancedCollision トレイトの実装
- 🔨 パフォーマンス最適化（BVH等）

## 📝 結論
既存の geo_algorithms クレートで解決可能
2. ✅ Foundation パターンを破壊しない
3. ✅ 新規クレート不要（過剰設計を回避）
4. ✅ 責務が一致（高レベル幾何アルゴリズム）
5. ✅ 依存関係が健全に保たれる

**今すぐ必要なアクション**:
- ❌ なし（現状維持で問題なし）

**Phase 4 で必要なアクション**:
1. geo_algorithms/Cargo.toml に `geo_nurbs = { path = "../geo_nurbs" }` 追加
2. geo_algorithms/src/collision.rs モジュール作成
3. NURBS collision 実装開始

**リスク**: なし
- 既存クレートの活用で過剰設計を回避
- orphan rules の制約なし（自プロジェクト内）
- ビルド時間増加なしsion 実装開始

**リスク**: 低
- orphan rules に注意が必要だが、自プロジェクト内なので問題なし
- ビルド時間増加は許容範囲
