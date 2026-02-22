use super::*;

#[test]
fn test_octree_creation() {
    let octree: Octree<f64, TestPoint> = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_DEFAULT_MAX_ITEMS,
    );

    assert_eq!(octree.total_nodes(), 1);
    assert_eq!(octree.total_items(), 0);
    assert!(octree.is_empty());
}

#[test]
fn test_insert_and_query() {
    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_DEFAULT_MAX_ITEMS,
    );

    let point1 = TestPoint {
        pos: Point3D::new(10.0, 10.0, 10.0),
    };
    let point2 = TestPoint {
        pos: Point3D::new(50.0, 50.0, 50.0),
    };

    octree.insert(point1.clone());
    octree.insert(point2.clone());

    assert_eq!(octree.total_items(), 2);

    let query_bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(20.0, 20.0, 20.0));
    let results = octree.query_region(&query_bbox);

    assert!(!results.is_empty());
}

#[test]
fn test_nearest_search() {
    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_DEFAULT_MAX_ITEMS,
    );

    octree.insert(TestPoint {
        pos: Point3D::new(10.0, 10.0, 10.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(50.0, 50.0, 50.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(90.0, 90.0, 90.0),
    });

    let query_point = Point3D::new(12.0, 12.0, 12.0);
    let result = octree.nearest(&query_point);

    assert!(result.is_some());
    let (_nearest, distance) = result.unwrap();
    assert!(distance < NEAREST_DISTANCE_MAX);
}

#[test]
fn test_clear() {
    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_DEFAULT_MAX_ITEMS,
    );

    octree.insert(TestPoint {
        pos: Point3D::new(10.0, 10.0, 10.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(50.0, 50.0, 50.0),
    });

    assert_eq!(octree.total_items(), 2);

    octree.clear();

    assert_eq!(octree.total_items(), 0);
    assert_eq!(octree.total_nodes(), 1);
    assert!(octree.is_empty());
}

#[test]
fn test_traverse() {
    let octree: Octree<f64, TestPoint> = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_DEFAULT_MAX_ITEMS,
    );

    let mut visited_count = 0;
    octree.traverse(|_node, _depth| {
        visited_count += 1;
    });

    assert_eq!(visited_count, 1);
}

#[test]
fn test_recursive_insert_with_subdivision() {
    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_SUBDIVIDE_MAX_ITEMS,
    );

    octree.insert(TestPoint {
        pos: Point3D::new(10.0, 10.0, 10.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(11.0, 11.0, 11.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(12.0, 12.0, 12.0),
    });

    assert_eq!(octree.total_items(), 3);
    assert!(octree.total_nodes() > 1);
}

#[test]
fn test_optimized_nearest_search() {
    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_OPTIMIZED_NEAREST_MAX_ITEMS,
    );

    octree.insert(TestPoint {
        pos: Point3D::new(10.0, 10.0, 10.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(50.0, 50.0, 50.0),
    });
    octree.insert(TestPoint {
        pos: Point3D::new(90.0, 90.0, 90.0),
    });

    let query_point = Point3D::new(11.0, 11.0, 11.0);
    let result = octree.nearest(&query_point);

    assert!(result.is_some());
    let (nearest, distance) = result.unwrap();
    assert!(distance < OPTIMIZED_NEAREST_DISTANCE_MAX);

    let expected_pos = Point3D::new(10.0, 10.0, 10.0);
    let nearest_pos = nearest.position();
    let position_tolerance = test_point_aabb_half_extent();
    assert!((nearest_pos.x() - expected_pos.x()).abs() < position_tolerance);
    assert!((nearest_pos.y() - expected_pos.y()).abs() < position_tolerance);
    assert!((nearest_pos.z() - expected_pos.z()).abs() < position_tolerance);
}
