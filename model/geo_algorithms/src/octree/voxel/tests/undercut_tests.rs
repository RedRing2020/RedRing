use super::super::*;
use geo_core::Point3D;

#[test]
fn test_detect_undercut_no_removal() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel_tree = VoxelOctree::new(bounds, 4);

    let target = Aabb3D::new(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(90.0, 90.0, 90.0),
    );

    let undercuts = voxel_tree.detect_undercut(&target);
    assert!(!undercuts.is_empty());
}

#[test]
fn test_detect_undercut_complete_removal() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    voxel_tree.remove_material_box(&bounds);

    let target = Aabb3D::new(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(90.0, 90.0, 90.0),
    );

    let undercuts = voxel_tree.detect_undercut(&target);
    assert!(undercuts.is_empty());
}

#[test]
fn test_detect_undercut_partial_removal() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let removed_region = Aabb3D::new(
        Point3D::new(20.0, 20.0, 20.0),
        Point3D::new(80.0, 80.0, 80.0),
    );
    voxel_tree.remove_material_box(&removed_region);

    let target = removed_region;
    let undercuts = voxel_tree.detect_undercut(&target);
    assert!(!undercuts.is_empty());

    for undercut_bbox in &undercuts {
        assert!(!undercut_bbox.intersects(&target));
    }
}

#[test]
fn test_detect_undercut_z_axis_tool() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 5);

    voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 20.0);

    let target = Aabb3D::new(
        Point3D::new(30.0, 30.0, 0.0),
        Point3D::new(70.0, 70.0, 100.0),
    );

    let undercuts = voxel_tree.detect_undercut(&target);
    assert!(!undercuts.is_empty());
}

#[test]
fn test_detect_undercut_target_larger_than_work() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let voxel_tree = VoxelOctree::new(bounds, 4);

    let target = Aabb3D::new(
        Point3D::new(-50.0, -50.0, -50.0),
        Point3D::new(150.0, 150.0, 150.0),
    );

    let undercuts = voxel_tree.detect_undercut(&target);
    assert!(undercuts.is_empty());
}
