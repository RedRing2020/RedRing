# Cross Operation Coverage Matrix Spec (v2.5)

対象Issue: #364（v1.0）、#855（v2.0）
関連: #343, #350, #351, #366

## 1. 目的

- 2D/3D の形状の組み合わせごとの演算（衝突判定・交差・距離）の網羅表を、実装と一致した状態で保つ。
- 実装済みの組み合わせは trait の実装としてコンパイル時に確かめ、結果が誤った簡易実装を網羅表の実装済みに含めない。
- 網羅表から外した組み合わせは、外した理由と設計の検討を記録し、呼び出し元が必要になったときの実装の起点にする。

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
| 距離 | NurbsCurve3D | Circle3D | 対称 |
| 距離 | NurbsCurve3D | CylindricalSolid3D | 対称 |
| 距離 | NurbsCurve3D | EllipsoidalSolid3D | 対称 |
| 距離 | NurbsCurve3D | InfiniteLine3D | 対称 |
| 距離 | NurbsCurve3D | LineSegment3D | 対称 |
| 距離 | NurbsCurve3D | Plane3D | 対称 |
| 距離 | NurbsCurve3D | Ray3D | 対称 |
| 距離 | NurbsCurve3D | SphericalSolid3D | 対称 |
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
| 交差 | ConicalSolid3D | InfiniteLine3D | 片方向 |
| 交差 | ConicalSolid3D | Ray3D | 片方向 |
| 交差 | ConicalSolid3D | LineSegment3D | 片方向 |
| 交差 | ConicalSurface3D | Point3D | 片方向 |
| 交差 | ConicalSurface3D | InfiniteLine3D | 片方向 |
| 交差 | ConicalSurface3D | Ray3D | 対称 |
| 交差 | ConicalSurface3D | LineSegment3D | 片方向 |
| 交差 | SphericalSolid3D | Point3D | 片方向 |
| 交差 | SphericalSolid3D | InfiniteLine3D | 片方向 |
| 交差 | SphericalSolid3D | Ray3D | 片方向 |
| 交差 | SphericalSurface3D | Ray3D | 対称 |
| 交差 | LineSegment3D | SphericalSurface3D | 片方向 |

## 5. 網羅表から外した組み合わせ

v1.0 の網羅表に登録されていたが、結果が誤った簡易実装を削除したうえで、正しい結果を今の交差の結果の種類（`IntersectionGeometry`）で表せないため網羅表から外した組み合わせ。どちらもテスト以外の呼び出し元はない。ブーリアン演算・断面の表示など、呼び出し元が必要になったときに、次の検討を出発点として結果の種類と実装を設計する（#861）。

### 5.1 `CylindricalSurface3D`–`CylindricalSurface3D` の交差

| 配置 | 交わり | 表し方の案 |
|---|---|---|
| 軸が平行 | 直線 0〜2 本（軸の間隔が半径の和・差に等しいときは接する 1 本）、軸と半径が等しければ一致 | 直線の複数を表す結果の種類 |
| 半径が等しく、軸が交わる | 楕円 2 つ | 楕円の複数を表す結果の種類 |
| 一般の位置 | 4 次の空間曲線（1〜2 本のつながった曲線） | 一方の円柱の角度 θ で表すと、もう一方の円柱の式が軸方向の位置の 2 次式になり、z±(θ) と閉じた式で書ける（正確な曲線の型）。または、許容誤差以内の NURBS 曲線で近似する |

一般の位置の面と面の交線は、CAD カーネルの面と面の交差（SSI）の一部として、他の面の組み合わせと合わせて設計する。一部の配置だけを実装し、表せない配置で「交わらない」と返すことはしない。

### 5.2 `ConicalSolid3D`–`Plane3D` の交差

断面は、円錐曲線（円・楕円・放物線・双曲線）の弧と、平面が底面を切る場合はその弦で囲まれた平面上の領域になる。

- 平面上の領域を表す結果の種類（平面と境界の閉路）が必要。`geo_topology` には面（Face）・閉路（Loop）の型がまだない
- 境界の放物線・双曲線の弧は、有理の 2 次の NURBS 曲線で正確に表せる。楕円弧は `EllipseArc3D` があるが、`geo_topology` の複合曲線の区間（線分・円弧・NURBS 曲線）には含まれない

## 6. 確認

- `model/geo_algorithms/tests/cross_operation_matrix.rs`
  - 実装済みの組み合わせの trait の実装をコンパイル時に確かめる（対称な組み合わせは両方の向き）
  - 実装済みの組み合わせの代表的な配置で、接触・交点の有無と交点の数を確かめる
- 実装済みの表へ組み合わせを移すときは、本仕様書の表とテストの両方を更新する

## 7. 改訂の履歴

### v2.5（#861）

- 円柱面同士、円錐の立体と平面を網羅表から外し、結果の種類と実装の検討を 5 章に記録した（未対応の組み合わせはなくなった）
- 作る演算がなくなった `IntersectionGeometry` の `InfiniteLine` と `CompositeCurve` を削除し、`geo_algorithms` の `geo_topology` への依存を外した

### v2.4（#855）

- NURBS 曲線と 8 つの形状の距離を実装し、実装済みの表へ移した。NURBS 曲線の点との距離は、形状の trait定義（`NurbsCurve3DDistance::distance_to_point`）として追加した
- 方法は `geo_commons::nurbs_curve_minimum_distance` の分枝限定法。曲線を Bézier の区間に分解し、制御点を囲む球で距離の下界を求めて、下界が最も小さい区間を分割する。返す距離は曲線上の点での距離で、真の最小値以上、真の最小値 + 既定の距離トレランス以下
- 制約: 曲線の長い範囲で距離がほぼ一定の場合（平面に平行な曲線、同心の円等）は、区間を許容誤差の大きさまで分割するため遅く、分割の回数の上限で探索を終えることがある（その場合は精度を保証しない）。勾配を使う下界で改善する（#863）

### v2.3（#855）

- 3D の球・円錐の立体と直線・半直線（円錐は線分も）の交差を実装し、実装済みの表へ移した。凸な立体の内部を通る区間を線分として返し、外側を許容誤差以内の距離で通る場合は接する点を返す（円錐は、直線に沿った立体までの距離の最小値を `analysis::minimize_unimodal` で求める）
- 円柱面同士、円錐の立体と平面は、交線・断面を表す結果の種類の設計が必要なため、未対応に残した（#861）

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
