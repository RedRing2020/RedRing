# ViewModel層アーキテクチャ設計

**作成日**: 2026年2月8日  
**関連Issue**: #204 - 形状可視化システム完成  
**目的**: ViewModelの適切な責務範囲の明確化と依存関係の整理

---

## 🎯 設計原則

### MVVMアーキテクチャにおけるViewModel層の責務

```text
View層 (GPU描画)
  ↑ GPU頂点データ ([f32; 3])
ViewModel層 (変換ロジック)
  ↑ トレイト経由アクセス
Model層 (幾何計算)
```

**ViewModel層の適切な責務**:
- ✅ **データ形式変換**: Model層の型 → View層のGPU形式
- ✅ **品質パラメータ管理**: テッセレーション品質設定
- ✅ **バッチ処理**: 複数形状の一括変換
- ❌ **幾何計算**: 法線計算、パラメトリック評価（→ Model層の責務）
- ❌ **形状構築**: 形状オブジェクトの生成（→ Model層の責務）

---

## 📊 現状の問題点（Issue #204実装での発見）

### 問題1: 不適切な幾何計算の実行

**現状の実装（mesh_converter.rs）**:
```rust
// ❌ ViewModel層で法線計算を実行
let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());
let normal = edge1.cross(&edge2).normalize();  // 👈 幾何演算
```

**問題点**:
- ベクトル演算はModel層の責務
- 法線計算ロジックの重複（Model層にも同様の実装がある可能性）
- テスト容易性の低下

**正しい設計**:
```rust
// ✅ Model層で法線を計算して返却
use geo_foundation::Triangle3DGeometry;

let normal = triangle.compute_normal();  // Model層のメソッド
let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];
```

### 問題2: パラメトリック評価の重複実装

**現状の実装（shape_converter.rs, Issue #204）**:
```rust
// ❌ ViewModel層で独自のヘルパー関数を実装
#[allow(clippy::too_many_arguments)]
fn calculate_cylinder_point(
    center: &Point3D<f64>,
    x_axis: &Vector3D<f64>,
    y_axis: &Vector3D<f64>,
    z_axis: &Vector3D<f64>,
    radius: f64,
    u: f64,
    v: f64,
) -> Point3D<f64> {
    // 円筒面のパラメトリック方程式を実装
    // ... 30行のコード ...
}
```

**問題点**:
- `CylindricalSurface3D::point_at_uv()` と完全に重複
- 10個のヘルパー関数（全1000行以上）が重複実装
- 引数過多でClippy警告（8-9引数）

**geo_primitivesの既存メソッド**:
```rust
// ✅ Model層に既に実装済み
impl CylindricalSurface3D<f64> {
    pub fn point_at_uv(&self, u: f64, v: f64) -> Point3D<f64> { ... }
    pub fn normal_at_uv(&self, u: f64, v: f64) -> Vector3D<f64> { ... }
}
```

**しかし、Foundation Pattern遵守の問題**:
```rust
// ❌ geo_primitives直接呼び出しは設計違反
let point = surface.point_at_uv(u, v);  // 具象型への依存
```

### 問題3: Foundation Traitsのタプル型返却問題

**現状の設計**:
```rust
// geo_foundation/src/core/cylindrical_surface_traits.rs
pub trait CylindricalSurface3DMeasure<T: Scalar> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);  // タプル型
}
```

**ViewModelでの使用**:
```rust
// タプルから型安全な構造体への変換が必要
let point_tuple = surface.point_at_uv(u, v);  // (f64, f64, f64)
let point = Point3D::new(point_tuple.0, point_tuple.1, point_tuple.2);  // 👈 冗長

// さらに演算が必要な場合
let vector = Vector3D::new(...);  // geo_primitives型の構築
let result = point.distance(&other);  // 👈 geo_primitives依存
```

**課題**:
- タプル ↔ 構造体の変換で `geo_primitives` への依存が発生
- 型安全性の喪失（タプルは何の座標かわからない）

