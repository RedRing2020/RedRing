# geo_contracts trait Structure Minimization Design

## 目的

Issue #535 では、`geo_contracts` の trait構造を次の最小構造へ再編する。

- 形状定義コンストラクタ trait
- 形状参照 trait
- それ以外の capability trait

本書は、その前提として現行 trait を棚卸しし、一次分類と曖昧境界の個別判定を明文化する。

補足:

- 本書は shape の意味論そのものの正本ではない
- 個別 shape の意味と API 意図は `GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` を正本とする
- 本書はその意味論を前提に、trait の責務分担と配置境界を定義する

## `#558` の前提: shape 意味論に基づく capability 付与条件

`#558` では、trait の置き場所を決める前に、「どの shape にどの capability を付与してよいか」を shape 意味論に基づいて固定する。

`GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` で整理した分類を、本書では次の capability 境界前提として採用する。

### 1. endpoint capability は境界付き曲線 shape に限定する

対象:

- `LineSegment`
- `Arc`
- `EllipseArc`

ここでいう endpoint capability は、少なくとも次を指す。

- `start` / `end`
- `start_point` / `end_point`
- 端点に意味を持つ `midpoint` のような区間由来語彙

設計反映:

- endpoint capability は、閉曲線や面 shape へ共通 capability として持ち込まない
- `Circle` は parameter の基準位置を持てても、shape 意味論上の正本 endpoint は持たない
- `Triangle` は頂点を持つが、curve endpoint capability の対象ではない

### 2. parameter evaluation capability は curve family に限定する

対象:

- `LineSegment`
- `Arc`
- `EllipseArc`
- `Circle`

ここでいう parameter evaluation capability は、少なくとも次を指す。

- `point_at_parameter`
- `parameter_for_point`
- `point_at_angle`
- `point_to_uv` / `uv_to_point` のような連続 parameter 座標評価

設計反映:

- `point_at_parameter` は全 shape 共通 capability とみなさない
- `Triangle` のような面 shape に、curve parameter capability を横展開しない
- surface family を扱う場合は、curve parameter と surface parameter を同一 trait に混在させない

### 3. periodic parameter は閉曲線 shape の個別論点として扱う

対象:

- `Circle`

設計反映:

- 閉曲線の parameter capability は、境界付き曲線と同一視しない
- `Circle` の parameter は評価 capability として許容するが、`start/end` を導く根拠には使わない
- periodic curve を導入する場合、endpoint capability とは別の分岐として扱う

### 4. boundary access は curve endpoint と polygon vertex を分ける

対象:

- curve endpoint: `LineSegment`, `Arc`, `EllipseArc`
- vertex/boundary access: `Triangle`

設計反映:

- `Triangle` の `vertex_a/b/c` や edge length は、curve endpoint capability ではなく polygon / face boundary access として扱う
- 面 shape の boundary access は、curve endpoint と同じ trait 群へ混在させない

### 5. primary measure vocabulary は shape family ごとに分ける

優先語彙:

- 境界付き曲線 shape: `length`
- 閉曲線: `circumference`
- 面 shape: `area`
- 互換 API: `measure`

設計反映:

- `measure` は単一の意味語彙として新設 capability の中心に置かない
- `*Measure` を後方互換の集約 trait として残す場合でも、新設 capability の説明単位は `length` / `circumference` / `area` のような具体語彙を優先する

## `#558` の一次配置方針

上記前提を踏まえ、`#558` では少なくとも次の capability 群を区別して扱う。

### shape definition core に残すもの

- `*Constructor`
- `*Properties`
- lightweight metadata

補足:

- `Properties` に残すのは、shape の定義パラメータと、その場で読める lightweight metadata に限定する
- endpoint capability は、shape にとって正本境界参照である場合に限って `Properties` 側へ残す余地がある

### minimal extension に置くもの

- curve parameter evaluation
- surface parameter evaluation
- containment
- unary distance
- projection
- sampling
- shape 固有 derived capability

補足:

- endpoint capability を `Properties` に残さない shape では、この層に独立 capability として置く
- `Circle` の periodic evaluation は、この層の curve evaluation 分岐として扱う
- `Triangle` の vertex / edge / perimeter は、面 shape 向け boundary / derived capability としてここで扱う

### operations に置くもの

- cross-shape distance
- collision
- intersection
- relation API
- solver / approximation / strategy oriented capability

## `#558` で特に分離対象とする混在

現状棚卸しから、少なくとも次の混在を解く必要がある。

### Arc / EllipseArc

