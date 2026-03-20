# Core/Extension 使用例

Core/Extension Foundation パターンで利用する代表的なコード例をまとめます。

## Core のみ使用

```rust
use geo_primitives::{Circle2D, Point2D};

let center = Point2D::new(0.0, 0.0);
let radius = 1.0;
let circle = Circle2D::new(center, radius)?;
let area = circle.area();
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Extension を含む使用

```rust
use geo_primitives::{Circle2D, Point2D};

let circle = Circle2D::unit_circle();
let point = Point2D::new(0.5, 0.5);
let contains = circle.contains_point(&point);
println!("contains: {}", contains);
```

## Analysis Transform（幾何変換拡張）

```rust
use analysis::Angle;
use analysis::linalg::vector::Vector3;
use geo_primitives::AnalysisTransform3D;

let translation = Vector3::new(1.0, 2.0, 3.0);
let translated = mesh.translate_analysis(&translation)?;

let axis = Vector3::new(0.0, 0.0, 1.0);
let angle = Angle::from_degrees(90.0);
let rotated = mesh.rotate_analysis(&mesh, &axis, angle)?;

let result = mesh.apply_composite_transform(
    Some(&translation),
    Some((&mesh, &axis, angle)),
    Some((2.0, 2.0, 2.0)),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Collision & Intersection（衝突判定・交差判定）

```rust
use geo_foundation::{BasicCollision, LineSegmentCollision, PointDistance};
use geo_primitives::{LineSegment3D, Point3D, Triangle3D};

let point = Point3D::new(1.0, 2.0, 3.0);
let distance = triangle.distance_to(&point);

let segment = LineSegment3D::new(start, end)?;
let intersects = triangle.intersects(&segment);

let min = (0.0, 0.0, 0.0);
let max = (10.0, 10.0, 10.0);
let distance_aabb = segment.distance_to_aabb(min, max);
println!("d={}, hit={}, aabb_d={}", distance, intersects, distance_aabb);
# Ok::<(), Box<dyn std::error::Error>>(())
```
