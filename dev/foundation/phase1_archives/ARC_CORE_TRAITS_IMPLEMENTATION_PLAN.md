# Arc Core Traits Implementation Plan

**作成日**: 2025年11月28日  
**最終更新日**: 2025年11月28日  
**ステータス**: ✅ Phase 1 完了

## 概要

Arc2D/3D の Core Traits 実装を段階的に行うための計画書。
Circle/LineSegment実装時の教訓を活かし、**Phase 1では最小限の12メソッド**のみ実装してエラー収束を確実にする。

## 設計方針

Circle/LineSegment実装と同じく、**最も基本的な3-5-4パターン**を採用：

- **Constructor**: 3メソッド（基本生成のみ）
- **Properties**: 5メソッド（必須プロパティのみ）
- **Measure**: 4メソッド（基本計量のみ）

## Phase 1: 最小限Core実装（12メソッド）

### Phase 1 実装リスト

#### Arc2DConstructor (3メソッド)

```rust
pub trait Arc2DConstructor<T: Scalar> {
    /// 中心、半径、角度から円弧を作成
    fn new(center: (T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>;
    
    /// 3点から円弧を作成
    fn from_three_points(start: (T, T), mid: (T, T), end: (T, T)) -> Option<Self>;
    
    /// 半円を作成（開始角度0、終了角度π）
    fn semicircle(center: (T, T), radius: T) -> Self;
}
```

#### Arc2DProperties (5メソッド)

```rust
pub trait Arc2DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T);
    
    /// 半径を取得
    fn radius(&self) -> T;
    
    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;
    
    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;
    
    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}
```

#### Arc2DMeasure (4メソッド)

```rust
pub trait Arc2DMeasure<T: Scalar> {
    /// 円弧の長さ（測度）
    fn measure(&self) -> T;
    
    /// 開始点を取得
    fn start_point(&self) -> (T, T);
    
    /// 終了点を取得
    fn end_point(&self) -> (T, T);
    
    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}
```

#### Arc3DConstructor (3メソッド)

```rust
pub trait Arc3DConstructor<T: Scalar> {
    /// 中心、半径、法線、角度から円弧を作成
    fn new(
        center: (T, T, T),
        radius: T,
        normal: (T, T, T),
        start_angle: T,
        end_angle: T
    ) -> Option<Self>;
    
    /// XY平面上の円弧を作成
    fn xy_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>;
    
    /// 3点から3D円弧を作成
    fn from_three_points(start: (T, T, T), mid: (T, T, T), end: (T, T, T)) -> Option<Self>;
}
```

#### Arc3DProperties (5メソッド)

```rust
pub trait Arc3DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T, T);
    
    /// 半径を取得
    fn radius(&self) -> T;
    
    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;
    
    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;
    
    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}
```

#### Arc3DMeasure (4メソッド)

```rust
pub trait Arc3DMeasure<T: Scalar> {
    /// 円弧の長さ（測度）
    fn measure(&self) -> T;
    
    /// 開始点を取得
    fn start_point(&self) -> (T, T, T);
    
    /// 終了点を取得
    fn end_point(&self) -> (T, T, T);
    
    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}
```

## 実装手順

### Step 1: トレイト定義の作成

`geo_foundation/src/core/arc_core_traits.rs` を Phase 1 の12メソッドで新規作成。

### Step 2: Arc2D 実装

`geo_primitives/src/arc_2d.rs` に Phase 1 Core Traits を実装。
既存の `Angle<T>` 型を活用。

### Step 3: Arc3D 実装

`geo_primitives/src/arc_3d.rs` に Phase 1 Core Traits を実装。

### Step 4: 検証

```bash
cargo build -p geo_primitives
cargo clippy -p geo_primitives -- -D warnings
cargo test -p geo_primitives arc
```

## 実装上の注意点

### Angle型の扱い

既存実装では `analysis::Angle<T>` を使用しているが、トレイトでは `T`（ラジアン値）を使用して型依存を排除。

### 3点からの円弧生成

- 2D: 3点を通る円を求めて円弧を構成
- 3D: 3点を通る円と平面法線を求める

### 半円の生成

簡易的な生成メソッドとして、よく使われるパターンを提供。

## Phase 2: 標準機能追加（予定）

Phase 1が成功したら以下を追加：

- Constructor: `from_circle`, `quarter_circle`など
- Properties: `angular_span`, `midpoint`, `is_full_circle`など
- Measure: `sector_area`, `chord_length`, `contains_point`など

## Phase 3: 高度な機能（予定）

- 円弧の交差判定
- トリミング操作
- 円弧の結合・分割

## ✅ 実装完了

**実装日**: 2025年11月28日  
**ステータス**: Phase 1 完了

### 検証結果

- ✅ `cargo build -p geo_primitives`: 成功
- ✅ `cargo clippy -p geo_primitives -- -D warnings`: 0 warnings
- ✅ `cargo test -p geo_primitives`: 11 arc tests + 310 total tests passed

### 実装ファイル

- `geo_foundation/src/core/arc_core_traits.rs`: Core Traits 定義（12メソッド）
- `geo_primitives/src/arc_2d.rs`: Arc2D 実装（3点円弧生成含む）
- `geo_primitives/src/arc_3d.rs`: Arc3D 実装（Rodrigues回転含む）

**Foundation Pattern 進捗**: Arc2D/3D 実装により 20/25 形状完了 (80%)

## 参考

- Circle実装: `CIRCLE_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- LineSegment実装: `LINESEGMENT_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- Foundation設計: `FOUNDATION_CORE_EXTENSION_REDESIGN_PROPOSAL.md`