- `start_point` / `end_point`
- `point_at_parameter`
- `point_at_angle`
- `midpoint` / `mid_point`
- `contains_point`
- `distance_to_point`

方針:

- endpoint
- parameter evaluation
- angle evaluation
- containment
- unary distance

を別 capability として説明できる状態へ分解する。

### Circle

- `circumference`
- `area`
- `contains_point`
- `distance_to_point`
- `closest_point_to`
- `point_at_parameter`

方針:

- 閉曲線の評価 capability
- 閉曲線の長さ語彙
- 面積語彙
- unary relation / projection

を分離し、endpoint 系 capability は導入しない。

### Triangle

- `measure`
- `edge_ab_length` / `edge_bc_length` / `edge_ca_length`
- `perimeter`
- `contains_point`
- `distance_to_point`
- `is_clockwise` / `is_planar`

方針:

- 面 shape の primary measure
- boundary / vertex access
- derived
- containment
- unary distance

を分離し、curve parameter capability と同列には扱わない。

## `#558` の具体的な trait 再分類一覧

ここでは、現行 trait を直ちに分割実装するのではなく、各 API をどの capability 群へ属させるべきかを shape ごとに一次分類する。

凡例:

- `definition`: `Constructor` / `Properties` に残す候補
- `endpoint`: 境界付き曲線の端点参照 capability
- `evaluation`: parameter または angle による評価 capability
- `derived`: unary な派生値 capability
- `containment`: 単一点に対する包含判定
- `distance`: 単一点に対する距離 capability
- `projection`: 単一点に対する最近点・射影 capability
- `sampling`: 離散化 capability

### LineSegment の再分類

基準形として、LineSegment はすでにかなり分離済みである。

| 現行 trait / API | 再分類 |
| --- | --- |
| `LineSegment2DConstructor` / `LineSegment3DConstructor` | `definition` |
| `LineSegment2DProperties::start/end/midpoint/length` | `definition` |
| `LineSegment3DProperties::start/end/midpoint/length` | `definition` |
| `LineSegment2DProperties::dimension/is_unit_length/is_horizontal/is_vertical` | `definition` |
| `LineSegment3DProperties::dimension/is_unit_length/is_on_xy_plane/is_on_yz_plane` | `definition` |
| `LineSegment2DDerived::measure/direction_vector/as_vector` | `derived` |
| `LineSegment3DDerived::measure/direction_vector/as_vector` | `derived` |
| `LineSegment2DEvaluation::point_at_parameter` | `evaluation` |
| `LineSegment3DEvaluation::point_at_parameter` | `evaluation` |
| `LineSegment2DContainment::contains_point` | `containment` |
| `LineSegment3DContainment::contains_point` | `containment` |
| `LineSegment2DDistance::distance_to_point` | `distance` |
| `LineSegment3DDistance::distance_to_point` | `distance` |
| `LineSegment2DProjection::closest_point_to` | `projection` |
| `LineSegment3DProjection::closest_point_to` | `projection` |
| `LineSegment2DMeasure` / `LineSegment3DMeasure` | 後方互換の集約 trait |

補足:

- `LineSegment` では endpoint が shape 意味論上の正本なので、`start/end/midpoint/length` を `Properties` に残す方針を維持する
- `measure` は primary vocabulary ではなく、`derived` 側の互換 API とみなす

### Arc の再分類

Arc は `Measure` に endpoint / evaluation / containment / distance が集中しているため、最優先の分離対象である。

