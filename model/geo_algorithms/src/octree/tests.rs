use super::*;
use geo_foundation::ToleranceSettings;

const OCTREE_TEST_BOUNDS_MIN: f64 = 0.0;
const OCTREE_TEST_BOUNDS_MAX: f64 = 100.0;
const OCTREE_DEFAULT_MAX_DEPTH: usize = 8;
const OCTREE_DEFAULT_MAX_ITEMS: usize = 10;
const OCTREE_SUBDIVIDE_MAX_ITEMS: usize = 2;
const OCTREE_OPTIMIZED_NEAREST_MAX_ITEMS: usize = 5;
const OCTREE_MEMORY_TEST_MAX_ITEMS: usize = 8;

const NEAREST_DISTANCE_MAX: f64 = 5.0;
const OPTIMIZED_NEAREST_DISTANCE_MAX: f64 = 2.0;
const COLLISION_REDUCTION_RATIO_MIN: f64 = 0.95;
const MEMORY_BYTES_PER_ITEM_MAX: f64 = 8192.0;

const PERFORMANCE_SAMPLE_COUNT: usize = 1000;
const LCG_INITIAL_SEED: u64 = 12_345;
const LCG_MULTIPLIER: u64 = 1_103_515_245;
const LCG_INCREMENT: u64 = 12_345;
const LCG_VALUE_MOD: u64 = 10_000;
const POSITION_SCALE_DIVISOR: f64 = 100.0;

fn test_point_aabb_half_extent() -> f64 {
    ToleranceSettings::<f64>::relaxed().distance_tolerance
}

fn default_octree_bounds() -> Aabb3D<f64> {
    Aabb3D::new(
        Point3D::new(
            OCTREE_TEST_BOUNDS_MIN,
            OCTREE_TEST_BOUNDS_MIN,
            OCTREE_TEST_BOUNDS_MIN,
        ),
        Point3D::new(
            OCTREE_TEST_BOUNDS_MAX,
            OCTREE_TEST_BOUNDS_MAX,
            OCTREE_TEST_BOUNDS_MAX,
        ),
    )
}

#[derive(Debug, Clone)]
struct TestPoint {
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

fn estimate_octree_memory_bytes(octree: &Octree<f64, TestPoint>) -> usize {
    octree.total_nodes() * std::mem::size_of::<OctreeNode<f64, TestPoint>>()
        + octree.total_items() * std::mem::size_of::<TestPoint>()
}

mod basic_tests;
mod memory_tests;
mod performance_tests;
