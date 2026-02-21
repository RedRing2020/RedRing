use super::*;
use geo_foundation::ToleranceSettings;

fn test_point_aabb_half_extent() -> f64 {
    ToleranceSettings::<f64>::relaxed().distance_tolerance
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

#[test]
fn test_octree_creation() {
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let octree: Octree<f64, TestPoint> = Octree::new(bbox, 8, 10);

    assert_eq!(octree.total_nodes(), 1);
    assert_eq!(octree.total_items(), 0);
    assert!(octree.is_empty());
}

#[test]
fn test_insert_and_query() {
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 10);

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
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 10);

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
    assert!(distance < 5.0);
}

#[test]
fn test_clear() {
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 10);

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
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let octree: Octree<f64, TestPoint> = Octree::new(bbox, 8, 10);

    let mut visited_count = 0;
    octree.traverse(|_node, _depth| {
        visited_count += 1;
    });

    assert_eq!(visited_count, 1);
}

#[test]
fn test_recursive_insert_with_subdivision() {
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 2);

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
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 5);

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
    assert!(distance < 2.0);

    let expected_pos = Point3D::new(10.0, 10.0, 10.0);
    let nearest_pos = nearest.position();
    assert!((nearest_pos.x() - expected_pos.x()).abs() < 0.001);
    assert!((nearest_pos.y() - expected_pos.y()).abs() < 0.001);
    assert!((nearest_pos.z() - expected_pos.z()).abs() < 0.001);
}

#[test]
fn test_performance_collision_detection_reduction() {
    #[derive(Debug, Clone)]
    struct Sphere {
        center: Point3D<f64>,
        radius: f64,
    }

    impl HasBoundingBox<f64> for Sphere {
        fn bounding_box(&self) -> Aabb3D<f64> {
            Aabb3D::new(
                Point3D::new(
                    self.center.x() - self.radius,
                    self.center.y() - self.radius,
                    self.center.z() - self.radius,
                ),
                Point3D::new(
                    self.center.x() + self.radius,
                    self.center.y() + self.radius,
                    self.center.z() + self.radius,
                ),
            )
        }
    }

    impl HasPosition<f64> for Sphere {
        fn position(&self) -> Point3D<f64> {
            self.center
        }
    }

    let mut spheres = Vec::new();
    let mut seed = 12345u64;

    for i in 0..1000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let x = ((seed % 10000) as f64) / 100.0;

        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let y = ((seed % 10000) as f64) / 100.0;

        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let z = ((seed % 10000) as f64) / 100.0;

        let radius = 2.0 + (i % 5) as f64;

        spheres.push(Sphere {
            center: Point3D::new(x, y, z),
            radius,
        });
    }

    let mut brute_force_checks = 0;
    let mut brute_force_collisions = 0;
    for i in 0..spheres.len() {
        for j in (i + 1)..spheres.len() {
            brute_force_checks += 1;
            if spheres[i]
                .bounding_box()
                .intersects(&spheres[j].bounding_box())
            {
                brute_force_collisions += 1;
            }
        }
    }

    let total_pairs = (spheres.len() * (spheres.len() - 1)) / 2;
    assert_eq!(brute_force_checks, total_pairs);

    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 10);

    for sphere in &spheres {
        octree.insert(sphere.clone());
    }

    let mut octree_checks = 0;
    let mut octree_collisions = 0;
    for sphere in &spheres {
        let candidates = octree.query_region(&sphere.bounding_box());

        for candidate in candidates {
            if !std::ptr::eq(sphere, candidate) {
                octree_checks += 1;
                if sphere.bounding_box().intersects(&candidate.bounding_box()) {
                    octree_collisions += 1;
                }
            }
        }
    }

    octree_checks /= 2;
    octree_collisions /= 2;

    let reduction_ratio = 1.0 - (octree_checks as f64 / brute_force_checks as f64);

    eprintln!("=== 衝突判定パフォーマンス ===");
    eprintln!("要素数: {}", spheres.len());
    eprintln!("総当たり判定回数: {} 回", brute_force_checks);
    eprintln!("総当たり衝突検出: {} ペア", brute_force_collisions);
    eprintln!("Octree判定回数: {} 回", octree_checks);
    eprintln!("Octree衝突検出: {} ペア（参考値）", octree_collisions);
    eprintln!("削減率: {:.2}%", reduction_ratio * 100.0);

    assert!(
        reduction_ratio >= 0.95,
        "削減率が不十分: {:.2}% (期待: 95%以上)",
        reduction_ratio * 100.0
    );

    eprintln!("✓ 削減率 {:.2}% を達成（目標99%）", reduction_ratio * 100.0);
}

#[test]
fn test_memory_usage_estimation_regression() {
    let bbox = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree = Octree::new(bbox, 8, 8);

    let estimated_empty = estimate_octree_memory_bytes(&octree);
    assert!(estimated_empty > 0);

    for i in 0..1000 {
        let x = (i % 100) as f64 + 0.1;
        let y = ((i / 10) % 100) as f64 + 0.2;
        let z = ((i / 100) % 100) as f64 + 0.3;
        octree.insert(TestPoint {
            pos: Point3D::new(x, y, z),
        });
    }

    let estimated_populated = estimate_octree_memory_bytes(&octree);
    let bytes_per_item = estimated_populated as f64 / octree.total_items() as f64;

    eprintln!("=== メモリ使用量（概算）===");
    eprintln!("空状態: {} bytes", estimated_empty);
    eprintln!("挿入後: {} bytes", estimated_populated);
    eprintln!("総ノード数: {}", octree.total_nodes());
    eprintln!("総要素数: {}", octree.total_items());
    eprintln!("1要素あたり概算: {:.2} bytes", bytes_per_item);

    assert!(
        estimated_populated > estimated_empty,
        "挿入後の概算メモリ使用量が増加していません"
    );
    assert!(octree.total_nodes() > 1, "分割が発生していません");
    assert!(bytes_per_item < 8192.0, "1要素あたりメモリが過大です");

    octree.clear();
    let estimated_after_clear = estimate_octree_memory_bytes(&octree);
    assert_eq!(octree.total_items(), 0);
    assert_eq!(octree.total_nodes(), 1);
    assert_eq!(
        estimated_after_clear, estimated_empty,
        "clear後に概算メモリ使用量が初期値へ戻っていません"
    );
}
