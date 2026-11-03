# Transform 使用例

Transform システムの代表的な利用例をまとめます。

## 基本変換

```rust
use analysis::Angle;
use analysis::linalg::vector::Vector3;
use geo_primitives::{AnalysisTransform3D, TriangleMesh3D};

let mesh = TriangleMesh3D::new(vertices, indices)?;

let translation = Vector3::new(1.0, 2.0, 3.0);
let translated = mesh.translate_analysis(&translation)?;

let axis = Vector3::new(0.0, 0.0, 1.0);
let angle = Angle::from_degrees(90.0);
let rotated = mesh.rotate_analysis(&mesh, &axis, angle)?;

let scaled = mesh.uniform_scale_analysis(&mesh, 2.0)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 複合変換

```rust
let result = mesh.apply_composite_transform(
    Some(&Vector3::new(1.0, 0.0, 0.0)),
    Some((&mesh, &Vector3::z_axis(), angle)),
    Some((2.0, 2.0, 2.0)),
)?;

let result_uniform = mesh.apply_composite_transform_uniform(
    Some(&translation),
    Some((&mesh, &axis, angle)),
    Some(2.0),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Matrix 直接操作

```rust
use analysis::Angle;
use analysis::linalg::matrix::Matrix4x4;
use analysis::linalg::vector::Vector3;

let custom_matrix = Matrix4x4::identity()
    * Matrix4x4::translation_3d(&Vector3::new(1.0, 2.0, 3.0))
    * Matrix4x4::rotation_axis(&Vector3::z_axis(), angle.to_radians())
    * Matrix4x4::scale_3d(&Vector3::new(2.0, 2.0, 2.0));

let transformed = mesh.transform_point_matrix(&custom_matrix);
```
