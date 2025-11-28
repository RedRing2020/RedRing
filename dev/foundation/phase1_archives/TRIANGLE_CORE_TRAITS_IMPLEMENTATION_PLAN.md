# Triangle2D/3D Core Traits Implementation Plan

**作成日**: 2025年11月28日  
**最終更新日**: 2025年11月28日  
**ステータス**: ✅ 完了

## 概要

Triangle2D/3D の Foundation Pattern Phase 1 実装。
3-5-4 パターン（Constructor 3 + Properties 5 + Measure 4）で統一。

---

## 実装方針

### Phase 1 の範囲（12メソッド）

#### Constructor（3メソッド）
- Triangle2D:
  - `new(a, b, c)` - 3点から三角形を構築
  - `unit_triangle()` - 単位正三角形
  - `from_array(points)` - 配列から構築

- Triangle3D:
  - `new(a, b, c)` - 3点から三角形を構築（3D座標）
  - `from_array(points)` - 配列から構築（3D）
  - `unit_triangle_xy()` - xy平面上の単位正三角形

#### Properties（5メソッド）
- Triangle2D:
  - `vertex_a()` - 頂点A座標
  - `vertex_b()` - 頂点B座標
  - `vertex_c()` - 頂点C座標
  - `centroid()` - 重心座標
  - `circumcenter()` - 外心座標（Option）

- Triangle3D:
  - `vertex_a()` - 頂点A座標（3D）
  - `vertex_b()` - 頂点B座標（3D）
  - `vertex_c()` - 頂点C座標（3D）
  - `centroid()` - 重心座標（3D）
  - `normal()` - 法線ベクトル（正規化済み）

#### Measure（4メソッド）
- 共通（2D/3D）:
  - `measure()` - 面積を返す
  - `edge_ab_length()` - 辺ABの長さ
  - `edge_bc_length()` - 辺BCの長さ
  - `edge_ca_length()` - 辺CAの長さ

---

## 実装詳細

### 1. Core Traits 定義

**ファイル**: `geo_foundation/src/core/triangle_core_traits.rs`（新規作成）

```rust
// Triangle2D Core Traits
pub trait Triangle2DConstructor<T: Scalar>: Sized {
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self>;
    fn unit_triangle() -> Self;
    fn from_array(points: [(T, T); 3]) -> Option<Self>;
}

pub trait Triangle2DProperties<T: Scalar> {
    fn vertex_a(&self) -> (T, T);
    fn vertex_b(&self) -> (T, T);
    fn vertex_c(&self) -> (T, T);
    fn centroid(&self) -> (T, T);
    fn circumcenter(&self) -> Option<(T, T)>;
}

pub trait Triangle2DMeasure<T: Scalar> {
    fn measure(&self) -> T;
    fn edge_ab_length(&self) -> T;
    fn edge_bc_length(&self) -> T;
    fn edge_ca_length(&self) -> T;
}

pub trait Triangle2DCore<T: Scalar>:
    Triangle2DConstructor<T> + Triangle2DProperties<T> + Triangle2DMeasure<T>
{}

// Triangle3D Core Traits (同様の構造)
```

**特徴**:
- 退化三角形（3点が一直線上）は `None` を返す
- タプル座標を使用して型システムの一貫性を保つ
- Triangle3D は normal() で法線ベクトルを提供

---

### 2. Triangle2D 実装

**ファイル**: `geo_primitives/src/triangle_2d.rs`

```rust
impl<T: Scalar> Triangle2DConstructor<T> for Triangle2D<T> {
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self> {
        let pa = Point2D::new(a.0, a.1);
        let pb = Point2D::new(b.0, b.1);
        let pc = Point2D::new(c.0, c.1);
        Self::new(pa, pb, pc) // 既存の実装メソッドを呼び出す
    }

    fn unit_triangle() -> Self {
        Self::unit_triangle() // 既存の実装メソッドを呼び出す
    }

    fn from_array(points: [(T, T); 3]) -> Option<Self> {
        let pa = Point2D::new(points[0].0, points[0].1);
        let pb = Point2D::new(points[1].0, points[1].1);
        let pc = Point2D::new(points[2].0, points[2].1);
        Self::new(pa, pb, pc)
    }
}
```

**実装のポイント**:
- トレイトメソッドは内部の実装メソッドをラップする形式
- タプル座標 ↔ Point2D の変換を適切に行う
- `measure()` は既存の `area()` メソッドを呼び出す

