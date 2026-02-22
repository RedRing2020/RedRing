use super::super::*;
use geo_core::Point3D;
use geo_primitives::LineSegment3D;

#[test]
fn test_capsule_removal_basic() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(50.0, 50.0, 0.0),
        Point3D::new(50.0, 50.0, 100.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment, 10.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(initial_volume - remaining > 20000.0);
}

#[test]
fn test_capsule_removal_diagonal() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment, 5.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
}

#[test]
fn test_capsule_removal_no_intersection() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(200.0, 200.0, 200.0),
        Point3D::new(300.0, 300.0, 300.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment, 10.0);

    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}

#[test]
fn test_capsule_removal_small_radius() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 5);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(50.0, 50.0, 0.0),
        Point3D::new(50.0, 50.0, 100.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment, 1.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(initial_volume - remaining > 100.0);
}

#[test]
fn test_capsule_removal_multiple_segments() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();

    let segment1 = LineSegment3D::new(
        Point3D::new(25.0, 25.0, 0.0),
        Point3D::new(25.0, 25.0, 100.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment1, 8.0);

    let volume_after_1 = voxel_tree.remaining_volume();
    assert!(volume_after_1 < initial_volume);

    let segment2 = LineSegment3D::new(
        Point3D::new(75.0, 75.0, 0.0),
        Point3D::new(75.0, 75.0, 100.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment2, 8.0);

    let volume_after_2 = voxel_tree.remaining_volume();
    assert!(volume_after_2 < volume_after_1);
}

#[test]
fn test_capsule_removal_horizontal() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(0.0, 50.0, 50.0),
        Point3D::new(100.0, 50.0, 50.0),
    )
    .unwrap();
    voxel_tree.remove_material_capsule(&segment, 10.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
}