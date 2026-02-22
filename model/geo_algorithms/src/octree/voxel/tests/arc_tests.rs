use super::super::*;
use geo_core::Point3D;
use geo_primitives::{Angle, Arc3D, LineSegment3D};

#[test]
fn test_arc_polyline_removal_basic() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 50.0),
        20.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(90.0),
    )
    .unwrap();

    voxel_tree.remove_material_arc_polyline(&arc, 5.0, 8);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
    assert!(initial_volume - remaining > 1000.0);
}

#[test]
fn test_arc_polyline_removal_convergence() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 50.0),
        30.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(180.0),
    )
    .unwrap();

    let mut tree_8seg = VoxelOctree::new(bounds, 6);
    tree_8seg.remove_material_arc_polyline(&arc, 8.0, 8);
    let volume_8seg = tree_8seg.remaining_volume();

    let mut tree_16seg = VoxelOctree::new(bounds, 6);
    tree_16seg.remove_material_arc_polyline(&arc, 8.0, 16);
    let volume_16seg = tree_16seg.remaining_volume();

    let mut tree_32seg = VoxelOctree::new(bounds, 6);
    tree_32seg.remove_material_arc_polyline(&arc, 8.0, 32);
    let volume_32seg = tree_32seg.remaining_volume();

    assert!(volume_16seg <= volume_8seg);
    assert!(volume_32seg <= volume_16seg);
    assert!(volume_32seg < bounds.volume());
}

#[test]
fn test_arc_polyline_removal_full_circle() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 50.0),
        15.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(360.0),
    )
    .unwrap();

    voxel_tree.remove_material_arc_polyline(&arc, 5.0, 16);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
}

#[test]
fn test_arc_polyline_removal_small_arc() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 5);

    let initial_volume = voxel_tree.remaining_volume();
    let arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 50.0),
        10.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(10.0),
    )
    .unwrap();

    voxel_tree.remove_material_arc_polyline(&arc, 3.0, 4);

    let remaining = voxel_tree.remaining_volume();
    assert!(remaining < initial_volume);
}

#[test]
fn test_arc_polyline_removal_no_intersection() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let arc = Arc3D::xy_arc(
        Point3D::new(200.0, 200.0, 200.0),
        20.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(90.0),
    )
    .unwrap();

    voxel_tree.remove_material_arc_polyline(&arc, 5.0, 8);
    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}

#[test]
fn test_arc_polyline_removal_zero_segments() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut voxel_tree = VoxelOctree::new(bounds, 4);

    let initial_volume = voxel_tree.remaining_volume();
    let arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 50.0),
        20.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(90.0),
    )
    .unwrap();

    voxel_tree.remove_material_arc_polyline(&arc, 5.0, 0);
    assert_eq!(voxel_tree.remaining_volume(), initial_volume);
}

#[test]
fn test_arc_polyline_vs_straight_segment() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let straight_arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 1000.0),
        1000.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(0.01),
    )
    .unwrap();

    let start = straight_arc.start_point();
    let end = straight_arc.end_point();
    let segment = LineSegment3D::new(start, end).unwrap();

    let mut tree_arc = VoxelOctree::new(bounds, 5);
    tree_arc.remove_material_arc_polyline(&straight_arc, 5.0, 16);

    let mut tree_seg = VoxelOctree::new(bounds, 5);
    tree_seg.remove_material_capsule(&segment, 5.0);

    let diff = (tree_arc.remaining_volume() - tree_seg.remaining_volume()).abs();
    let total = bounds.volume();
    assert!(diff / total < 0.01);
}