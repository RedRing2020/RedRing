//! Octree向けサンプルデータ生成。

use geo_core::{Aabb3D, Point3D};
use geo_primitives::LineSegment3D;

use super::VoxelOctree;

/// ボックス除去ベースのサンプルVoxelOctreeを生成する。
pub fn create_sample_voxel_octree(max_depth: usize, query_expand: f64) -> VoxelOctree<f64> {
    let work_bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 50.0),
    );
    let mut voxel_tree = VoxelOctree::new(work_bounds, max_depth);

    let center_pocket = Aabb3D::new(
        Point3D::new(30.0 - query_expand, 30.0 - query_expand, 0.0),
        Point3D::new(70.0 + query_expand, 70.0 + query_expand, 50.0),
    );
    voxel_tree.remove_material_box(&center_pocket);

    let horizontal_slot = Aabb3D::new(
        Point3D::new(0.0, 45.0 - query_expand, 0.0),
        Point3D::new(100.0, 55.0 + query_expand, 50.0),
    );
    voxel_tree.remove_material_box(&horizontal_slot);

    voxel_tree
}

/// 掃引円柱除去ベースのサンプルVoxelOctreeを生成する。
pub fn create_sample_swept_cylinder_voxel_octree(max_depth: usize) -> VoxelOctree<f64> {
    let work_bounds = Aabb3D::new(
        Point3D::new(-60.0, -60.0, -20.0),
        Point3D::new(60.0, 60.0, 30.0),
    );
    let mut voxel_tree = VoxelOctree::new(work_bounds, max_depth);

    let sample_segments = [
        (
            Point3D::new(-45.0, -25.0, 5.0),
            Point3D::new(45.0, -25.0, 5.0),
        ),
        (Point3D::new(-45.0, 0.0, 4.0), Point3D::new(45.0, 0.0, 4.0)),
        (
            Point3D::new(-45.0, 25.0, 3.0),
            Point3D::new(45.0, 25.0, 3.0),
        ),
    ];

    for (start, end) in sample_segments {
        if let Some(line) = LineSegment3D::new(start, end) {
            voxel_tree.remove_material_swept_cylinder(&line, 5.0);
        }
    }

    voxel_tree
}