---

## ✅ 解決策

### Solution 1: Foundation Traitsに関連型メソッドを追加

**提案**: タプル版と構造体版を併存させる

```rust
// geo_foundation/src/core/parametric_surface.rs (新規)
pub trait ParametricSurface3D<T: Scalar> {
    // 既存: タプル版（後方互換性のため残す）
    fn point_at_uv_tuple(&self, u: T, v: T) -> (T, T, T);
    fn normal_at_uv_tuple(&self, u: T, v: T) -> (T, T, T);
    
    // 新規: 関連型版（型安全）
    type Point;
    type Vector;
    fn point_at_uv(&self, u: T, v: T) -> Self::Point;
    fn normal_at_uv(&self, u: T, v: T) -> Self::Vector;
}

// geo_primitives での実装
impl ParametricSurface3D<f64> for CylindricalSurface3D<f64> {
    type Point = Point3D<f64>;
    type Vector = Vector3D<f64>;
    
    fn point_at_uv(&self, u: f64, v: f64) -> Self::Point {
        // 既存の実装を使用
        self.point_at_uv_impl(u, v)
    }
    
    fn normal_at_uv(&self, u: f64, v: f64) -> Self::Vector {
        self.normal_at_uv_impl(u, v)
    }
}
```

**メリット**:
- ViewModel層は `geo_foundation` トレイトのみに依存
- 型安全性を保ちながらFoundation Pattern遵守
- 既存コードへの影響最小（タプル版は残す）

**デメリット**:
- トレイトが複雑化
- ViewModel層で関連型を扱う必要がある

### Solution 2: geo_coreにPoint3D/Vector3D抽象型を定義（非推奨）

**理由**:
- `geo_core` は「低レイヤー基本型」（Aabb等）の責務
- `Point3D/Vector3D` は `geo_primitives` の基本実装として適切
- アーキテクチャの責務混乱を招く

### Solution 3: Model層にテッセレーション機能を提供

**提案**: `geo_algorithms` に曲面メッシュ化機能を追加

```rust
// geo_algorithms/src/tessellation/mod.rs (新規)
pub struct TessellationParams {
    pub u_divisions: usize,
    pub v_divisions: usize,
}

pub struct SurfaceMesh {
    pub vertices: Vec<Point3D<f64>>,
    pub normals: Vec<Vector3D<f64>>,
    pub indices: Vec<u32>,
}

pub fn tessellate_cylindrical_surface(
    surface: &CylindricalSurface3D<f64>,
    params: &TessellationParams,
) -> SurfaceMesh {
    // パラメトリック評価と三角形分割
}

// 全15形状に対応する関数を提供
```

**ViewModelでの使用**:
```rust
// viewmodel/converter/src/shape_converter.rs
use geo_algorithms::tessellation;

pub fn cylindrical_surface_to_vertices(
    surface: &CylindricalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let params = TessellationParams {
        u_divisions: quality.circle_segments,
        v_divisions: quality.sphere_v_divisions,
    };
    
    let mesh = tessellation::tessellate_cylindrical_surface(surface, &params);
    
    // GPU形式への変換のみ実行（ViewModelの責務）
    mesh.vertices.iter().zip(mesh.normals.iter())
        .map(|(p, n)| VertexData::new(
            [p.x() as f32, p.y() as f32, p.z() as f32],
            [n.x() as f32, n.y() as f32, n.z() as f32],
        ))
        .collect()
}
```

**メリット**:
- ViewModel層の責務が明確（GPU形式変換のみ）
- テッセレーションロジックの再利用性
- geo_algorithms経由でModel層にアクセス（Foundation Pattern遵守）

**デメリット**:
- geo_algorithmsの実装コスト
- 新規モジュールの追加

---

## 🎯 推奨アプローチ（段階的実装）

### Phase 1: 即座の問題修正（Issue #204完了のため）

**方針**: 最小限の変更でFoundation Pattern遵守