| 現行 trait / API | 再分類 |
| --- | --- |
| `Arc2DConstructor` / `Arc3DConstructor` | `definition` |
| `Arc2DProperties::center/radius/start_angle/end_angle/dimension/angle_span/is_full_circle/is_semicircle` | `definition` |
| `Arc3DProperties::center/radius/start_angle/end_angle/dimension/angle_span/is_full_circle/is_on_xy_plane` | `definition` |
| `Arc2DMeasure::measure` | `derived` |
| `Arc3DMeasure::measure` | `derived` |
| `Arc2DMeasure::start_point/end_point/midpoint` | `endpoint` |
| `Arc3DMeasure::start_point/end_point/midpoint` | `endpoint` |
| `Arc2DMeasure::point_at_parameter` | `evaluation` |
| `Arc3DMeasure::point_at_parameter` | `evaluation` |
| `Arc2DMeasure::point_at_angle` | `evaluation` |
| `Arc3DMeasure::point_at_angle` | `evaluation` |
| `Arc2DMeasure::contains_point` | `containment` |
| `Arc3DMeasure::contains_point` | `containment` |
| `Arc2DMeasure::distance_to_point` | `distance` |
| `Arc3DMeasure::distance_to_point` | `distance` |
| `Arc2DSampling::sample_points/sample_by_arc_length` | `sampling` |
| `Arc2DContainment::contains_point/contains_angle` | `containment` |
| `Arc2DContainment::point_at_angle` | `evaluation` |
| `Arc2DCore` / `Arc3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- `start_point/end_point/midpoint` は Arc では endpoint capability として自然に存在する
- `point_at_angle` は endpoint ではなく angle evaluation として分ける
- `Arc2DContainment` は現行の時点で `contains_*` と `point_at_angle` を混在しているため、分割候補として扱う

### EllipseArc の再分類

EllipseArc も Arc と同系統だが、`bounding_box` と tolerance 付き containment が混在している点が追加論点である。

| 現行 trait / API | 再分類 |
| --- | --- |
| `EllipseArc2DConstructor` / `EllipseArc3DConstructor` | `definition` |
| `EllipseArc2DProperties::center/semi_major_axis/semi_minor_axis/start_angle/end_angle/rotation/sweep_angle/eccentricity` | `definition` |
| `EllipseArc3DProperties::center/semi_major_axis/semi_minor_axis/start_angle/end_angle/normal/sweep_angle/eccentricity` | `definition` |
| `EllipseArc2DMeasure::measure` | `derived` |
| `EllipseArc3DMeasure::measure` | `derived` |
| `EllipseArc2DMeasure::start_point/end_point/mid_point` | `endpoint` |
| `EllipseArc3DMeasure::start_point/end_point/mid_point` | `endpoint` |
| `EllipseArc2DMeasure::point_at_parameter` | `evaluation` |
| `EllipseArc3DMeasure::point_at_parameter` | `evaluation` |
| `EllipseArc2DMeasure::point_at_angle` | `evaluation` |
| `EllipseArc3DMeasure::point_at_angle` | `evaluation` |
| `EllipseArc2DMeasure::contains_point` | `containment` |
| `EllipseArc3DMeasure::contains_point` | `containment` |
| `EllipseArc2DMeasure::bounding_box` | `derived` |
| `EllipseArc3DMeasure::bounding_box` | `derived` |
| `EllipseArc2DCore` / `EllipseArc3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- `contains_point(point, tolerance)` の tolerance 引数は unary containment capability 側の責務として扱う
- `bounding_box` は relation ではないため `operations` ではなく unary `derived` 側へ置く

### Ellipse の再分類

Ellipse は閉曲線 shape であり、Circle と同様に endpoint capability は持たない。
一方で、現行 API は `perimeter` と `measure` を併存させており、閉曲線の主語彙が曖昧である。

本整理では、Ellipse の周回長語彙は `length` へ寄せず、閉曲線 family の primary vocabulary として `circumference` を採用する。
`perimeter` は既存 API との互換語彙、`measure` は集約互換 API として後退させる。

