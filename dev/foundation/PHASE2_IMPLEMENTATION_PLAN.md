# Foundation Pattern Phase 2 実装計画

**作成日**: 2025年11月28日  
**Phase 1 完了日**: 2025年11月28日  
**ステータス**: 準備中

---

## 📊 Phase 2 概要

Phase 1 で確立した Core Traits (Constructor/Properties/Measure) の 3-5-4 パターンを、**標準機能**へ拡張します。

### Phase 2 の目標

1. **標準機能の追加**: Phase 1 の最小実装から実用的な機能セットへ拡張
2. **高度な計算トレイトの実装**: EllipseCalculation, ArcCalculation 等の特化トレイト
3. **幾何演算の拡張**: 交差判定、包含判定、パラメトリック操作

---

## 🎯 Phase 2 実装パターン

### 拡張方針

Phase 1 で確立した **3-5-4 パターン**を維持しつつ、各カテゴリに**追加メソッド**を実装：

```rust
// Phase 1: 最小実装 (3-5-4)
Constructor: 3メソッド
Properties: 5メソッド  
Measure: 4メソッド
Total: 12メソッド

// Phase 2: 標準機能追加 (6-8-8)
Constructor: 3 + 3 = 6メソッド
Properties: 5 + 3 = 8メソッド
Measure: 4 + 4 = 8メソッド
Total: 22メソッド
```

---

## 📋 Phase 2 対象図形と優先順位

### 【高優先度】基本図形 (4図形)

#### 1. Circle2D/3D
**理由**: 最も使用頻度が高く、交差判定・接線計算が重要

**追加メソッド** (Constructor +3):
```rust
fn from_center_and_point(center: (T, T), point_on_circle: (T, T)) -> Option<Self>;
fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>;
fn centered_at_origin(radius: T) -> Option<Self>;
```

**追加メソッド** (Properties +3):
```rust
fn is_unit_circle(&self) -> bool;
fn is_centered_at_origin(&self) -> bool;
fn is_degenerate(&self) -> bool;
```

**追加メソッド** (Measure +4):
```rust
fn point_on_circumference(&self, point: (T, T)) -> bool;
fn closest_point_to(&self, point: (T, T)) -> (T, T);
fn point_at_parameter(&self, t: T) -> (T, T);
fn distance_to_circle(&self, other: &Self) -> T;
```

#### 2. LineSegment2D/3D
**理由**: 直線系の完成、交差判定が重要

**追加メソッド** (Constructor +3):
```rust
fn from_points_with_length(start: (T, T), direction: (T, T), length: T) -> Option<Self>;
fn horizontal(start_x: T, end_x: T, y: T) -> Option<Self>;
fn vertical(x: T, start_y: T, end_y: T) -> Option<Self>;
```

**追加メソッド** (Properties +3):
```rust
fn is_horizontal(&self) -> bool;
fn is_vertical(&self) -> bool;
fn is_degenerate(&self) -> bool;
```

**追加メソッド** (Measure +4):
```rust
fn point_on_segment(&self, point: (T, T), tolerance: T) -> bool;
fn closest_point_to(&self, point: (T, T)) -> (T, T);
fn point_at_parameter(&self, t: T) -> (T, T);
fn distance_to_segment(&self, other: &Self) -> T;
```

### 【中優先度】曲線系 (4図形)

#### 3. Arc2D/3D
**追加メソッド** (Constructor +3):
```rust
fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>;
fn from_chord(start: (T, T), end: (T, T), height: T) -> Option<Self>;
fn semicircle(center: (T, T), radius: T, start_angle: T) -> Option<Self>;
```

#### 4. EllipseArc2D/3D
**追加メソッド** (Constructor +3):
```rust
fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T), semi_minor: T) -> Option<Self>;
fn from_chord(start: (T, T), end: (T, T), height: T, semi_minor: T) -> Option<Self>;
fn semi_ellipse(center: (T, T), semi_major: T, semi_minor: T, start_angle: T) -> Option<Self>;
```

### 【低優先度】その他 (3図形)

#### 5. Ellipse2D/3D
**追加メソッド**: 高度な楕円計算（Phase 1 で基本実装済み）

#### 6. Triangle2D/3D
**追加メソッド**: 三角形の特性判定（正三角形、直角三角形等）