```rust
// viewmodel/converter/src/shape_converter.rs

// ❌ 削除: 独自ヘルパー関数（10個、1000行以上）
// fn calculate_cylinder_point(...) { ... }
// fn calculate_sphere_point(...) { ... }
// ...

// ✅ Foundation Traits経由でパラメトリック評価
use geo_foundation::{CylindricalSurface3DMeasure, SphericalSurface3DMeasure, ...};

pub fn cylindrical_surface_to_vertices(
    surface: &CylindricalSurface3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let mut vertices = Vec::new();
    
    for i in 0..v_divisions {
        for j in 0..u_divisions {
            let u = (j as f64 / u_divisions as f64) * 2.0 * PI;
            let v = (i as f64 / v_divisions as f64) * height;
            
            // ✅ トレイト経由でパラメトリック評価
            let (px, py, pz) = <CylindricalSurface3D<f64> as CylindricalSurface3DMeasure<f64>>::point_at_uv(surface, u, v);
            let (nx, ny, nz) = <CylindricalSurface3D<f64> as CylindricalSurface3DMeasure<f64>>::normal_at(surface, u, v);
            
            // GPU形式への変換
            vertices.push(VertexData::new(
                [px as f32, py as f32, pz as f32],
                [nx as f32, ny as f32, nz as f32],
            ));
        }
    }
    
    vertices
}
```

**影響範囲**:
- shape_converter.rs: ヘルパー関数削除（-1000行）、トレイト呼び出しに変更（+200行）
- Clippy警告の完全解消
- Foundation Pattern完全遵守

### Phase 2: Foundation Traitsの改善（将来実装）

**内容**: Solution 1を実装して型安全性向上

```rust
// geo_foundation/src/core/parametric_surface.rs
pub trait ParametricSurface3D<T: Scalar> {
    type Point;
    type Vector;
    fn point_at_uv(&self, u: T, v: T) -> Self::Point;
    fn normal_at_uv(&self, u: T, v: T) -> Self::Vector;
}
```

**タイミング**: Issue #204完了後、別Issueで実施

### Phase 3: テッセレーション機能の分離（将来実装）

**内容**: Solution 3を実装してViewModel責務を明確化

```rust
// geo_algorithms/src/tessellation/
tessellate_cylindrical_surface()
tessellate_spherical_surface()
...
```

**タイミング**: Phase 2完了後、パフォーマンス最適化時

---

## 📋 依存関係の整理

### 修正前（Issue #204実装）

```text
viewmodel/converter
├── geo_foundation (Properties traits)  ✅ 正常
├── geo_primitives (全15形状 + Point3D/Vector3D)  ⚠️ 直接依存
├── geo_algorithms  ✅ 正常
├── geo_io  ✅ 正常
└── analysis  ✅ 正常
```

**問題**:
- geo_primitives具象型への直接依存（Foundation Pattern違反）
- パラメトリック評価を独自実装（重複コード）

### 修正後（Phase 1完了時）

```text
viewmodel/converter
├── geo_foundation (Properties + Measure traits)  ✅ トレイト経由のみ
├── geo_primitives (Point3D/Vector3D型宣言のみ)  ✅ タプル変換用
├── geo_algorithms  ✅ 正常
├── geo_io  ✅ 正常
└── analysis  ✅ 正常
```

**改善点**:
- トレイト経由のパラメトリック評価
- 重複コード削除
- Foundation Pattern遵守

### NURBS GPU評価データの責務整理（Issue #210）

**方針**:
- ViewModel層は**geo_nurbsに直接依存しない**
- NURBS曲線・曲面の制御点/重み/ノット取得は**Foundationトレイト経由**で行う
- App層は**geo_*クレートに直接依存しない**（viewmodel経由で評価データ生成）