| 現行 trait / API | 再分類 |
| --- | --- |
| `Ellipse2DConstructor` / `Ellipse3DConstructor` | `definition` |
| `Ellipse2DProperties::center/semi_major_axis/semi_minor_axis/rotation` | `definition` |
| `Ellipse3DProperties::center_3d/center_3d_tuple/normal/major_axis_direction/minor_axis_direction/semi_major_axis/semi_minor_axis` | `definition` |
| `Ellipse2DDerived::eccentricity/focal_distance/focus1/focus2/linear_eccentricity/is_circle` | `derived` |
| `Ellipse3DDerived::eccentricity/focal_distance/is_circle` | `derived` |
| `Ellipse2DDerived::circumference/perimeter/measure/area` | `derived` |
| `Ellipse3DDerived::circumference/perimeter/measure/area` | `derived` |
| `Ellipse2DEvaluation::point_at_parameter` | `evaluation` |
| `Ellipse3DEvaluation::point_at_parameter` | `evaluation` |
| `Ellipse2DContainment::contains_point/point_on_boundary` | `containment` |
| `Ellipse3DContainment::contains_point_3d` | `containment` |
| `Ellipse2DDistance::distance_to_point` | `distance` |
| `Ellipse3DDistance::distance_to_point_3d` | `distance` |
| `Ellipse2DCore` / `Ellipse3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- Ellipse の周回長は `length` ではなく `circumference` を主語彙とする
- `perimeter` は段階廃止候補だが、当面は `circumference` への互換 alias として残してよい
- `measure` は新規責務説明には使わず、後方互換の集約 API としてのみ扱う
- `focus1/focus2/focal_distance/eccentricity/linear_eccentricity/is_circle` は定義パラメータではなく unary `derived` とみなす

### NURBS Curve の再分類

NURBS curve は endpoint を shape 意味論の正本として持たず、parameter evaluation と length 語彙を中心に整理する。
既存 API は 2D/3D ともに `*Measure` へ point evaluation、接線、長さ、曲率、弧長逆算が混在しているため、少なくとも evaluation と derived を分ける必要がある。

本整理では、curve family の primary measure vocabulary として `length` を維持し、`*Measure` は後方互換の集約 trait として後退させる。

| 現行 trait / API | 再分類 |
| --- | --- |
| `NurbsCurve2DConstructor` / `NurbsCurve3DConstructor` | `definition` |
| `NurbsCurve2DProperties::degree/num_control_points/knot_vector/is_rational/dimension` | `definition` |
| `NurbsCurve3DProperties::degree/knot_vector/control_points_count/weights/is_rational/parameter_domain/coordinates` | `definition` |
| `NurbsCurve2DEvaluation::point_at/tangent_at` | `evaluation` |
| `NurbsCurve3DEvaluation::evaluate/parameter_at_length` | `evaluation` |
| `NurbsCurve2DDerived::length/curvature_at` | `derived` |
| `NurbsCurve3DDerived::arc_length/arc_length_total` | `derived` |
| `NurbsCurve2DCore` / `NurbsCurve3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- NURBS curve は `Circle` のような closed curve ではないため、主語彙は `circumference` ではなく `length` を維持する
- `parameter_at_length` は endpoint ではなく、length から parameter を求める evaluation capability として扱う
- `arc_length(u_start, u_end, tolerance)` は区間に対する unary derived quantity とみなし、後方互換の `*Measure` から分離する
- adaptive tessellation や近似戦略は operations/strategy 側の論点であり、本整理では core capability へ持ち込まない

### NURBS Surface の再分類

NURBS surface は curve family と異なり、UV parameter evaluation と surface area 語彙を中心に整理する。
既存 API は `*Measure` に point evaluation、normal/tangent evaluation、surface area が混在しているため、少なくとも evaluation と derived を分ける必要がある。

本整理では、surface family の primary measure vocabulary として `surface_area` を維持し、`*Measure` は後方互換の集約 trait として後退させる。

| 現行 trait / API | 再分類 |
| --- | --- |
| `NurbsSurface3DConstructor` | `definition` |
| `NurbsSurface3DProperties::u_degree/v_degree/u_count/v_count/u_knots/v_knots/is_rational/coordinates/weights` | `definition` |
| `NurbsSurface3DEvaluation::point_at_uv/normal_at/tangent_vectors_at` | `evaluation` |
| `NurbsSurface3DDerived::surface_area` | `derived` |
| `NurbsSurface3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- NURBS surface では `surface_area` を primary vocabulary とし、`measure` は新規責務説明の中心に置かない
- `normal_at` と `tangent_vectors_at` は unary derived quantity ではなく、UV parameter に依存する evaluation capability として扱う
- adaptive tessellation や近似戦略は operations/strategy 側の論点であり、本整理では core capability へ持ち込まない

### Circle の再分類

Circle は閉曲線であり、parameter evaluation を持っても endpoint capability は持たない。

| 現行 trait / API | 再分類 |
| --- | --- |
| `Circle2DConstructor` / `Circle3DConstructor` | `definition` |
| `Circle2DProperties::center/radius/ref_direction/diameter/dimension/is_unit_circle/is_centered_at_origin/is_degenerate` | `definition` |
| `Circle3DProperties::center/radius/axis/ref_direction/dimension/is_unit_circle/is_centered_at_origin/is_degenerate/is_on_xy_plane` | `definition` |
| `Circle2DMeasure::circumference` | `derived` |
| `Circle3DMeasure::circumference` | `derived` |
| `Circle2DMeasure::area` | `derived` |
| `Circle3DMeasure::area` | `derived` |
| `Circle2DMeasure::point_at_parameter` | `evaluation` |
| `Circle3DMeasure::point_at_parameter` | `evaluation` |
| `Circle2DMeasure::contains_point/point_on_circumference` | `containment` |
| `Circle3DMeasure::contains_point/point_on_circumference` | `containment` |
| `Circle2DMeasure::distance_to_point` | `distance` |
| `Circle3DMeasure::distance_to_point` | `distance` |
| `Circle2DMeasure::closest_point_to` | `projection` |
| `Circle3DMeasure::closest_point_to` | `projection` |
| `Circle2DCore` / `Circle3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- `area` は閉曲線そのものの評価というより、その interior を伴う派生量として扱う
- `ref_direction` は parameter 原点の便宜的基準として使えても、endpoint capability の根拠には使わない

