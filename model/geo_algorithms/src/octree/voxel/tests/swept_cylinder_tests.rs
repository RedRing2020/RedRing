use super::super::*;
use crate::LineSegment3D;
use geo_core::Point3D;

#[test]
fn test_swept_cylinder_removes_less_than_capsule_near_segment_ends() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let mut tree_capsule = VoxelOctree::new(bounds, 6);
    let mut tree_swept = VoxelOctree::new(bounds, 6);

    let initial_volume = tree_capsule.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(30.0, 50.0, 50.0),
        Point3D::new(70.0, 50.0, 50.0),
    )
    .unwrap();

    tree_capsule.remove_material_capsule(&segment, 10.0);
    tree_swept.remove_material_swept_cylinder(&segment, 10.0);

    let removed_capsule = initial_volume - tree_capsule.remaining_volume();
    let removed_swept = initial_volume - tree_swept.remaining_volume();

    assert!(removed_swept > 0.0);
    assert!(removed_capsule > removed_swept);
}

#[test]
fn test_swept_cylinder_no_intersection_keeps_volume() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 5);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(200.0, 200.0, 200.0),
        Point3D::new(300.0, 300.0, 300.0),
    )
    .unwrap();

    voxel_tree.remove_material_swept_cylinder(&segment, 8.0);

    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}

#[test]
fn test_swept_cylinder_short_segment_reduces_endpoint_overcut() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let mut tree_capsule = VoxelOctree::new(bounds, 6);
    let mut tree_swept = VoxelOctree::new(bounds, 6);

    let initial_volume = tree_capsule.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(50.0, 50.0, 50.0),
        Point3D::new(55.0, 50.0, 50.0),
    )
    .unwrap();

    tree_capsule.remove_material_capsule(&segment, 8.0);
    tree_swept.remove_material_swept_cylinder(&segment, 8.0);

    let removed_capsule = initial_volume - tree_capsule.remaining_volume();
    let removed_swept = initial_volume - tree_swept.remaining_volume();

    assert!(removed_swept > 0.0);
    assert!(removed_capsule > removed_swept);
}

#[test]
fn test_swept_cylinder_coarse_leaf_partial_overlap_does_not_remove_all() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 0);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(90.0, 90.0, 90.0),
        Point3D::new(120.0, 120.0, 120.0),
    )
    .unwrap();
    voxel_tree.remove_material_swept_cylinder(&segment, 5.0);

    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}
