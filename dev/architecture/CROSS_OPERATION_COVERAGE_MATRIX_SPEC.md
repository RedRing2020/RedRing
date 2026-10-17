# Cross Operation Coverage Matrix Spec (v2.2)

対象Issue: #364（v1.0）、#855（v2.0）
関連: #343, #350, #351, #366

## 1. 目的

- 2D/3D の形状の組み合わせごとの演算（衝突判定・交差・距離）の網羅表を、実装と一致した状態で保つ。
- 実装済みの組み合わせは trait の実装としてコンパイル時に確かめ、結果が誤った簡易実装を網羅表の実装済みに含めない。
- 未対応の組み合わせを明示し、系統ごとの実装のタスクの起点にする。

## 2. 範囲

- 演算: 衝突判定（`CrossCollision`）、交差（`CrossIntersection`）、距離（`CrossDistance`）
- 次元: 2D、3D
- 対象: `geo_primitives` と `geo_nurbs` の形状の組み合わせ

範囲外:
- 1 つの形状と点の距離。各形状の trait定義（`*Distance::distance_to_point`）で扱い、網羅表には含めない
- 個別のアルゴリズムの高精度化

## 3. trait の配置と実装の規則

- trait は `geo_algorithms::operations` に置く（`geo_algorithms` から再エクスポートする）。`geo_primitives` と `geo_nurbs` の形状の組み合わせを、どちらのクレートにも依存する `geo_algorithms` で実装するため（orphan rule）
- 形状は参照、点は値ではなく相手の形状として参照で渡す（`Other` が `Point3D` の場合も `&Point3D`）
- 代表の向き（A-B）を実装し、対称な組み合わせの逆の向き（B-A）は代表の向きを呼ぶ
- 衝突判定は「意味論の上での距離が `tolerance` 以内か」とする。交差を実装した組み合わせの衝突判定は、交差の有無で判定する
- 曲線（線分・円・円弧・楕円等）は曲線、領域・立体（三角形・球・円錐等）は領域として扱う（`GEOMETRY_SHAPE_SEMANTICS_DESIGN.md`）

```rust
pub trait CrossDistance<T: Scalar, Other> {
    fn distance_to(&self, other: &Other) -> T;
}

pub trait CrossCollision<T: Scalar, Other> {
    fn collides(&self, other: &Other, tolerance: T) -> bool;
}

pub trait CrossIntersection<T: Scalar, Other> {
    fn intersection(&self, other: &Other, tolerance: T) -> IntersectionResult<T>;
}
```

## 4. 実装済みの組み合わせ

「対称」は A-B と B-A の両方を実装する組み合わせ、「片方向」は A-B のみを実装する組み合わせ。

### 4.1 2D

| 演算 | A | B | 向き |
|---|---|---|---|
| 衝突判定 | Ray2D | Circle2D | 対称 |
| 衝突判定 | Ray2D | LineSegment2D | 対称 |
| 衝突判定 | LineSegment2D | Arc2D | 対称 |
| 衝突判定 | Arc2D | Circle2D | 対称 |
| 衝突判定 | Triangle2D | Circle2D | 対称 |
| 衝突判定 | Triangle2D | LineSegment2D | 対称 |
| 衝突判定 | Ellipse2D | Circle2D | 対称 |
| 衝突判定 | EllipseArc2D | Circle2D | 対称 |
| 衝突判定 | InfiniteLine2D | Circle2D | 対称 |
| 衝突判定 | InfiniteLine2D | LineSegment2D | 対称 |
| 衝突判定 | InfiniteLine2D | Ray2D | 対称 |
| 交差 | Circle2D | Ellipse2D | 対称 |
| 交差 | Triangle2D | LineSegment2D | 対称 |
| 交差 | Ray2D | Circle2D | 対称 |
| 交差 | Ray2D | LineSegment2D | 対称 |
| 交差 | InfiniteLine2D | Circle2D | 対称 |
| 交差 | InfiniteLine2D | LineSegment2D | 対称 |
| 交差 | InfiniteLine2D | Ray2D | 対称 |

### 4.2 3D

| 演算 | A | B | 向き |
|---|---|---|---|
| 衝突判定 | InfiniteLine3D | LineSegment3D | 対称 |
| 衝突判定 | InfiniteLine3D | Ray3D | 対称 |
| 距離 | InfiniteLine3D | InfiniteLine3D | 片方向 |
| 距離 | LineSegment3D | Aabb3D | 片方向 |
| 交差 | InfiniteLine3D | LineSegment3D | 片方向 |
| 交差 | InfiniteLine3D | Ray3D | 対称 |
| 交差 | InfiniteLine3D | InfiniteLine3D | 片方向 |
| 交差 | LineSegment3D | LineSegment3D | 片方向 |
| 交差 | Ray3D | Ray3D | 片方向 |
| 交差 | Plane3D | LineSegment3D | 片方向 |
| 交差 | Plane3D | Ray3D | 片方向 |
| 交差 | Triangle3D | LineSegment3D | 対称 |
| 交差 | Triangle3D | Ray3D | 対称 |
| 交差 | ConicalSolid3D | Point3D | 片方向 |
| 交差 | ConicalSurface3D | Point3D | 片方向 |
| 交差 | ConicalSurface3D | InfiniteLine3D | 片方向 |
| 交差 | ConicalSurface3D | Ray3D | 対称 |
| 交差 | ConicalSurface3D | LineSegment3D | 片方向 |
| 交差 | SphericalSolid3D | Point3D | 片方向 |
| 交差 | SphericalSurface3D | Ray3D | 対称 |
| 交差 | LineSegment3D | SphericalSurface3D | 片方向 |

