use super::*;

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
    let mut seed = LCG_INITIAL_SEED;

    for i in 0..PERFORMANCE_SAMPLE_COUNT {
        seed = seed
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        let x = ((seed % LCG_VALUE_MOD) as f64) / POSITION_SCALE_DIVISOR;

        seed = seed
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        let y = ((seed % LCG_VALUE_MOD) as f64) / POSITION_SCALE_DIVISOR;

        seed = seed
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        let z = ((seed % LCG_VALUE_MOD) as f64) / POSITION_SCALE_DIVISOR;

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

    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_DEFAULT_MAX_ITEMS,
    );

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
        reduction_ratio >= COLLISION_REDUCTION_RATIO_MIN,
        "削減率が不十分: {:.2}% (期待: 95%以上)",
        reduction_ratio * 100.0
    );

    eprintln!("✓ 削減率 {:.2}% を達成（目標99%）", reduction_ratio * 100.0);
}
