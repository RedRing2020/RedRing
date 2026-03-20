# RECT World/View 座標分離設計（Issue #9）

## 目的

- `Rect2D` / `Rect3D` を **World 座標の幾何プリミティブ**として実装する。
- `SelectionRect` は **画面座標の入力・表示 state** に限定し、幾何演算責務を持たせない。
- Foundation Pattern（Core Traits + Primitive 実装 + Foundation + Transform）に準拠する。

## 設計方針

### 1. WorldRect（model側）

- `Rect2D<T>`: 2D軸平行矩形（原点 + 幅 + 高さ）
- `Rect3D<T>`: 任意平面上の矩形（原点 + `u_axis` + `v_axis` + 幅 + 高さ）
  - `u_axis` と `v_axis` は直交正規化して保持
  - 法線は `u_axis × v_axis`

### 2. SelectionRect（view側）

- 画面ピクセル座標の state（`x, y, width, height`）
- 役割はドラッグ範囲の可視化と入力中間表現のみ
- 包含判定や幾何学的意味付けは WorldRect 側で実施

### 3. 画面→ワールドの橋渡し

- `SelectionRect` からレイ生成/平面交点を通して `Rect3D` へ変換
- 変換責務は ViewModel/Camera 側に置く

## Foundation Pattern 実装範囲

- `geo_contracts/src/geometry/core/rectangle_traits.rs`
  - `Rect2DConstructor<T>` / `Rect2DProperties<T>` / `Rect2DMeasure<T>`
  - `Rect3DConstructor<T>` / `Rect3DProperties<T>` / `Rect3DMeasure<T>`
- `geo_primitives/src/rectangle_2d.rs`（Core）
- `geo_primitives/src/rectangle_2d_foundation.rs`（ExtensionFoundation）
- `geo_primitives/src/rectangle_2d_transform.rs`（AnalysisTransform2D）
- `geo_primitives/src/rectangle_3d.rs`（Core）
- `geo_primitives/src/rectangle_3d_foundation.rs`（ExtensionFoundation）
- `geo_primitives/src/rectangle_3d_transform.rs`（AnalysisTransform3D）

## API 最小仕様

### Rect2D

- `new(origin, width, height)`
- `resize(width, height)`
- `contains_point(point)`（境界含む）
- `area()`, `perimeter()`

### Rect3D

- `new(origin, u_axis, v_axis, width, height)`
- `resize(width, height)`
- `contains_point(point, tolerance)`（平面距離 + UV範囲で判定）
- `area()`, `normal()`, `corners()`

## 非ゴール

- SelectionRect の描画実装・入力イベント統合は本Issueでは必須外
- 画面→ワールド変換の最適化（SIMD等）は後続

## 検証

- 単体テスト: contains / resize / 面積 / 角点
- `cargo test --workspace`
- `cargo clippy -- -D warnings`