---

### 3. Triangle3D 実装

**ファイル**: `geo_primitives/src/triangle_3d.rs`

```rust
impl<T: Scalar> Triangle3DConstructor<T> for Triangle3D<T> {
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self> {
        let pa = Point3D::new(a.0, a.1, a.2);
        let pb = Point3D::new(b.0, b.1, b.2);
        let pc = Point3D::new(c.0, c.1, c.2);
        Self::new(pa, pb, pc)
    }

    fn from_array(points: [(T, T, T); 3]) -> Option<Self> {
        let pa = Point3D::new(points[0].0, points[0].1, points[0].2);
        let pb = Point3D::new(points[1].0, points[1].1, points[1].2);
        let pc = Point3D::new(points[2].0, points[2].1, points[2].2);
        Self::new(pa, pb, pc)
    }

    fn unit_triangle_xy() -> Self {
        let h = T::from_f64(0.8660254037844387); // sqrt(3)/2
        let pa = Point3D::new(T::ZERO, T::ONE, T::ZERO);
        let pb = Point3D::new(-h, -T::ONE / (T::ONE + T::ONE), T::ZERO);
        let pc = Point3D::new(h, -T::ONE / (T::ONE + T::ONE), T::ZERO);
        Self::new(pa, pb, pc).expect("Unit triangle should always be valid")
    }
}

impl<T: Scalar> Triangle3DProperties<T> for Triangle3D<T> {
    fn normal(&self) -> (T, T, T) {
        let n = self.normal().unwrap_or(Vector3D::unit_z());
        (n.x(), n.y(), n.z())
    }
    // ... 他のプロパティ
}
```

**3D 特有の考慮事項**:
- `unit_triangle_xy()`: xy平面上に単位正三角形を生成
- `normal()`: 退化した場合は Z軸単位ベクトルをフォールバック
- 3成分タプル座標を一貫して使用

---

## 既存実装との統合

### 既存メソッドの活用

Triangle2D/3D には既に以下のメソッドが実装済み:
- `new(Point, Point, Point)` - 内部実装
- `unit_triangle()` / `unit_triangle_xy()` - 内部実装
- `vertex_a/b/c()` - アクセサ
- `area()` - 面積計算
- `centroid()` - 重心計算
- `circumcenter()` (2D) / `normal()` (3D) - 幾何プロパティ
- `edge_ab/bc/ca()` - 辺ベクトル取得

### トレイト実装の方針

既存メソッドを活用し、トレイトメソッドは以下のように実装:

1. **Constructor**: タプル座標 → Point型 → 既存 new() 呼び出し
2. **Properties**: 既存アクセサ → Point型 → タプル座標に変換
3. **Measure**: 既存メソッド（area, edge_ab().length()）を直接使用

---

## 検証結果

### ビルド・テスト

```bash
✅ cargo build -p geo_primitives
✅ cargo clippy -p geo_primitives -- -D warnings (0 warnings)
✅ cargo test -p geo_primitives (312 tests passed)
```

### 既存テストとの互換性

Triangle2D/3D の既存テストは全て通過:
- `test_triangle_creation` - 三角形の生成
- `test_area_calculation` - 面積計算
- `test_centroid` - 重心計算
- `test_circumcenter` (2D) - 外心計算
- `test_triangle_normal` (3D) - 法線ベクトル
- `test_degenerate_triangle` - 退化三角形の検出

---

## Phase 2/3 での拡張予定

### Phase 2: 標準機能（将来実装）

- 向き判定（clockwise/counterclockwise）
- 点の包含判定（contains_point）
- バリセントリック座標
- 3辺の長さから三角形を構築

### Phase 3: 高度な機能（将来実装）

- 三角形の分割
- 内心・外心・重心の詳細情報
- 角度計算
- 相似・合同判定

---

## まとめ

Triangle2D/3D の Phase 1 Core Traits 実装により:

- ✅ **Foundation Pattern 進捗**: 24/25 形状（96%）完了
- ✅ **3-5-4 パターン**: 一貫して適用
- ✅ **既存実装の尊重**: 内部メソッドを活用
- ✅ **型安全性**: タプル座標での統一インターフェース
- ✅ **テスト**: 既存テスト全てパス

**残り1形状**: Ellipse2D のみ