### Triangle の再分類

Triangle は面 shape であり、curve endpoint や curve parameter capability と切り離して扱う必要がある。

| 現行 trait / API | 再分類 |
| --- | --- |
| `Triangle2DConstructor` / `Triangle3DConstructor` | `definition` |
| `Triangle2DProperties::vertex_a/vertex_b/vertex_c` | `definition` |
| `Triangle3DProperties::vertex_a/vertex_b/vertex_c` | `definition` |
| `Triangle2DProperties::centroid/circumcenter/incenter/circumradius/inradius` | `derived` |
| `Triangle3DProperties::centroid/normal/circumcenter/circumradius/inradius` | `derived` |
| `Triangle2DMeasure::measure` | `derived` |
| `Triangle3DMeasure::measure` | `derived` |
| `Triangle2DMeasure::edge_ab_length/edge_bc_length/edge_ca_length` | `derived` |
| `Triangle3DMeasure::edge_ab_length/edge_bc_length/edge_ca_length` | `derived` |
| `Triangle2DMeasure::perimeter` | `derived` |
| `Triangle3DMeasure::perimeter` | `derived` |
| `Triangle2DMeasure::contains_point` | `containment` |
| `Triangle3DMeasure::contains_point` | `containment` |
| `Triangle2DMeasure::distance_to_point` | `distance` |
| `Triangle3DMeasure::distance_to_point` | `distance` |
| `Triangle2DMeasure::is_clockwise` | `derived` |
| `Triangle3DMeasure::is_planar` | `derived` |
| `Triangle2DCore` / `Triangle3DCore` | `Constructor + Properties` へ縮退候補 |

補足:

- `vertex_a/b/c` は面 shape の boundary access であり、curve endpoint capability とは別物として扱う
- `centroid` や `normal` は lightweight metadata ではなく、実質的には unary `derived` capability とみなす

## `#558` の一次結論: `Properties` に残すものと出すもの

今回の再分類から、少なくとも次をルール化できる。

- `Properties` に残すのは、shape の定義パラメータと軽量な状態参照だけに絞る
- 境界付き曲線の `start/end/length` は、shape 意味論上の正本境界なら `Properties` に残してよい
- `centroid`、`normal`、`circumradius`、`bounding_box` のような unary 派生量は `Properties` から `derived` へ出す
- `contains_point`、`distance_to_point`、`closest_point_to`、`point_at_parameter`、`point_at_angle` は `Properties` に残さない
- `*Core` は新設 capability の説明単位ではなく、互換のための `Constructor + Properties` alias としてのみ残す

## 現行構造の棚卸し

### `geometry/core`

`geometry/core` には shape ごとの trait ファイルが並び、概ね次の組を持つ。

- `*Constructor`
- `*Properties`
- `*Measure`
- `*Core`

代表例:

- `point_traits.rs`: `Point2DConstructor` / `Point2DProperties` / `Point2DMeasure` / `Point2DCore`
- `vector_traits.rs`: `Vector2DConstructor` / `Vector2DProperties` / `Vector2DMeasure` / `Vector2DCore`
- `circle_traits.rs`: `Circle2DConstructor` / `Circle2DProperties` / `Circle2DMeasure` / `Circle2DCore`

ただし、現行 `core` には shape 定義以外も混在している。

代表例:

- `arc_traits.rs`: `Arc2DSampling`, `Arc2DContainment`
- `linesegment_traits.rs`: `LineSegment3DCollisionDetection`
- `aabb_traits.rs`: 参照系と関係判定系が単一 trait に混在

### `geometry/foundation`

`geometry/foundation` には次の 2 trait がある。

- `ExtensionFoundation<T>`
- `Bounded<T>`

`ExtensionFoundation<T>` は `primitive_kind()` と `measure()` を持つが、後者は各 shape の `*Measure` と責務が重なる。

### `geometry/operations`

`geometry/operations` には横断演算系 trait がある。

- `collision.rs`: `BasicCollision`, `AdvancedCollision`, `PointDistance`, `BBoxCollision`
- `distance.rs`: `CrossDistance`, `FallibleCrossDistance`, `DistanceConvergenceError`
- `intersection.rs`: `BasicIntersection`, `MultipleIntersection`, `SelfIntersection`
- `ellipse_calculation_traits.rs`: `EllipseCalculation`, `EllipseAdaptiveCalculation`, `EllipseAccuracyAnalysis`

## 一次分類

### A. shape definition core 残留候補

この層には、形状を定義し、形状の定義パラメータを参照する trait だけを残す。

