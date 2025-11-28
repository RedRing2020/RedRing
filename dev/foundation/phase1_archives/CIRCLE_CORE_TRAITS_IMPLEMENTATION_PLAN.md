# Circle Core Traits 段階的実装計画

**作成日**: 2025年11月11日  
**最終更新日**: 2025年11月28日  
**ステータス**: ✅ Phase 1 完了  
**対象**: Circle2D/Circle3D の Foundation Pattern 実装

## 🎯 実装方針

**問題**: 一度に多くのメソッドを定義すると実装時にエラーが収束せず破綻する  
**解決策**: 3段階に分けて最小限のメソッドから実装開始

---

## 📋 Phase 1: 最小限の Core Traits（必須メソッドのみ）

### 目標
- ビルドが通る最小限の実装
- 既存機能を壊さない
- Circle の基本的な使用ができる

### Constructor (3メソッド)

```rust
pub trait Circle2DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、半径）
    fn new(center: (T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 参照方向を指定して作成（STEP準拠）
    fn new_with_ref_direction(
        center: (T, T),
        radius: T,
        ref_direction: (T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    /// 単位円作成（原点中心、半径1）
    fn unit_circle() -> Self
    where
        Self: Sized;
}

pub trait Circle3DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、軸方向、半径）
    fn new(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の円作成
    fn new_xy_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XY平面単位円
    fn unit_circle_xy() -> Self
    where
        Self: Sized;
}
```

### Properties (5メソッド)

```rust
pub trait Circle2DProperties<T: Scalar> {
    /// 中心点取得
    fn center(&self) -> (T, T);

    /// 半径取得
    fn radius(&self) -> T;

    /// 参照方向取得
    fn ref_direction(&self) -> (T, T);

    /// 直径取得
    fn diameter(&self) -> T;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

pub trait Circle3DProperties<T: Scalar> {
    /// 中心点取得
    fn center(&self) -> (T, T, T);

    /// 半径取得
    fn radius(&self) -> T;

    /// 軸方向（法線）取得
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得
    fn ref_direction(&self) -> (T, T, T);

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}
```

### Measure (4メソッド)

```rust
pub trait Circle2DMeasure<T: Scalar> {
    /// 円周の長さ
    fn circumference(&self) -> T;

    /// 円の面積
    fn area(&self) -> T;

    /// 点が円内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 点から円周への距離
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Circle3DMeasure<T: Scalar> {
    /// 円周の長さ
    fn circumference(&self) -> T;

    /// 円の面積
    fn area(&self) -> T;

    /// 点が円内部にあるか判定（平面上も考慮）
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点から円周への距離
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}
```

**Phase 1 合計**: Constructor(3) + Properties(5) + Measure(4) = **12メソッド/図形**

---

## 📋 Phase 2: 標準機能追加

Phase 1 が安定してから追加するメソッド

### Constructor 追加 (各3-4メソッド)

```rust
// Circle2DConstructor に追加
fn from_center_and_point(center: (T, T), point_on_circle: (T, T)) -> Option<Self>;
fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>;
fn centered_at_origin(radius: T) -> Option<Self>;

// Circle3DConstructor に追加
fn new_with_ref_direction(...) -> Option<Self>;
fn new_xz_plane(center: (T, T, T), radius: T) -> Option<Self>;
fn new_yz_plane(center: (T, T, T), radius: T) -> Option<Self>;
fn unit_circle_xz() -> Self;
fn unit_circle_yz() -> Self;
```

### Properties 追加 (各3-4メソッド)

```rust
// Circle2DProperties に追加
fn is_unit_circle(&self) -> bool;
fn is_centered_at_origin(&self) -> bool;
fn is_degenerate(&self) -> bool;

// Circle3DProperties に追加
fn normal(&self) -> (T, T, T);  // axis のエイリアス
fn is_unit_circle(&self) -> bool;
fn is_centered_at_origin(&self) -> bool;
fn is_on_xy_plane(&self) -> bool;
```

### Measure 追加 (各4-5メソッド)

