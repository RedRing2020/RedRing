//! SphericalSolid3D の relation 回帰テスト

#[cfg(test)]
mod tests {
    use crate::{Point3D, SphericalSolid3D};
    use geo_contracts::IntersectsRelation;

    #[test]
    fn test_spherical_solid_intersection_is_exposed_via_relation_trait() {
        let sphere_a = SphericalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();
        let sphere_b = SphericalSolid3D::new_standard(Point3D::new(9.0, 0.0, 0.0), 5.0).unwrap();
        let sphere_c = SphericalSolid3D::new_standard(Point3D::new(11.0, 0.0, 0.0), 5.0).unwrap();

        assert!(IntersectsRelation::intersects(&sphere_a, &sphere_b));
        assert!(!IntersectsRelation::intersects(&sphere_a, &sphere_c));
    }

    #[test]
    fn test_spherical_solid_intersection_tangent_counts_as_intersecting() {
        let sphere_a = SphericalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();
        let sphere_b = SphericalSolid3D::new_standard(Point3D::new(10.0, 0.0, 0.0), 5.0).unwrap();

        assert!(IntersectsRelation::intersects(&sphere_a, &sphere_b));
    }
}