- 全 `*Constructor`
- 全 `*Properties`
- `*Core` は残す場合でも `Constructor + Properties` の統合 alias に限定する

該当例:

- `Point2DConstructor`, `Point2DProperties`
- `Vector3DConstructor`, `Vector3DProperties`
- `Circle2DConstructor`, `Circle2DProperties`
- `Plane3DConstructor`, `Plane3DProperties`

### B. minimal extension 候補

この層には、単一 shape に対する追加 capability を置く。

- 全 `*Measure`
- `Arc2DSampling`
- `Arc2DContainment`
- `Bounded<T>`
- `primitive_kind()` のような shape metadata capability

### C. operations 残留候補

この層には、shape 間演算や計算戦略を置く。

- `BasicCollision`, `AdvancedCollision`, `PointDistance`, `BBoxCollision`
- `CrossDistance`, `FallibleCrossDistance`, `DistanceConvergenceError`
- `BasicIntersection`, `MultipleIntersection`, `SelfIntersection`
- `EllipseCalculation`, `EllipseAdaptiveCalculation`, `EllipseAccuracyAnalysis`
- `LineSegment3DCollisionDetection`

## 曖昧境界の個別判定

### 1. `*Core` は shape definition か

判定:

- Yes, ただし統合対象は `Constructor + Properties` に限定する

理由:

- `*Core` 自体は責務ではなく公開上の統合 alias だから
- 現行の `Constructor + Properties + Measure` は最小構造の境界を曖昧にするから

設計反映:

- `*Core` を残すなら `shape definition core alias` としてのみ扱う
- `*Measure` は `*Core` から外す

### 2. `*Properties` の中の派生 getter は参照か

判定:

- Yes

対象例:

- `dimension()`
- `diameter()`
- `angle_span()`
- `is_unit_circle()`
- `is_on_xy_plane()`
- `is_zero()`

理由:

- 単一 shape の保持パラメータから直接導ける lightweight metadata であり、別 capability 層へ出すと分解過剰になるから
- 他 shape や外部入力を取らず、探索・射影・サンプリング・solver を伴わないから

設計反映:

- `Properties` には lightweight metadata を残してよい
- ただし point evaluation, projection, containment, distance は残さない

### 3. `*Measure` は shape definition か

判定:

- No

理由:

- 長さ、面積、体積、距離、補間、射影、法線評価は shape の定義そのものではなく capability だから
- 現行ソースにも `TODO(#318): Measure contracts are temporarily colocated and will be split by responsibility.` が残っているから

設計反映:

- 全 `*Measure` を shape definition core から分離する

### 4. `contains_point` は `Properties` 側か `Measure` 側か

判定:

- shape definition には置かない
- 単一 shape capability として `Measure` 側または後継 capability trait 側へ置く

理由:

- 点 inclusion 判定は外部入力を取る関係判定であり、単なる参照ではないから
- `PointDistance` や collision 系との責務境界も明示しやすくなるから

設計反映:

- `contains_point` は definition 層から除外する

### 5. `point_at_parameter` / `sample_points` はどこに属するか

判定:

- shape definition には置かない
- 単一 shape の evaluation / sampling capability として extension 側へ置く

理由:

- これらは shape の参照ではなく、shape を使った計算・離散化だから

設計反映:

- `Arc2DSampling` は extension 側へ移す
- `point_at_parameter` を含む API 群も definition 層から除外する

### 6. `distance_to_point` / `distance_to_circle` / `distance_to_segment` はどこに属するか

判定:

- shape definition には置かない
- cross-shape を含むものは `operations`
- 単一点を相手にするものも definition 層には置かない

理由:

- 距離計算は shape 参照ではなく演算だから
- 現行でも同種 API は `geometry::operations::CrossDistance` へ移行中だから

設計反映:

- deprecated な same-shape distance API は `operations` へ寄せるか削除する
- `distance_to_point` も definition から外す

### 7. `ExtensionFoundation<T>` は現形のまま残すべきか

判定:

- No

理由:

- `primitive_kind()` は metadata capability だが、`measure()` は `*Measure` と責務重複するから
- 1 trait に metadata と generic measure summary を束ねると境界が再び曖昧になるから

設計反映:

- `ExtensionFoundation<T>` はそのまま残さない
- 必要なら `primitive_kind()` 専用の lightweight metadata trait に分離する
- `measure()` は dedicated capability 側へ寄せる

### 8. `Bounded<T>` は shape definition か

判定:

- No

理由:

