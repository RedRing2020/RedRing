# Point3D Refactoring Plan - analysis::Point3 統合計画

**作成日**: 2025年12月21日  
**最終更新**: 2025年12月21日（Phase 3完了）  
**関連Issue**: [#166](https://github.com/RedRing2020/RedRing/issues/166)  
**優先度**: High（今後の機能拡張での手戻り防止のため優先実施）  
**ステータス**: ✅ **Phase 3完了** - 87箇所のanalysis::Point3置換完了、全テスト通過（721 tests）

## 📋 目的

`analysis::Point3` と `geo_primitives::Point3D` の二重実装を解消し、`geo_core::Point3D` に統合することで：
- ✅ 変換コストの削減（Point3D ⇄ analysis::Point3 不要化）
- ✅ インターフェースの統一
- ✅ レイヤー設計の整合性確保（geo_core が基本型を提供）
- ✅ 今後の機能拡張での無駄な手戻り防止

---

## 🔍 現状分析（2025年12月21日実施）

### 1. analysis::Point3 使用状況

**総使用箇所**: 約87箇所

#### 使用パターン別内訳

1. **geo_core::Aabb3D** (17箇所)
   - フィールド定義: `min: analysis::Point3<T>`, `max: analysis::Point3<T>`
   - メソッドシグネチャ: `new()`, `from_points()`, `contains()`, `min()`, `max()`, `center()`
   - テストコード: 8箇所

2. **Foundation の bounding_box() 実装** (約50箇所)
   ```rust
   // 典型的なパターン
   fn bounding_box(&self) -> Self::BBox {
       geo_core::Aabb3D::new(
           analysis::Point3::new(min_x, min_y, min_z),
           analysis::Point3::new(max_x, max_y, max_z),
       )
   }
   ```
   
   影響ファイル:
   - `*_foundation.rs`: Circle3D, Triangle3D, Arc3D, Ellipse3D, Spherical*, Cylindrical*, Conical*, Torus*, TriangleMesh3D, EllipsoidalSurface3D

3. **Transform 実装の変換ヘルパー** (約20箇所)
   ```rust
   pub fn point_to_analysis_vector<T: Scalar>(point: Point3D<T>) -> Vector3<T> {
       Vector3::new(point.x(), point.y(), point.z())
   }
   ```
   
   影響ファイル:
   - `conical_solid_3d_transform.rs`
   - `conical_surface_3d_transform.rs`
   - `cylindrical_solid_3d_transform.rs`
   - `cylindrical_surface_3d_transform.rs`

### 2. analysis::Plane3 使用状況

**調査結果**: ✅ **削除不要**

- **analysis クレート内部のみで使用** (4箇所)
  - `geometry/plane3.rs`: 定義と実装
  - `geometry/mod.rs`: 再エクスポート
- **model 層では一切参照されていない**
  
**結論**: analysis の内部実装として残す（外部からは使用されていない）

### 3. geo_primitives::Point3D の実装状況

**ファイル**: `model/geo_primitives/src/point_3d.rs` (377行)

**実装内容**:
- ✅ 基本構造体定義 (`struct Point3D<T: Scalar>`)
- ✅ コンストラクタ (new, origin, from_tuple, from_spherical)
- ✅ アクセサ (x, y, z, coords)
- ✅ 計算メソッド (distance_to, norm, lerp, midpoint)
- ✅ 演算子オーバーロード (Add, Sub)
- ✅ Foundation トレイト実装 (Point3DConstructor, Properties, Measure, Core)
- ✅ Analysis 変換機能 (to_analysis_point3, from_analysis_point3)
- ✅ From trait 実装

**関連ファイル**:
- `point_3d_foundation.rs`: ExtensionFoundation 実装
- `point_3d_extensions.rs`: 拡張機能
- `point_3d_tests.rs`: テストスイート

---

## 🎯 実装方針（確定版）

### 基本方針

1. **geo_primitives::Point3D から geo_core へ実装を移動**
   - 基本実装のみを geo_core に配置
   - Foundation 拡張は geo_primitives に残す

2. **analysis::Point3 の扱い**
   - 完全削除は**しない**
   - analysis クレート内部専用として残す
   - 外部（model層）からは geo_core::Point3D を使用

3. **段階的移行**
   - 87箇所を一度に変更せず、Phase 分けで実施
   - 各 Phase でテスト実行・検証

---

## 📝 実装計画（4 Phase）

### Phase 1: geo_core::Point3D 基本実装（1日）

#### 1-1. 新規ファイル作成

**ファイル**: `model/geo_core/src/point_3d.rs`

**移動する実装**:
```rust
// 構造体定義
pub struct Point3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

// 基本コンストラクタ
impl<T: Scalar> Point3D<T> {
    pub fn new(x: T, y: T, z: T) -> Self { ... }
    pub fn origin() -> Self { ... }
    pub fn from_tuple(coords: (T, T, T)) -> Self { ... }
    
    // アクセサ
    pub fn x(&self) -> T { ... }
    pub fn y(&self) -> T { ... }
    pub fn z(&self) -> T { ... }
    pub fn coords(&self) -> [T; 3] { ... }
    
    // 基本計算
    pub fn distance_to(&self, other: &Self) -> T { ... }
    pub fn distance_squared_to(&self, other: &Self) -> T { ... }
    pub fn norm(&self) -> T { ... }
    pub fn norm_squared(&self) -> T { ... }
    pub fn midpoint(&self, other: &Self) -> Self { ... }
    pub fn lerp(&self, other: &Self, t: T) -> Self { ... }
}

// 演算子オーバーロード
impl<T: Scalar> Sub for Point3D<T> { ... }
impl<T: Scalar> Add<Vector3D<T>> for Point3D<T> { ... }
impl<T: Scalar> Sub<Vector3D<T>> for Point3D<T> { ... }

// From trait
impl<T: Scalar> From<(T, T, T)> for Point3D<T> { ... }
```

**必要な依存関係**:
```toml
# geo_core/Cargo.toml に追加
[dependencies]
analysis = { path = "../../foundation/analysis" }
geo_foundation = { path = "../geo_foundation" }
```

#### 1-2. geo_core への組み込み

**ファイル**: `model/geo_core/src/lib.rs`

```rust
pub mod point_3d;
pub use point_3d::Point3D;
```

#### 1-3. テスト作成

基本機能のテストを `point_3d.rs` 内に記載（または別ファイル化）

#### 1-4. 検証

```bash
cargo build -p geo_core
cargo test -p geo_core
```

---

### Phase 2: geo_primitives リファクタリング（1日）

#### 2-1. point_3d.rs の軽量化

**ファイル**: `model/geo_primitives/src/point_3d.rs`

**変更内容**:
```rust
// geo_core から再エクスポート
pub use geo_core::Point3D;

// Foundation トレイト実装は維持
use geo_foundation::{
    core::{
        point_core_traits::{Point3DConstructor, Point3DCore, Point3DMeasure, Point3DProperties},
        point_traits,
    },
    Scalar,
};

impl<T: Scalar> Point3DConstructor<T> for Point3D<T> { ... }
impl<T: Scalar> Point3DProperties<T> for Point3D<T> { ... }
impl<T: Scalar> Point3DMeasure<T> for Point3D<T> { ... }
impl<T: Scalar> Point3DCore<T> for Point3D<T> {}

// Analysis 変換機能は維持
impl<T: Scalar> Point3D<T> {
    pub fn to_analysis_point3(&self) -> analysis::linalg::point3::Point3<T> { ... }
    pub fn from_analysis_point3(p: analysis::linalg::point3::Point3<T>) -> Self { ... }
}

impl<T: Scalar> From<analysis::linalg::point3::Point3<T>> for Point3D<T> { ... }
impl<T: Scalar> From<Point3D<T>> for analysis::linalg::point3::Point3<T> { ... }
```

#### 2-2. geo_primitives の依存関係更新

**ファイル**: `model/geo_primitives/Cargo.toml`

```toml
[dependencies]
geo_core = { path = "../geo_core" }  # 追加
analysis = { path = "../../foundation/analysis" }
geo_foundation = { path = "../geo_foundation" }
```

#### 2-3. 既存ファイルの維持

以下のファイルはそのまま維持:
- `point_3d_foundation.rs`: ExtensionFoundation 実装
- `point_3d_extensions.rs`: 拡張機能
- `point_3d_tests.rs`: テストスイート

#### 2-4. 検証

```bash
cargo build -p geo_primitives
cargo test -p geo_primitives
cargo test --workspace  # 全体テスト
```

---

### Phase 3: analysis::Point3 置換（2-3日）

#### 3-1. geo_core::Aabb3D 修正（優先度: 最高）

**ファイル**: `model/geo_core/src/aabb_3d.rs`

**変更内容**:
```rust
// Before
use analysis::abstract_types::Scalar;

pub struct Aabb3D<T: Scalar> {
    min: analysis::Point3<T>,
    max: analysis::Point3<T>,
}

// After
use analysis::abstract_types::Scalar;
use crate::Point3D;  // geo_core::Point3D を使用

pub struct Aabb3D<T: Scalar> {
    min: Point3D<T>,
    max: Point3D<T>,
}
```

**影響箇所**: 17箇所
- フィールド定義: 2箇所
- メソッドシグネチャ: 7箇所
- テストコード: 8箇所

#### 3-2. Foundation の bounding_box() 修正（約50箇所）

**パターン1**: 直接的な変換
```rust
// Before
fn bounding_box(&self) -> Self::BBox {
    geo_core::Aabb3D::new(
        analysis::Point3::new(min_x, min_y, min_z),
        analysis::Point3::new(max_x, max_y, max_z),
    )
}

// After
fn bounding_box(&self) -> Self::BBox {
    use geo_core::Point3D;
    geo_core::Aabb3D::new(
        Point3D::new(min_x, min_y, min_z),
        Point3D::new(max_x, max_y, max_z),
    )
}
```

**影響ファイル**（優先順に実施）:
1. `circle_3d_foundation.rs`
2. `triangle_3d_foundation.rs`
3. `arc_3d_foundation.rs`
4. `spherical_surface_3d_foundation.rs`
5. `spherical_solid_3d_foundation.rs`
6. `cylindrical_surface_3d_foundation.rs`（Phase 3 で追加）
7. `cylindrical_solid_3d_foundation.rs`（Phase 3 で追加）
8. `conical_surface_3d_foundation.rs`（Phase 3 で追加）
9. `conical_solid_3d_foundation.rs`（Phase 3 で追加）
10. その他（EllipseArc3D, Torus*, TriangleMesh3D, EllipsoidalSurface3D）

#### 3-3. Transform ヘルパー関数の削除/修正（約20箇所）

**不要になる関数**:
```rust
// Before: 変換ヘルパーが必要
pub fn point_to_analysis_vector<T: Scalar>(point: Point3D<T>) -> Vector3<T> {
    Vector3::new(point.x(), point.y(), point.z())
}

// After: geo_core::Point3D は直接 Vector3D と演算可能
// → ヘルパー関数不要（削除）
```

**影響ファイル**:
- `conical_solid_3d_transform.rs`
- `conical_surface_3d_transform.rs`
- `cylindrical_solid_3d_transform.rs`
- `cylindrical_surface_3d_transform.rs`

#### 3-4. 段階的検証

各ファイル修正後に個別テスト:
```bash
cargo test -p geo_primitives --lib -- <shape_name>
```

全ファイル修正後に全体テスト:
```bash
cargo test --workspace
cargo clippy --all -- -D warnings
```

---

### Phase 4: 最終クリーンアップ（1日）

#### 4-1. analysis::Point3 の役割明確化

**ドキュメント更新**: `foundation/analysis/src/linalg/point3.rs`

```rust
//! 3次元点の数値計算実装
//!
//! **注意**: この型は analysis クレート内部専用です。
//! 幾何計算層では `geo_core::Point3D` を使用してください。
//!
//! 数学的な3次元点（位置）を表す型
//! Vector3との相互変換とトレイト共通化を提供
```

#### 4-2. 全体テスト実行

```bash
# 全ワークスペース
cargo build --workspace
cargo test --workspace
cargo clippy --all -- -D warnings
cargo fmt --all -- --check

# テスト実行時間測定
cargo test --workspace --release
```

#### 4-3. ドキュメント更新

以下のドキュメントを更新:
- `dev/architecture/ARCHITECTURE.md`: Point3D の配置を明記
- `README.md`: 必要に応じて更新
- Issue #166 にコメント: 完了報告

#### 4-4. 完了チェックリスト

- [ ] geo_core::Point3D 実装完了
- [ ] geo_primitives リファクタリング完了
- [ ] 87箇所の analysis::Point3 置換完了
- [ ] 全テスト成功（660+ tests）
- [ ] Clippy 警告 0件
- [ ] フォーマット適用
- [ ] ドキュメント更新
- [ ] Issue #166 完了報告

---

## ⚠️ リスク管理

### 高リスク項目

1. **geo_core::Aabb3D の変更**
   - 影響範囲: 全 Foundation 実装
   - 軽減策: 最初に修正し、早期検証

2. **Transform 実装の変更**
   - 影響範囲: Phase 3 で追加された形状
   - 軽減策: ヘルパー関数の慎重な削除

3. **型推論エラーの可能性**
   - 原因: Point3D の再エクスポート
   - 軽減策: 明示的な型注釈追加

### 低リスク項目

1. **analysis::Plane3**
   - 外部依存なし、変更不要

2. **Foundation トレイト実装**
   - geo_primitives で維持、影響小

---

## 📊 期待される効果

### 定量的効果

- **変換コスト削減**: 87箇所の `Point3D ⇄ analysis::Point3` 変換が不要化
- **コード削減**: 変換ヘルパー関数 約20箇所削除
- **ビルド時間**: 変化なし（既存コードの再配置のみ）

### 定性的効果

- ✅ レイヤー設計の整合性確保
- ✅ 今後の機能拡張での手戻り防止
- ✅ インターフェース統一による保守性向上
- ✅ 新規開発者の理解容易化

---

## 🔗 関連リンク

- **Issue**: [#166 geo_core::Point3D新規作成によるanalysis::Point3統合計画](https://github.com/RedRing2020/RedRing/issues/166)
- **プロジェクト構造方針**: `.github/PROJECT_STRUCTURE_POLICY.md`
- **アーキテクチャ**: `dev/architecture/ARCHITECTURE.md`
- **Phase 3 完了報告**: `dev/foundation/PHASE3_COMPLETION_REPORT.md`

---

## 📅 実施スケジュール

- **Phase 1**: ✅ 2025年12月21日完了 - geo_core::Point3D 基本実装
- **Phase 2**: ✅ 2025年12月21日完了 - Point3D/Vector3D完全移行
- **Phase 3**: ✅ 2025年12月21日完了 - analysis::Point3 置換（87箇所）
- **Phase 4**: 予定 - 最終クリーンアップ

**合計**: Phase 1-3 を1日で完了（Issue #166 の見積もり: 5-7日）

---

## ✅ Phase 3 完了報告（2025年12月21日）

### 実施内容

1. **geo_core::Aabb3D 修正** (17箇所)
   - フィールド定義: `min/max: analysis::Point3<T>` → `Point3D<T>`
   - 全メソッドをメソッド呼び出しに変更（`.x` → `.x()`）
   - テストコード修正完了

2. **geo_primitives Foundation修正** (18箇所)
   - 8ファイルの `bounding_box()` 実装修正
   - `analysis::Point3::new()` → `Point3D` 直接使用
   - テストコード内の assert修正

3. **geo_primitives Extensions修正** (50箇所)
   - `use analysis::Point3` 削除
   - 全`Point3::new()`を`geo_core::Point3D::new()`に置換
   - bounding_box計算ロジックの統一化

4. **geo_nurbs修正** (2箇所)
   - `use analysis::Point3` → `use geo_core::Point3D`
   - テストコード修正

### 変更ファイル一覧

**geo_core** (1ファイル):
- `aabb_3d.rs` - 17箇所修正

**geo_primitives** (20ファイル):
- `*_foundation.rs`: arc_3d, circle_3d, spherical_solid_3d, spherical_surface_3d, triangle_3d, triangle_mesh_3d, torus_solid_3d, torus_surface_3d
- `*.rs`: conical_solid_3d, conical_surface_3d, cylindrical_solid_3d, cylindrical_surface_3d, ellipsoidal_surface_3d, spherical_solid_3d, spherical_surface_3d, ellipse_arc_3d
- `*_extensions.rs`: cylindrical_surface_3d, ellipse_arc_3d, line_segment_3d, ray_3d

**geo_nurbs** (2ファイル):
- `curve_3d_foundation.rs`, `curve_3d_extensions.rs`

**合計**: 23ファイル、87箇所の修正

### 検証結果

- ✅ **ビルド成功**: 全クレートビルド成功
- ✅ **テスト成功**: 721 tests passed
- ✅ **geo_core**: 113 tests passed
- ✅ **geo_primitives**: 353 tests passed
- ✅ **geo_nurbs**: テスト通過

### Phase 3 で判明した事項

- Transform変換ヘルパー関数は現時点では削除不要
- geo_core::Point3Dの機能が充実すれば将来的に不要になる可能性
- 全ての`analysis::Point3`使用箇所を`geo_core::Point3D`に統一完了

---

**作成者**: GitHub Copilot  
**最終更新**: 2025年12月21日  
**ステータス**: Phase 3 完了 → Phase 4 実施待ち
