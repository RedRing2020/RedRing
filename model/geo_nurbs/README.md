# geo_nurbs - NURBS Curves and Surfaces

**最終更新日**: 2026年1月10日

NURBS（Non-Uniform Rational B-Spline）曲線・曲面の実装クレート。
Foundation Patternに完全準拠し、統一された型安全なインターフェースを提供します。

## 概要

`geo_nurbs`は、CAD/CAMシステムで広く使用されるNURBS曲線・曲面を実装します。
全ての型が**Foundation Pattern（Core + Extension + Transform）**に準拠しており、
`geo_primitives`と同様の統一インターフェースで操作可能です。

## 提供する型

### NurbsCurve2D - 2次元NURBS曲線

- **パラメトリック曲線**: 制御点、ノットベクトル、重みで定義
- **有理・非有理**: 重み付き制御点による円弧・自由曲線の表現
- **2D変換**: Matrix3x3ベースの平行移動・回転・スケール

### NurbsCurve3D - 3次元NURBS曲線

- **空間曲線**: 3次元制御点による自由曲線
- **Bézier互換**: Bézier曲線としての生成・変換
- **3D変換**: Matrix4x4ベースの全3D変換
- **高度な境界ボックス**: 精密計算・適応的サンプリング

### NurbsSurface3D - 3次元NURBS曲面

- **パラメトリック曲面**: 2方向（u, v）の制御点グリッド
- **有理・非有理**: 球面・トーラス等の正確な表現
- **法線計算**: 各点での曲面法線ベクトル
- **面積計算**: 数値積分による曲面面積

## Foundation Pattern 準拠

### Core Traits（基本機能）

すべてのNURBS型は以下のCore Traitsを実装：

```rust
use geo_contracts::{
    NurbsCurve3DConstructor, NurbsCurve3DProperties, NurbsCurve3DMeasure
};

// Constructor - 生成
let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
    degree, knots, control_points, weights
)?;

// Properties - プロパティアクセス
let deg = <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::degree(&curve);
let is_rational = <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::is_rational(&curve);

// Measure - 計量
let point = <NurbsCurve3D<f64> as NurbsCurve3DMeasure<f64>>::evaluate(&curve, 0.5)?;
let length = <NurbsCurve3D<f64> as NurbsCurve3DMeasure<f64>>::arc_length_total(&curve, 1e-6);
```

### Extension Traits（拡張機能）

```rust
use geo_contracts::Bounded;

// PrimitiveKind取得
let kind = curve.primitive_kind(); // PrimitiveKind::NurbsCurve3D

// 境界ボックス
let bbox = curve.aabb()?; // Aabb3D<T>
```

補足:

- `PrimitiveMetadata` と `Bounded` は本体と `*_bounds.rs` に分離して実装する
- 標準 `aabb()` は制御点ベースの保守的な bounds を返す
- より高精度な bounds は `curve_3d_extensions.rs` のような extension 側へ分離する

### Transform Traits（変換操作）

```rust
use analysis::Angle;
use analysis::vector::Vector3;
use geo_nurbs::AnalysisTransform3D;

// 平行移動
let translated = curve.translate_analysis(&Vector3::new(1.0, 2.0, 3.0))?;

// 回転
let axis = Vector3::new(0.0, 0.0, 1.0);
let angle = Angle::from_degrees(90.0);
let rotated = curve.rotate_analysis(&curve, &axis, angle)?;

// スケール
let scaled = curve.uniform_scale_analysis(&curve, 2.0)?;
```

## 使用例

### 2次元NURBS曲線の作成と評価

```rust
use geo_nurbs::NurbsCurve2D;
use geo_contracts::NurbsCurve2DConstructor;

// 制御点（2D座標）
let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
let weights = Some(vec![1.0, 1.0, 1.0]);
let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
let degree = 2;

// NURBS曲線を生成
let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
    control_points, weights, knot_vector, degree
)?;

// パラメータt=0.5での評価
let point = curve.evaluate_at(0.5);
println!("Point at t=0.5: ({}, {})", point.x(), point.y());
```

### 3次元NURBS曲線の変換

```rust
use analysis::Angle;
use analysis::vector::Vector3;
use geo_contracts::NurbsCurve3DConstructor;
use geo_nurbs::{AnalysisTransform3D, NurbsCurve3D};

// 線分を作成
let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
    (0.0, 0.0, 0.0),
    (1.0, 0.0, 0.0)
)?;

// 平行移動
let translation = Vector3::new(5.0, 3.0, 2.0);
let translated = curve.translate_analysis(&translation)?;

// Z軸周りに90度回転
let axis = Vector3::new(0.0, 0.0, 1.0);
let angle = Angle::from_degrees(90.0);
let rotated = translated.rotate_analysis(&translated, &axis, angle)?;
```

### NURBS曲面の作成と法線計算