- AABB は shape の定義パラメータではなく、shape から導出される spatial envelope だから
- shape definition を読むだけなら `aabb()` を必須にしなくてよいから

設計反映:

- `Bounded<T>` は extension 側の capability として扱う

### 9. `Aabb2DTrait` / `Aabb3DTrait` は 1 trait のままでよいか

判定:

- No

理由:

- `min()` / `max()` は shape 参照だが、`contains_point()` / `contains_bbox()` / `intersects()` は関係演算だから
- `width()` / `height()` / `depth()` / `center()` / `area()` / `volume()` / `is_valid()` は unary derived capability であり、definition と operations の中間にあるから

設計反映:

- AABB は少なくとも次の 2 層に分ける
- `Aabb*Properties`: `min`, `max`
- `Aabb*Derived`: `width`, `height`, `depth`, `center`, `area`, `volume`, `is_valid`
- `contains_point`, `contains_bbox`, `intersects` は extension または operations へ移す
- 既存 `Aabb2DTrait` / `Aabb3DTrait` の延命は前提にせず、新規 capability 群として再定義してよい
- AABB 単体で参照系 / derived / relation の意味を再設計し、旧 trait 命名との互換維持より責務境界の明確化を優先する

### 10. `EllipseCalculation*` は extension か operations か

判定:

- operations

理由:

### 14. Rect と linear shape への適用方針

判定:

- #540 の AABB で使った `properties / derived / relation` の説明軸を、そのまま Rect と linear shape 群へ拡張する
- ただし linear shape 群では `evaluation / projection / transform` も明示的に分離する

対象:

- `Rect2D/3D`
- `InfiniteLine2D/3D`
- `Ray2D/3D`
- `LineSegment2D/3D`
- `Plane3D`

設計反映:

- `*Core` は `Constructor + Properties` の統合 alias に縮小する
- 旧 `*Measure` は後方互換のために集約 trait として残してよいが、新規実装の責務配置はそこで説明しない
- `contains_point` は `Containment`
- `point_at_parameter` / `parameter_for_point` / `point_to_uv` / `uv_to_point` は `Evaluation`
- `closest_point` / `project_point` は `Projection`
- `mirror_point` / `rotate_*` / `translate` / `reverse` / `offset` は `Transform`
- `area` / `perimeter` / `corners` / `direction_vector` / `as_vector` / `equation_coefficients` は `Derived`
- `distance_to_point` のような単一点相手の距離は unary capability として `Distance` に分ける

理由:

- `Measure` という名前のまま unary measure, relation, projection, transform を混在させると、AABB 後に採用した taxonomy と説明軸が揃わないから
- 先行対象で capability taxonomy を分けておくと、後続の `Circle` / `Triangle` / solid / surface にも同じ軸で横展開できるから

- これは shape の定義でも一般的な単一 shape metadata でもなく、近似式・数値計算戦略の選択そのものだから
- capability としても algorithm 寄りであり、他の operations 群と同じ層に置く方が自然だから

設計反映:

- `EllipseCalculation`, `EllipseAdaptiveCalculation`, `EllipseAccuracyAnalysis` は operations 側に残す
- ただし `geo_contracts` に置くのは trait定義の正本までとし、adaptive 選択や比較分析の default 実装は持ち込まない
- strategy の実装本体は `geo_primitives` の impl entry point または将来的な `geo_algorithms` 側 helper へ置く
- 近似式・距離計算の数値カーネル自体は `geo_commons` に維持し、`geo_contracts` は数値閾値や比較ロジックを保持しない

### 11. `LineSegment3DCollisionDetection` は extension か operations か

判定:

- operations

理由:

- AABB との cross-shape 演算であり、単一 shape の自己記述ではないから

設計反映:

- `geometry/core` から外し、operations 側へ寄せる

### 12. #541 で優先して移す対象

Issue #541 では、まず既存の `geometry::operations` trait に自然に載るものから移す。

- deprecated な same-shape distance API
- `LineSegment3D` と AABB の距離
- `InfiniteLine3D` / `Plane3D` / `Ellipse3D` が持つ明示的な交点計算

設計反映:

- 距離は `CrossDistance`
- 単一点交差は `BasicIntersection`
- 複数交点は `MultipleIntersection`
- concrete 型の convenience method は必要最小限だけ残し、trait定義の正本は `operations` 側に寄せる

### 13. relation 系メソッドは独立に厳格判定する

判定:

- optional ではなく、責務分離の中核ルールとして扱う
- 相手 shape の存在を前提にし、幾何学的な関係を返す API は relation 系として個別判定する

relation 系の代表例:

