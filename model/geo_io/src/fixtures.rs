//! geo_io 向けサンプルデータ生成。

use geo_primitives::{Point3D, TriangleMesh3D};
use std::path::Path;

use crate::stl;

/// デバッグ用のサンプル STL ファイルを生成する。
pub fn create_sample_stl_mesh(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let vertices = vec![
        Point3D::new(-0.5, -0.5, -0.5),
        Point3D::new(0.5, -0.5, -0.5),
        Point3D::new(0.5, 0.5, -0.5),
        Point3D::new(-0.5, 0.5, -0.5),
        Point3D::new(-0.5, -0.5, 0.5),
        Point3D::new(0.5, -0.5, 0.5),
        Point3D::new(0.5, 0.5, 0.5),
        Point3D::new(-0.5, 0.5, 0.5),
    ];

    let indices = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
    ];

    let mesh = TriangleMesh3D::new(vertices, indices)?;
    stl::save_stl(&mesh, path)?;

    tracing::info!("サンプルSTLファイル作成（立方体）: {:?}", path);
    Ok(())
}