**実装イメージ**:
```text
viewmodel/converter
├── geo_foundation (NURBSトレイト拡張: 制御点/重み/ノット取得)
├── geo_core (低レイヤー型のみ)
├── geo_primitives (必要最小限)
├── geo_algorithms (許可されるがA方針では使用しない)
└── analysis

view/app
├── viewmodel (評価データ生成を委譲)
└── render/stage/graphics/analysis
```

**補足**:
- geo_foundationのNURBSトレイト拡張はFoundationパターンの修正に該当
- 既存の依存ルールに従い、**ViewModel→geo_algorithms**は許可されるが本方針では採用しない

### 理想形（Phase 3完了時）

```text
viewmodel/converter
├── geo_foundation (トレイトのみ)  ✅
├── geo_algorithms (tessellation module)  ✅ Model層経由
└── analysis  ✅
```

**完全な責務分離**:
- ViewModel層: GPU形式変換のみ
- Model層: 全ての幾何計算

---

## 🔧 実装ガイドライン

### ViewModel層で許可される操作

```rust
// ✅ 許可: GPU形式への型変換
let position = [x as f32, y as f32, z as f32];

// ✅ 許可: Foundation Traits経由のプロパティアクセス
let radius = <Circle3D<f64> as Circle3DProperties<f64>>::radius(&circle);

// ✅ 許可: Foundation Traits経由のパラメトリック評価
let (px, py, pz) = <Surface as SurfaceMeasure<f64>>::point_at_uv(&surface, u, v);

// ❌ 禁止: 幾何演算
let normal = edge1.cross(&edge2);  // Model層で実行すべき

// ❌ 禁止: 形状構築
let circle = Circle3D::new(...);  // Model層またはgeo_ioの責務

// ❌ 禁止: 具象型メソッドの直接呼び出し
let point = surface.point_at_uv(u, v);  // トレイト経由にすべき
```

### Foundation Traits呼び出しパターン

```rust
// パターン1: 明示的なトレイト境界（推奨）
use geo_foundation::CylindricalSurface3DMeasure;

pub fn convert<T>(surface: &T, u: f64, v: f64) -> VertexData
where
    T: CylindricalSurface3DMeasure<f64>,
{
    let (px, py, pz) = surface.point_at_uv(u, v);
    VertexData::new([px as f32, py as f32, pz as f32], ...)
}

// パターン2: 完全修飾構文（型推論が困難な場合）
let point = <CylindricalSurface3D<f64> as CylindricalSurface3DMeasure<f64>>::point_at_uv(surface, u, v);
```

---

## 📊 コード削減効果（Phase 1実装時）

| カテゴリ | 削減前 | 削減後 | 削減量 |
|----------|--------|--------|--------|
| ヘルパー関数 | 10個 | 0個 | **-10** |
| コード行数 | ~1,200行 | ~400行 | **-800行** |
| Clippy警告 | 3件 | 0件 | **-3件** |
| テスト維持コスト | 高 | 低 | **改善** |

---

## 🎯 成功基準

### Phase 1完了時

- [ ] 全ヘルパー関数削除（calculate_*_point, calculate_*_normal）
- [ ] Foundation Traits経由のパラメトリック評価
- [ ] Clippy警告ゼロ
- [ ] 既存テスト全合格
- [ ] アーキテクチャチェックスクリプト合格

### Phase 2完了時

- [ ] ParametricSurface3Dトレイト実装
- [ ] タプル↔構造体変換の最小化
- [ ] 型安全性の向上

### Phase 3完了時

- [ ] ViewModel層のコードが500行以下
- [ ] geo_primitivesへの依存が型宣言のみ
- [ ] 全ての幾何計算がModel層に集約

---

## 🔗 関連ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 全体アーキテクチャ
- [SHAPE_TESSELLATION_DESIGN.md](./SHAPE_TESSELLATION_DESIGN.md) - テッセレーション設計
- [FOUNDATION_PATTERN.md](../foundation/) - Foundation Pattern詳細
- [Issue #204](https://github.com/RedRing2020/RedRing/issues/204) - 形状可視化システム完成