- `is_parallel_to`
- `is_perpendicular_to`
- `is_same_line`
- `intersects`
- `is_skew_to`
- `is_same_direction`
- `is_opposite_direction`
- `angle_to`
- `angle_between`
- `closest_points`

理由:

- unary な measure / evaluation / metadata と、binary な relation が混在すると `core` と `minimal extension` の責務が再び曖昧になるから
- 相手 shape を取る relation API を曖昧に残すと、same-shape distance や intersection と同様に `operations` へ寄せるべき責務が再流入するから
- #541 の完了条件は、cross-shape の計算だけでなく relation API の所属も一貫して説明できる状態にすることだから

設計反映:

- relation 系メソッドは `measure` の残余として扱わない
- `other: &Self` や他 shape 引数を取り、関係判定・角度・最近点対を返す API は relation 系候補として必ず棚卸しする
- relation 系を `minimal extension` に残す場合でも、`operations` に寄せない理由を Issue / PR で明示する

## 採用する境界線

最終的な境界線は次のとおりとする。

### shape definition core

- `*Constructor`
- `*Properties`
- 必要なら `*Core = Constructor + Properties`

### minimal extension

- `*Measure`
- `Bounded`
- sampling / evaluation / containment のような単一 shape capability
- `primitive_kind()` のような lightweight metadata capability

補足:

- relation 系メソッドはこの層へ無批判に残さない
- 相手 shape を取る API は、まず `operations` 候補として判定する

### operations

- collision / distance / intersection
- strategy / approximation / solver oriented capability
- cross-shape specialized contracts
- relation API

## 次段で必要な作業

- 各 `*_traits.rs` を上記 3 層へ再配置した場合の配置案を作る
- `ExtensionFoundation` の後継を `primitive_kind` 専用に残すか廃止するか決める
- `Aabb*Trait` の分割粒度を確定する
- `*Core` alias を公開 API として残すか決める

## 実装タスク管理

段階的移行タスクと、各実装 Issue 本文に記載すべきチェック項目は、#535 の成果物として本設計書で管理する。

- 各形状または各責務分離タスクの実装 Issue は、本設計書のチェック項目を適用して起票する
- チェック項目は独立のテンプレート Issue ではなく、#535 の設計成果物として扱う

実装 Issue は shape 単位ではなく、責務分離単位で分ける。

- `definition core 縮退`
- `measure 系 extension 分離`
- `metadata capability 分離`
- `operations への移送`
- `AABB 特例整理`

## 実装 Issue 用チェック項目

各実装 Issue では、少なくとも以下を本文に含める。

### 1. 対象確定

- [ ] 対象 trait 名と対象ファイルが列挙されている
- [ ] 非対象 trait が明記されている
- [ ] 変更先の層が `shape definition core` / `minimal extension` / `operations` のどれか明記されている

### 2. 責務判定

- [ ] `Constructor` は definition core に残すか確認した
- [ ] `Properties` の各メソッドが参照系に残せるか確認した
- [ ] `Measure` 系 API を definition core に残していない
- [ ] `contains_point` / `distance_to_point` / `point_at_parameter` / sampling 系を個別判定した
- [ ] relation 系メソッド（平行・垂直・交差・同方向・角度・最近点対）を個別判定した

### 3. 配置変更

- [ ] 移動先モジュールが決まっている
- [ ] `mod.rs` の公開が更新されている
- [ ] `geo_contracts::lib.rs` の公開が必要なら更新されている
- [ ] `*Core` alias を残す場合は `Constructor + Properties` のみを束ねている

### 4. 互換性

- [ ] 旧経路は duplicate trait ではなく re-export または alias で維持している
- [ ] 利用側の import 経路が壊れていない
- [ ] 一時互換を残す場合、削除予定が Issue または文書に書かれている

### 5. 境界の明確化

- [ ] extension と operations の境界が今回の変更で曖昧になっていない
- [ ] metadata capability と measure capability を混在させていない
- [ ] cross-shape 演算を definition core に持ち込んでいない
- [ ] relation 系メソッドを "measure のついで" で残していない
- [ ] shape 参照と capability の責務を文章で説明できる

### 6. 検証

- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo fmt`
- [ ] `cargo test --workspace`
- [ ] 必要なら `./scripts/check_architecture_dependencies.ps1 -ExitOnError`

### 7. 完了判定

- [ ] 正本モジュールが 1 つに定まっている
- [ ] 旧構造へ逆戻りする duplicate 定義が入っていない
- [ ] 次段階で扱う未解決事項が列挙されている
- [ ] PR 本文で「残したもの」と「次 Issue へ送るもの」を分離して説明できる
