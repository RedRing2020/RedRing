use super::super::*;
use geo_core::Point3D;

#[test]
fn test_voxel_octree_creation() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel_tree = VoxelOctree::new(bounds, 3);

    assert_eq!(voxel_tree.max_depth(), 3);
    assert_eq!(voxel_tree.solid_voxel_count(), 1);
    assert!((voxel_tree.remaining_volume() - 1000000.0).abs() < 0.001);
}

#[test]
fn test_remove_material_box_complete() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 3);

    let tool_aabb = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    voxel_tree.remove_material_box(&tool_aabb);

    assert_eq!(voxel_tree.remaining_volume(), 0.0);
    assert_eq!(voxel_tree.solid_voxel_count(), 0);
}

#[test]
fn test_remove_material_box_partial() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let tool_aabb = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(50.0, 50.0, 50.0));
    voxel_tree.remove_material_box(&tool_aabb);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(remaining > initial_volume * 0.5);
}

#[test]
fn test_remove_material_box_subdivision() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 5);

    let initial_volume = voxel_tree.remaining_volume();
    let tool_aabb = Aabb3D::new(
        Point3D::new(40.0, 40.0, 40.0),
        Point3D::new(60.0, 60.0, 60.0),
    );
    voxel_tree.remove_material_box(&tool_aabb);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(remaining > initial_volume * 0.95);
}

#[test]
fn test_multiple_removals() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();

    let tool1 = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(30.0, 30.0, 30.0));
    voxel_tree.remove_material_box(&tool1);

    let volume_after_1 = voxel_tree.remaining_volume();
    assert!(volume_after_1 < initial_volume);

    let tool2 = Aabb3D::new(
        Point3D::new(70.0, 70.0, 70.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    voxel_tree.remove_material_box(&tool2);

    let volume_after_2 = voxel_tree.remaining_volume();
    assert!(volume_after_2 < volume_after_1);
}

#[test]
fn test_no_intersection() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 3);

    let initial_volume = voxel_tree.remaining_volume();
    let tool_aabb = Aabb3D::new(
        Point3D::new(200.0, 200.0, 200.0),
        Point3D::new(300.0, 300.0, 300.0),
    );
    voxel_tree.remove_material_box(&tool_aabb);

    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}

#[test]
fn test_remove_material_box_leaf_partial_overlap_does_not_remove_all() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 0);

    let initial_volume = voxel_tree.remaining_volume();
    let tool_aabb = Aabb3D::new(
        Point3D::new(90.0, 90.0, 90.0),
        Point3D::new(110.0, 110.0, 110.0),
    );
    voxel_tree.remove_material_box(&tool_aabb);

    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}
