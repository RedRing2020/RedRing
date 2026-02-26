# NURBS 使用例

NURBS 曲線・曲面の代表的な作成と操作例をまとめます。

## 2D NURBS 曲線の作成

```rust
use geo_nurbs::{NurbsCurve2D, Point2D};

let control_points = vec![
    Point2D::new(0.0, 0.0),
    Point2D::new(1.0, 1.0),
    Point2D::new(2.0, 0.0),
];

let curve = NurbsCurve2D::new(
    control_points,
    Some(vec![1.0, 1.0, 1.0]),
    vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    2,
)?;

let point = curve.evaluate_at(0.5);
let derivative = curve.derivative_at(0.5);
let length = curve.approximate_length(100);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 3D NURBS サーフェスの作成

```rust
use geo_nurbs::{NurbsSurface3D, Point3D};

let control_grid = vec![
    vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
    vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0)],
];

let surface = NurbsSurface3D::new(
    control_grid,
    None,
    vec![0.0, 0.0, 1.0, 1.0],
    vec![0.0, 0.0, 1.0, 1.0],
    1,
    1,
)?;

let point = surface.evaluate_at(0.5, 0.5);
let normal = surface.normal_at(0.5, 0.5);
let area = surface.approximate_area(50, 50);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## NURBS 変換操作

```rust
use geo_nurbs::transform::{CurveSplitting, DegreeElevation, KnotInsertion};

let (new_points, new_weights, new_knots) =
    KnotInsertion::insert_knot_2d(&control_points, &weights, &knots, degree, 0.5)?;

let (left_curve, right_curve) =
    CurveSplitting::split_curve_2d(&control_points, &weights, &knots, degree, 0.5)?;

let (elev_points, elev_weights, elev_knots, new_degree) =
    DegreeElevation::elevate_degree_2d(&control_points, &weights, &knots, degree)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
