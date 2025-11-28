# EllipseArc Core Traits Implementation Plan

**作成日**: 2025年11月28日
**最終更新日**: 2025年11月28日

## 概要

EllipseArc2D/3D の Core Traits 実装を段階的に行うための計画書。
Circle/LineSegment/Arc実装時の教訓を活かし、**Phase 1では最小限の12メソッド**のみ実装してエラー収束を確実にする。

## 問題認識

既存の `ellipse_arc_core_traits.rs` は以下の問題を抱えている：

- **Constructor**: 7-6個のメソッド（多すぎる）
- **Properties**: 13個以上のメソッド（多すぎる）
- **Measure**: 19個以上のメソッド（多すぎる）
- **合計**: 40個以上のメソッド → エラー収束不可能

## 設計方針

Circle/LineSegment/Arc実装と同じく、**最も基本的な3-5-4パターン**を採用：

- **Constructor**: 3メソッド（基本生成のみ）
- **Properties**: 5メソッド（必須プロパティのみ）
- **Measure**: 4メソッド（基本計量のみ）

## Phase 1: 最小限Core実装（12メソッド）

### Phase 1 実装リスト

#### EllipseArc2DConstructor (3メソッド)

```rust
pub trait EllipseArc2DConstructor<T: Scalar> {
    /// 中心、長軸、短軸、回転角、角度範囲から楕円弧を作成
    fn new(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T
    ) -> Option<Self>;
    
    /// XY平面上の単位楕円弧を作成（回転なし、0度から90度）
    fn unit_ellipse_arc() -> Self;
    
    /// 指定した角度範囲の楕円弧を作成（簡易版）
    fn from_ellipse_and_angles(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        start_angle: T,
        end_angle: T
    ) -> Option<Self>;
}
```

#### EllipseArc2DProperties (5メソッド)

```rust
pub trait EllipseArc2DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T);
    
    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;
    
    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;
    
    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;
    
    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;
}
```

#### EllipseArc2DMeasure (4メソッド)

```rust
pub trait EllipseArc2DMeasure<T: Scalar> {
    /// 楕円弧の長さ（測度）
    fn measure(&self) -> T;
    
    /// 開始点を取得
    fn start_point(&self) -> (T, T);
    
    /// 終了点を取得
    fn end_point(&self) -> (T, T);
    
    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}
```

#### EllipseArc3DConstructor (3メソッド)

```rust
pub trait EllipseArc3DConstructor<T: Scalar> {
    /// 3D空間での楕円弧を作成（法線、長軸方向、角度範囲）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major: T,
        semi_minor: T,
        major_direction: (T, T, T),
        start_angle: T,
        end_angle: T
    ) -> Option<Self>;
    
    /// XY平面上の楕円弧を作成
    fn xy_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T
    ) -> Option<Self>;
    
    /// 単位楕円弧をXY平面上に作成
    fn unit_ellipse_arc_xy() -> Self;
}
```

#### EllipseArc3DProperties (5メソッド)

```rust
pub trait EllipseArc3DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T, T);
    
    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;
    
    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;
    
    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;
    
    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;
}
```

#### EllipseArc3DMeasure (4メソッド)

```rust
pub trait EllipseArc3DMeasure<T: Scalar> {
    /// 楕円弧の長さ（測度）
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

### Step 1: トレイト定義の最小化

`geo_foundation/src/core/ellipse_arc_core_traits.rs` を Phase 1 の12メソッドのみに書き換える。

### Step 2: EllipseArc2D 実装

`geo_primitives/src/ellipse_arc_2d.rs` に Phase 1 Core Traits を実装。

### Step 3: EllipseArc3D 実装

`geo_primitives/src/ellipse_arc_3d.rs` に Phase 1 Core Traits を実装。

### Step 4: 検証

```bash
cargo build -p geo_primitives
cargo clippy -p geo_primitives -- -D warnings
cargo test -p geo_primitives ellipse_arc
```

## 実装上の注意点

### Angle型の扱い

既存実装では `Angle<T>` を使用しているが、トレイトでは `T`（ラジアン値）を使用して型依存を排除。

### 楕円弧の長さ計算

楕円弧の長さは解析的に求まらないため、数値積分が必要。Phase 1では簡易近似を使用。

### 基底楕円との関係

EllipseArc2D/3Dは基底となるEllipse2D/3Dを持つ設計。

## Phase 2: 標準機能追加（予定）

Phase 1が成功したら以下を追加：

- Constructor: `from_three_points`, `from_center_and_endpoints`など
- Properties: `rotation`, `sweep_angle`, `mid_point`など
- Measure: `arc_length_precise`, `sector_area`, `tangent_at_parameter`など

## Phase 3: 高度な機能（予定）

- 楕円弧の交差判定
- パラメトリック表現機能
- 適応的サンプリング

## ✅ 実装完了

**実装日**: 2025年11月28日  
**ステータス**: Phase 1 完了

### 検証結果

- ✅ `cargo build -p geo_primitives`: 成功
- ✅ `cargo clippy -p geo_primitives -- -D warnings`: 0 warnings
- ✅ `cargo test -p geo_primitives`: 312 tests passed

### 実装ファイル

- `geo_foundation/src/core/ellipse_arc_core_traits.rs`: 40+メソッドから12メソッドに最小化
- `geo_primitives/src/ellipse_arc_2d.rs`: EllipseArc2D Core Traits 実装
- `geo_primitives/src/ellipse_arc_3d.rs`: EllipseArc3D Core Traits 実装

**Foundation Pattern 進捗**: 22/25 形状完了 (88%)

## 参考

- Arc実装: `ARC_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- Circle実装: `CIRCLE_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- LineSegment実装: `LINESEGMENT_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- Foundation設計: `FOUNDATION_CORE_EXTENSION_REDESIGN_PROPOSAL.md`