```rust
// Circle2DMeasure に追加
fn point_on_circumference(&self, point: (T, T)) -> bool;
fn closest_point_to(&self, point: (T, T)) -> (T, T);
fn point_at_parameter(&self, t: T) -> (T, T);
fn parameter_at_point(&self, point: (T, T)) -> Option<T>;
fn distance_to_circle(&self, other: &Self) -> T;

// Circle3DMeasure に追加
fn point_on_circumference(&self, point: (T, T, T)) -> bool;
fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);
fn point_at_parameter(&self, t: T) -> (T, T, T);
fn parameter_at_point(&self, point: (T, T, T)) -> Option<T>;
```

---

## 📋 Phase 3: 高度な機能（オプション）

Phase 2 が安定してから必要に応じて追加

### 交差計算
```rust
fn intersection_with_circle(&self, other: &Self) -> Vec<(T, T)>;
fn intersection_with_line(&self, line_point: (T, T), line_direction: (T, T)) -> Vec<(T, T)>;
fn intersects_circle(&self, other: &Self) -> bool;
```

### 包含判定
```rust
fn contains_circle(&self, other: &Self) -> bool;
fn is_inside_circle(&self, other: &Self) -> bool;
fn is_tangent_to_circle(&self, other: &Self) -> bool;
```

---

## 🔧 実装手順

### Step 1: Trait定義ファイル更新

**ファイル**: `model/geo_foundation/src/core/circle_core_traits.rs`

```rust
// Phase 1 の最小限トレイト定義のみ記述
// 既存の余分なメソッドは全てコメントアウトまたは削除
```

### Step 2: Circle2D実装

**ファイル**: `model/geo_primitives/src/circle_2d.rs`

```rust
// Phase 1 トレイト実装を追加
impl<T: Scalar> Circle2DConstructor<T> for Circle2D<T> {
    // 3メソッドのみ実装
}

impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
    // 5メソッドのみ実装
}

impl<T: Scalar> Circle2DMeasure<T> for Circle2D<T> {
    // 4メソッドのみ実装
}
```

### Step 3: Circle3D実装

**ファイル**: `model/geo_primitives/src/circle_3d.rs`

同様に Phase 1 のメソッドのみ実装

### Step 4: ビルド確認

```bash
cargo build
cargo clippy --workspace -- -D warnings
cargo test -p geo_primitives circle
```

### Step 5: Phase 2 へ移行

Phase 1 が完全に動作確認できたら、Phase 2 のメソッドを追加

---

## ✅ 成功基準

### Phase 1
- [ ] `cargo build` 成功
- [ ] `cargo clippy` 警告なし
- [ ] 基本的な円の生成・プロパティ取得・計量が動作
- [ ] 既存のテストが全てパス

### Phase 2
- [ ] Phase 1 の機能を維持
- [ ] 標準的な幾何操作が動作
- [ ] Arc2D/3D との互換性確保

### Phase 3
- [ ] 高度な交差計算が動作
- [ ] 包含判定が正確

---

## 📚 参考実装

### 成功例
- `Ray2D/3D` - 各トレイトに適度なメソッド数（10-15個）
- `Direction2D/3D` - 段階的に実装完了
- `Ellipse2D/3D` - 複雑な形状でも成功

### 学習ポイント
1. **最小限から開始**: まず動くものを作る
2. **段階的追加**: 一度に1機能ずつ
3. **テスト駆動**: 各段階でテストを確認

---

## 🚨 注意事項

### やってはいけないこと
- ❌ 最初から全メソッドを定義
- ❌ 未実装メソッドを残したまま進む
- ❌ エラーが出たまま次の実装に進む

### 推奨事項
- ✅ Phase 1 完了まで他のメソッドは追加しない
- ✅ 各段階で必ずビルド・テスト確認
- ✅ エラーは即座に修正

---

## ✅ 実装完了

**実装日**: 2025年11月11日  
**ステータス**: Phase 1 完了

### 検証結果
- ✅ `cargo build` 成功
- ✅ `cargo clippy` 警告なし
- ✅ `cargo test` 16 tests passed
- ✅ 既存テスト全てパス

### 実装ファイル
- `geo_foundation/src/core/circle_core_traits.rs`: Core Traits 定義
- `geo_primitives/src/circle_2d.rs`: Circle2D 実装
- `geo_primitives/src/circle_3d.rs`: Circle3D 実装

**Foundation Pattern 進捗**: Circle2D/3D 実装により 16/25 形状完了 (64%)
