use super::super::*;
use crate::LineSegment3D;
use geo_core::Point3D;

#[test]
fn test_z_axis_removal_basic() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(initial_volume - remaining > 20000.0);
}

#[test]
fn test_z_axis_removal_comparison_with_capsule() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let mut tree1 = VoxelOctree::new(bounds, 4);
    let segment = LineSegment3D::new(
        Point3D::new(50.0, 50.0, 0.0),
        Point3D::new(50.0, 50.0, 100.0),
    )
    .unwrap();
    tree1.remove_material_capsule(&segment, 10.0);
    let volume1 = tree1.remaining_volume();

    let mut tree2 = VoxelOctree::new(bounds, 4);
    tree2.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);
    let volume2 = tree2.remaining_volume();

    let diff = (volume1 - volume2).abs();
    assert!(diff < 1000.0);
}

#[test]
fn test_z_axis_removal_offset_center() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    voxel_tree.remove_material_z_axis(25.0, 75.0, 10.0, 90.0, 8.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
}

#[test]
fn test_z_axis_removal_no_intersection() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    voxel_tree.remove_material_z_axis(200.0, 200.0, 0.0, 100.0, 10.0);

    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}

#[test]
fn test_z_axis_removal_partial_z_range() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    voxel_tree.remove_material_z_axis(50.0, 50.0, 20.0, 60.0, 15.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);

    let removed_ratio = (initial_volume - remaining) / initial_volume;
    assert!(removed_ratio < 0.5);
}

#[test]
fn test_z_axis_removal_multiple_operations() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();

    voxel_tree.remove_material_z_axis(25.0, 25.0, 0.0, 100.0, 8.0);
    let volume_after_1 = voxel_tree.remaining_volume();
    assert!(volume_after_1 < initial_volume);

    voxel_tree.remove_material_z_axis(75.0, 75.0, 0.0, 100.0, 8.0);
    let volume_after_2 = voxel_tree.remaining_volume();
    assert!(volume_after_2 < volume_after_1);
}

#[test]
fn test_z_axis_removal_small_radius() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 5);

    let initial_volume = voxel_tree.remaining_volume();
    voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 1.0);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(initial_volume - remaining > 100.0);
}

#[test]
fn test_voxel_size_calculation() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let voxel_tree_3 = VoxelOctree::new(bounds, 3);
    let expected_size_3 = 100.0 / 8.0;
    assert!((voxel_tree_3.voxel_size_at_max_depth() - expected_size_3).abs() < 0.001);

    let voxel_tree_5 = VoxelOctree::new(bounds, 5);
    let expected_size_5 = 100.0 / 32.0;
    assert!((voxel_tree_5.voxel_size_at_max_depth() - expected_size_5).abs() < 0.001);
}