#### 7. Plane3D
**追加メソッド**: 平面との交差判定、投影計算

---

## 🔧 Phase 2 実装手順

### Step 1: Circle2D の拡張実装（パイロット）

Circle2D を Phase 2 のパイロット実装として選定し、標準パターンを確立：

1. **Core Traits 拡張**
   - `geo_foundation/src/core/circle_core_traits.rs` を更新
   - Constructor/Properties/Measure に各3-4メソッド追加

2. **実装**
   - `geo_primitives/src/circle_2d.rs` に実装追加
   - 既存の内部メソッドを活用

3. **テスト**
   - 新規メソッドのユニットテスト追加
   - `cargo test -p geo_primitives circle_2d` で検証

4. **ドキュメント**
   - 実装パターンを文書化
   - 他の図形への適用ガイド作成

### Step 2: LineSegment2D の実装

Circle2D で確立したパターンを LineSegment2D に適用：

1. Trait 拡張
2. 実装追加
3. テスト追加
4. パターン検証

### Step 3: 他図形への展開

確立したパターンを以下の順序で展開：

1. Circle3D
2. LineSegment3D
3. Arc2D/3D
4. EllipseArc2D/3D
5. その他

---

## 📋 高度な計算トレイト (Phase 2+)

Phase 2 では既存の高度な計算トレイトも整理・統合：

### EllipseCalculation トレイト

既に実装済みだが、Phase 2 で統一パターンに整理：

```rust
pub trait EllipseCalculation<T: Scalar> {
    type Point;
    
    // 周長計算（複数の近似式）
    fn perimeter_ramanujan_i(&self) -> T;
    fn perimeter_ramanujan_ii(&self) -> T;
    fn perimeter_pade(&self) -> T;
    fn perimeter_cantrell(&self) -> T;
    fn perimeter_series(&self, terms: usize) -> T;
    fn perimeter_numerical(&self, n_points: usize) -> T;
    
    // 幾何パラメータ
    fn eccentricity(&self) -> T;
    fn focal_distance(&self) -> T;
    fn area(&self) -> T;
    fn foci(&self) -> (Self::Point, Self::Point);
}
```

### ArcCalculation トレイト (新規)

```rust
pub trait ArcCalculation<T: Scalar> {
    fn arc_length(&self) -> T;
    fn chord_length(&self) -> T;
    fn sagitta(&self) -> T;  // 矢高
    fn angle_span(&self) -> T;
    fn midpoint(&self) -> (T, T);
}
```

---

## 🎯 Phase 2 完了条件

### 必須条件

1. **最低4図形の拡張実装**
   - Circle2D/3D
   - LineSegment2D/3D

2. **ビルド・テスト成功**
   ```bash
   ✅ cargo build --workspace
   ✅ cargo clippy --workspace -- -D warnings
   ✅ cargo test --workspace
   ```

3. **ドキュメント整備**
   - Phase 2 実装パターン文書
   - 各図形の拡張機能リファレンス

### オプション条件

1. **曲線系の拡張実装**
   - Arc2D/3D
   - EllipseArc2D/3D

2. **高度な計算トレイトの整理**
   - EllipseCalculation の統一化
   - ArcCalculation の新規実装

---

## 📚 参考資料

### Phase 1 実装パターン

- `dev/foundation/FOUNDATION_CORE_TRAITS_REDESIGN_METHODOLOGY.md` - 基本方法論
- Phase 1 完了ドキュメント (作成予定)

### 個別実装計画 (Phase 1)

以下のドキュメントに Phase 2 の詳細記載あり：
- `CIRCLE_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- `ARC_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- `LINESEGMENT_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- `ELLIPSEARC_CORE_TRAITS_IMPLEMENTATION_PLAN.md`
- `TRIANGLE_CORE_TRAITS_IMPLEMENTATION_PLAN.md`

---

## 🚀 次のステップ

1. **GitHub Issue 作成**: Phase 2 の実装トラッキング用 Issue 作成
2. **Circle2D パイロット実装**: 最初の拡張実装を開始
3. **パターン確立**: 他図形への展開パターンを確立
4. **段階的展開**: 優先順位に従って順次実装

---

**作成者**: GitHub Copilot + n-takatsu  
**Phase 1 完了日**: 2025年11月28日  
**Phase 2 開始予定日**: 2025年11月28日
