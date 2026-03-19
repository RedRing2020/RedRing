//! EllipsoidalSurface3D の補助テスト

#[cfg(test)]
mod tests {
    use crate::{EllipsoidalSurface3D, Point3D};
    use geo_contracts::BasicCollision;

    #[test]
    fn test_distance_to_surface_at_center() {
        let surface = EllipsoidalSurface3D::new_at_origin(2.0, 3.0, 4.0).unwrap();
        let center = Point3D::origin();
        let distance = surface.distance_to_surface(&center);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_collision_with_surface_point() {
        let surface = EllipsoidalSurface3D::new_at_origin(2.0, 2.0, 2.0).unwrap();
        let point_on_surface = Point3D::new(2.0, 0.0, 0.0);
        assert!(surface.intersects(&point_on_surface, 1e-8));
    }
}