## 5. 未対応の組み合わせ

v1.0 の網羅表に登録されていたが、結果が誤った簡易実装だったため削除した組み合わせ（実装済みの表へ移したものは除く）。系統ごとに正しい実装を追加し、実装済みの表へ移す（2D → 3D の線形・円 → 2 次曲面 → NURBS の順）。

| 系統 | 演算 | A | B | 削除した実装の問題 |
|---|---|---|---|---|
| 3D 2 次曲面 | 交差 | CylindricalSurface3D | CylindricalSurface3D | 交線ではなく、2 つの中心の中点を返す |
| 3D 2 次曲面 | 交差 | ConicalSolid3D | InfiniteLine3D | 直線の基準点が立体の内部にあるかだけを調べる |
| 3D 2 次曲面 | 交差 | ConicalSolid3D | Ray3D | 半直線の始点が立体の内部にあるかだけを調べる |
| 3D 2 次曲面 | 交差 | ConicalSolid3D | LineSegment3D | 線分の始点が立体の内部にあるかだけを調べる |
| 3D 2 次曲面 | 交差 | ConicalSolid3D | Plane3D | 断面ではなく底面の中心を返す |
| 3D 2 次曲面 | 交差 | SphericalSolid3D | InfiniteLine3D | 立体の内部を通る区間ではなく、中心に最も近い点を 1 つ返す |
| 3D 2 次曲面 | 交差 | SphericalSolid3D | Ray3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | Circle3D | 曲線上の等間隔のサンプル点の距離の最小値で近似する |
| NURBS | 距離 | NurbsCurve3D | CylindricalSolid3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | EllipsoidalSolid3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | InfiniteLine3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | LineSegment3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | Plane3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | Ray3D | 同上 |
| NURBS | 距離 | NurbsCurve3D | SphericalSolid3D | 同上 |

NURBS 曲線と点の距離（v1.0 の `NurbsCurve3D`-`Point3D`）も同じ近似だったため削除した。点との距離は網羅表の範囲外のため、NURBS 曲線の trait定義の点との距離として扱う。

## 6. 確認

- `model/geo_algorithms/tests/cross_operation_matrix.rs`
  - 実装済みの組み合わせの trait の実装をコンパイル時に確かめる（対称な組み合わせは両方の向き）
  - 実装済みの組み合わせの代表的な配置で、接触・交点の有無と交点の数を確かめる
- 実装済みの表へ組み合わせを移すときは、本仕様書の表とテストの両方を更新する

## 7. 改訂の履歴

### v2.2（#855）

- 2D の楕円・楕円弧と円の衝突判定、円と楕円の交差を実装し、実装済みの表へ移した（2D の未対応はなくなった）
- 楕円のパラメータについての式は、`x = tan(θ/2)` で 4 次多項式にし、`analysis::polynomial_real_roots_in_interval` で解く（`geo_commons` の楕円のパラメータの計算）
- 同じ計算で、`EllipseArc2D::distance_to_point` が範囲内の距離の極小点を見落としていた誤りを直した

### v2.1（#855）

- 2D の円弧と円・三角形と円の衝突判定、三角形と線分の交差・衝突判定を実装し、実装済みの表へ移した

### v2.0（#855）

- 文字列の関数名で照合していた網羅表のテスト（`coverage_matrix_engine.rs`）を、trait の実装のコンパイル時の確認に置き換えた
- 形状の組み合わせごとの公開関数（`geo_algorithms::collision` / `distance` / `intersection`）を、`geo_algorithms::operations` の trait の実装に置き換えた。網羅表にない関数・引数の順序を入れ替えただけの関数・結果が誤った簡易実装は削除した
- 点との距離の組み合わせを網羅表から外した
- #343 の判断を次のとおり改める
  - 形状をまたぐ演算の trait の一次参照先は `geo_contracts::geometry::operations` ではなく `geo_algorithms::operations` とする（`geo_primitives` と `geo_nurbs` の形状の組み合わせを orphan rule の範囲で実装するため）
  - 収束の失敗を明示する結果型（`DistanceConvergenceError`）は、使う実装がなくなったため削除した。収束演算を実装する段階で改めて設計する
  - 代表の向きを実装し逆の向きはそれを呼ぶ規則、段階的に導入する方針は維持する

### v1.0（#364）

- 形状の組み合わせ・演算・必須・対称・交点の数の種別を持つ網羅表の論理スキーマと、未登録・非対称・重複の検出の規則を定めた