```rust
use geo_nurbs::NurbsSurface3D;
use geo_contracts::{NurbsSurface3DConstructor, NurbsSurface3DMeasure};

// 単位平面を生成
let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

// パラメータ(u=0.5, v=0.5)での法線ベクトル
let normal = <NurbsSurface3D<f64> as NurbsSurface3DMeasure<f64>>::normal_at(
    &surface, 0.5, 0.5
);
println!("Normal: ({}, {}, {})", normal.0, normal.1, normal.2);

// 曲面の面積
let area = <NurbsSurface3D<f64> as NurbsSurface3DMeasure<f64>>::area(&surface, 10);
```

### 高度な境界ボックス計算（NurbsCurve3Dのみ）

```rust
use geo_nurbs::curve_3d_extensions::AabbOptions;

// 制御点ベース（高速・保守的）
let rough_bbox = curve.bounding_box_with_options(AabbOptions::Rough);

// 精密計算（トレランス指定）
let precise_bbox = curve.bounding_box_with_options(
    AabbOptions::Precise { tolerance: 0.01 }
);

// 適応的サンプリング（分割数指定）
let adaptive_bbox = curve.bounding_box_with_options(
    AabbOptions::Adaptive { max_subdivisions: 500 }
);
```

## bounds 運用ルール

- finite な NURBS 型には `Bounded` を付与する
- `Bounded` 実装は `*_bounds.rs` に配置する
- 2D/3D の同族型を追加する場合、bounds を片側だけで止めない
- `NurbsSurface2D` は現状未定義のため、`NurbsSurface3D` だけに bounds があるのは設計上の現状であり、横展開漏れではない

## モジュール構成

```
geo_nurbs/src/
├── lib.rs                       # モジュール定義
├── error.rs                     # NurbsError型
├── weight_storage.rs            # 重みの内部表現
├── knot.rs                      # ノットベクトル操作
├── basis.rs                     # B-spline基底関数
│
├── curve_2d.rs                  # NurbsCurve2D - Core Traits実装
├── curve_2d_bounds.rs           # NurbsCurve2D - Bounds実装
├── curve_2d_transform.rs        # NurbsCurve2D - Transform実装
│
├── curve_3d.rs                  # NurbsCurve3D - Core Traits実装
├── curve_3d_bounds.rs           # NurbsCurve3D - Bounds実装
├── curve_3d_extensions.rs       # NurbsCurve3D - 拡張機能（境界ボックス等）
├── curve_3d_transform.rs        # NurbsCurve3D - Transform実装
│
├── surface_3d.rs                # NurbsSurface3D - Core Traits実装
├── surface_3d_bounds.rs         # NurbsSurface3D - Bounds実装
├── surface_3d_transform.rs      # NurbsSurface3D - Transform実装
│
└── operations/                  # NURBS編集操作
    ├── degree_elevation.rs      # 次数上昇
    ├── knot_insertion.rs        # ノット挿入
    └── knot_removal.rs          # ノット削除
```

## アーキテクチャ依存関係

```text
analysis → geo_contracts
            ↓
        geo_core (Point2D, Point3D, Aabb2D, Aabb3D)
            ↓
        geo_nurbs ✅（geo_primitives依存なし）
```

- ✅ Foundation Pattern完全準拠
- ✅ `geo_primitives`への依存なし（循環依存回避）
- ✅ 型安全な統一インターフェース

## テスト

```bash
# geo_nurbsクレートのテスト
cargo test -p geo_nurbs

# 特定の型のテスト
cargo test -p geo_nurbs --lib curve_2d
cargo test -p geo_nurbs --lib curve_3d_transform
```

**テスト数**: 64テスト（全て成功）
- NurbsCurve2D: Core + Extension + Transform
- NurbsCurve3D: Core + Extension + Transform + 高度な境界ボックス
- NurbsSurface3D: Core + Extension + Transform

## パフォーマンス

### 境界ボックス計算（NurbsCurve3D）

- **Rough** (制御点ベース): O(n) - 最速、保守的
- **Precise** (トレランス指定): O(n × m) - m はサンプル数
- **Adaptive** (分割数指定): O(n × subdivisions)

### 弧長計算

- **簡易実装**: 一定間隔サンプリング O(n)
- **精度**: トレランス指定による適応的サンプリング

## 今後の拡張

- [ ] `SafeTransform` trait実装（エラーハンドリング版）
- [ ] NURBS曲線・曲面の交差判定
- [ ] NURBS-Primitives間の衝突判定
- [ ] 弧長パラメータ化（arc-length parameterization）
- [ ] NURBS曲線の分割（split）操作

## 関連ドキュメント

- **実装記録**: `dev/architecture/NURBS_FOUNDATION_PATTERN.md`
- **アーキテクチャ**: `dev/architecture/ARCHITECTURE.md`
- **Foundation Pattern**: `dev/foundation/FOUNDATION_CORE_TRAITS_REDESIGN_METHODOLOGY.md`

## 参考文献

- "The NURBS Book" (Les Piegl, Wayne Tiller)
- ISO 10303-42 (STEP AP242) - Geometric and topological representation

---

**Issue**: #194  
**PR**: #197  
**最終更新**: 2026年1月10日
