# LineSegment Core Traits Implementation Plan

**作成日**: 2025年11月28日
**最終更新日**: 2025年11月28日

## 概要

LineSegment2D/3D の Core Traits 実装を段階的に行うための計画書。
Circle実装時の教訓を活かし、**Phase 1では最小限の12メソッド**のみ実装してエラー収束を確実にする。

## 問題認識

既存の `linesegment_core_traits.rs` は以下の問題を抱えている：

- **Constructor**: 8-10個のメソッド（多すぎる）
- **Properties**: 15-22個のメソッド（多すぎる）
- **Measure**: 20-23個のメソッド（多すぎる）
- **合計**: 43-55個のメソッド → エラー収束不可能

## Phase 1: 最小限Core実装（12メソッド）

### 設計方針

Circle実装と同じく、**最も基本的な3-5-4パターン**を採用：

- **Constructor**: 3メソッド（基本生成のみ）
- **Properties**: 5メソッド（必須プロパティのみ）
- **Measure**: 4メソッド（基本計量のみ）

### Phase 1 実装リスト

#### LineSegment2DConstructor (3メソッド)

```rust
pub trait LineSegment2DConstructor<T: Scalar> {
    /// 2つの点から線分を作成
    fn new(start: (T, T), end: (T, T)) -> Option<Self>;
    
    /// 起点、方向、長さから線分を作成
    fn from_point_direction_length(start: (T, T), direction: (T, T), length: T) -> Option<Self>;
    
    /// X軸方向の単位線分（原点から(1,0)まで）
    fn unit_x() -> Self;
}
```

#### LineSegment2DProperties (5メソッド)

```rust
pub trait LineSegment2DProperties<T: Scalar> {
    /// 開始点を取得
    fn start(&self) -> (T, T);
    
    /// 終了点を取得
    fn end(&self) -> (T, T);
    
    /// 中点を取得
    fn midpoint(&self) -> (T, T);
    
    /// 線分の長さを取得
    fn length(&self) -> T;
    
    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}
```

#### LineSegment2DMeasure (4メソッド)

```rust
pub trait LineSegment2DMeasure<T: Scalar> {
    /// 線分の長さ（測度）
    fn measure(&self) -> T;
    
    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;
    
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T)) -> bool;
    
    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}
```

#### LineSegment3DConstructor (3メソッド)

```rust
pub trait LineSegment3DConstructor<T: Scalar> {
    /// 2つの3D点から線分を作成
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self>;
    
    /// 起点、方向、長さから3D線分を作成
    fn from_point_direction_length(start: (T, T, T), direction: (T, T, T), length: T) -> Option<Self>;
    
    /// X軸方向の単位線分（原点から(1,0,0)まで）
    fn unit_x() -> Self;
}
```

#### LineSegment3DProperties (5メソッド)

```rust
pub trait LineSegment3DProperties<T: Scalar> {
    /// 開始点を取得
    fn start(&self) -> (T, T, T);
    
    /// 終了点を取得
    fn end(&self) -> (T, T, T);
    
    /// 中点を取得
    fn midpoint(&self) -> (T, T, T);
    
    /// 線分の長さを取得
    fn length(&self) -> T;
    
    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}
```

#### LineSegment3DMeasure (4メソッド)

```rust
pub trait LineSegment3DMeasure<T: Scalar> {
    /// 線分の長さ（測度）
    fn measure(&self) -> T;
    
    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T, T)) -> bool;
    
    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}
```

## 実装手順

### Step 1: トレイト定義の最小化

`geo_foundation/src/core/linesegment_core_traits.rs` を Phase 1 の12メソッドのみに書き換える。

### Step 2: LineSegment2D 実装

`geo_primitives/src/line_segment_2d.rs` に Phase 1 Core Traits を実装。

### Step 3: LineSegment3D 実装

`geo_primitives/src/line_segment_3d.rs` に Phase 1 Core Traits を実装。

### Step 4: 検証

```bash
cargo build -p geo_primitives
cargo clippy -p geo_primitives -- -D warnings
cargo test -p geo_primitives line_segment
```

## Phase 2: 標準機能追加（予定）

Phase 1が成功したら以下を追加：

- Constructor: `from_2d_in_xy_plane`, `degenerate`など
- Properties: `direction_vector`, `is_horizontal`, `bounding_box`など
- Measure: `closest_point_to`, `intersects_segment`, `reverse`など

## Phase 3: 高度な機能（予定）

- 線分交差判定の詳細実装
- 平面との交点計算
- 線分の分割・延長機能

## ✅ 実装完了

**実装日**: 2025年11月28日  
**ステータス**: Phase 1 完了

### 検証結果

- ✅ `cargo build -p geo_primitives`: 成功
- ✅ `cargo clippy -p geo_primitives -- -D warnings`: 0 warnings
- ✅ `cargo test -p geo_primitives`: 310 tests passed

### 実装ファイル

- `geo_foundation/src/core/linesegment_core_traits.rs`: Core Traits 定義 (12メソッド)
- `geo_primitives/src/line_segment_2d.rs`: LineSegment2D 実装
- `geo_primitives/src/line_segment_3d.rs`: LineSegment3D 実装

**Foundation Pattern 進捗**: 18/25 形状完了 (72%)

## 参考

- Circle実装: `CIRCLE_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- Foundation設計: `FOUNDATION_CORE_EXTENSION_REDESIGN_PROPOSAL.md`
