use geo_algorithms::octree::{HasBoundingBox, HasPosition, Octree, OctreeTolerance};
use geo_core::{Aabb3D, Point3D};
use geo_foundation::ToleranceSettings;

fn test_point_aabb_half_extent() -> f64 {
    ToleranceSettings::<f64>::relaxed().distance_tolerance
}

#[derive(Debug, Clone)]
struct TestPoint {
    id: usize,
    pos: Point3D<f64>,
}

impl HasBoundingBox<f64> for TestPoint {
    fn bounding_box(&self) -> Aabb3D<f64> {
        let half_extent = test_point_aabb_half_extent();
        Aabb3D::new(
            Point3D::new(
                self.pos.x() - half_extent,
                self.pos.y() - half_extent,
                self.pos.z() - half_extent,
            ),
            Point3D::new(
                self.pos.x() + half_extent,
                self.pos.y() + half_extent,
                self.pos.z() + half_extent,
            ),
        )
    }
}

impl HasPosition<f64> for TestPoint {
    fn position(&self) -> Point3D<f64> {
        self.pos
    }
}

fn sample_octree() -> Octree<f64, TestPoint> {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let base = test_point_aabb_half_extent();
    let tolerance = OctreeTolerance::new(base, base, base * 0.5);
    let mut octree = Octree::with_tolerance(bounds, 8, 8, tolerance);

    for i in 0..200 {
        let x = (i * 37 % 100) as f64 + 0.25;
        let y = (i * 53 % 100) as f64 + 0.50;
        let z = (i * 91 % 100) as f64 + 0.75;
        octree.insert(TestPoint {
            id: i,
            pos: Point3D::new(x, y, z),
        });
    }

    octree
}

#[test]
fn query_regions_parallel_matches_sequential() {
    let octree = sample_octree();
    let regions = vec![
        Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(25.0, 25.0, 25.0)),
        Aabb3D::new(
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(40.0, 40.0, 40.0),
        ),
        Aabb3D::new(
            Point3D::new(30.0, 30.0, 30.0),
            Point3D::new(80.0, 80.0, 80.0),
        ),
        Aabb3D::new(
            Point3D::new(70.0, 70.0, 70.0),
            Point3D::new(99.0, 99.0, 99.0),
        ),
    ];

    let sequential: Vec<Vec<&TestPoint>> = regions.iter().map(|r| octree.query_region(r)).collect();
    let parallel = octree.query_regions_parallel(&regions);

    assert_eq!(sequential.len(), parallel.len());

    for (seq, par) in sequential.iter().zip(parallel.iter()) {
        let mut seq_ids: Vec<usize> = seq.iter().map(|p| p.id).collect();
        let mut par_ids: Vec<usize> = par.iter().map(|p| p.id).collect();
        seq_ids.sort_unstable();
        par_ids.sort_unstable();
        assert_eq!(seq_ids, par_ids);
    }
}

#[test]
fn nearest_many_parallel_matches_sequential() {
    let octree = sample_octree();
    let queries = vec![
        Point3D::new(1.0, 1.0, 1.0),
        Point3D::new(12.0, 45.0, 60.0),
        Point3D::new(35.0, 35.0, 35.0),
        Point3D::new(75.0, 12.0, 91.0),
        Point3D::new(99.0, 99.0, 99.0),
    ];

    let sequential: Vec<Option<(&TestPoint, f64)>> =
        queries.iter().map(|q| octree.nearest(q)).collect();
    let parallel = octree.nearest_many_parallel(&queries);

    assert_eq!(sequential.len(), parallel.len());

    for (seq, par) in sequential.iter().zip(parallel.iter()) {
        match (seq, par) {
            (Some((s_item, s_dist)), Some((p_item, p_dist))) => {
                assert_eq!(s_item.id, p_item.id);
                assert!((s_dist - p_dist).abs() < 1e-9);
            }
            (None, None) => {}
            _ => panic!("sequential と parallel の結果が不一致"),
        }
    }
}
