# VoxelOctree 使用例

VoxelOctree の実運用向けサンプルは、このページに集約します。

## 線分経路による除去（カプセル）

```rust,ignore
use geo_algorithms::octree::voxel::VoxelOctree;
use geo_algorithms::LineSegment3D;
use geo_core::{Aabb3D, Point3D};

let work_bounds = Aabb3D::new(
    Point3D::new(0.0, 0.0, 0.0),
    Point3D::new(100.0, 100.0, 100.0),
);
let mut voxel_tree = VoxelOctree::new(work_bounds, 6);

let segment = LineSegment3D::new(
    Point3D::new(0.0, 0.0, 0.0),
    Point3D::new(50.0, 50.0, 50.0),
)
.unwrap();

voxel_tree.remove_material_capsule(&segment, 5.0);
```

## 円弧経路による除去（線分近似）

```rust,ignore
use geo_algorithms::octree::voxel::VoxelOctree;
use geo_algorithms::{Angle, Arc3D};
use geo_core::{Aabb3D, Point3D};

let work_bounds = Aabb3D::new(
    Point3D::new(0.0, 0.0, 0.0),
    Point3D::new(100.0, 100.0, 100.0),
);
let mut voxel_tree = VoxelOctree::new(work_bounds, 6);

let arc = Arc3D::xy_arc(
    Point3D::new(50.0, 50.0, 0.0),
    20.0,
    Angle::from_degrees(0.0),
    Angle::from_degrees(90.0),
)
.unwrap();

voxel_tree.remove_material_arc_polyline(&arc, 5.0, 16);
```
