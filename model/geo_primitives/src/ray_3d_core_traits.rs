//! Ray3D Core Traits Implementation
//!
//! Foundation Pattern に基づく Ray3D の Core traits 実装
//! 統一された3つのCore機能（Constructor/Properties/Measure）を提供

use crate::{Point3D, Ray3D, Vector3D};
use analysis::linalg::{point3::Point3, vector::Vector3};
use geo_foundation::{
    core::ray_core_traits::{Ray3DConstructor, Ray3DMeasure, Ray3DProperties},
    Scalar,
};

// ============================================================================
// Ray3DConstructor トレイト実装
// ============================================================================

impl<T: Scalar> Ray3DConstructor<T> for Ray3D<T> {
    fn new(origin: Point3<T>, direction: Vector3<T>) -> Option<Self>
    where
        Self: Sized,
    {
        // Vector3からDirection3Dを作成
        let direction_vector = Vector3D::new(direction.x(), direction.y(), direction.z());
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());

        Ray3D::new(origin_point, direction_vector)
    }

    fn from_points(start: Point3<T>, through: Point3<T>) -> Option<Self>
    where
        Self: Sized,
    {
        let start_point = Point3D::new(start.x(), start.y(), start.z());
        let through_point = Point3D::new(through.x(), through.y(), through.z());

        Ray3D::from_points(start_point, through_point)
    }

    fn along_positive_x(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::along_x_axis(origin_point)
    }

    fn along_positive_y(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::along_y_axis(origin_point)
    }

    fn along_positive_z(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        Ray3D::along_z_axis(origin_point)
    }

    fn along_negative_x(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        let neg_x_direction = Vector3D::new(-T::ONE, T::ZERO, T::ZERO);
        Ray3D::new(origin_point, neg_x_direction).unwrap()
    }

    fn along_negative_y(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        let neg_y_direction = Vector3D::new(T::ZERO, -T::ONE, T::ZERO);
        Ray3D::new(origin_point, neg_y_direction).unwrap()
    }

    fn along_negative_z(origin: Point3<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point3D::new(origin.x(), origin.y(), origin.z());
        let neg_z_direction = Vector3D::new(T::ZERO, T::ZERO, -T::ONE);
        Ray3D::new(origin_point, neg_z_direction).unwrap()
    }

    fn x_axis() -> Self
    where
        Self: Sized,
    {
        Ray3D::along_x_axis(Point3D::origin())
    }

    fn y_axis() -> Self
    where
        Self: Sized,
    {
        Ray3D::along_y_axis(Point3D::origin())
    }

    fn z_axis() -> Self
    where
        Self: Sized,
    {
        Ray3D::along_z_axis(Point3D::origin())
    }
}

// ============================================================================
// Ray3DProperties トレイト実装
// ============================================================================

impl<T: Scalar> Ray3DProperties<T> for Ray3D<T> {
    fn origin(&self) -> Point3<T> {
        let origin = self.origin();
        Point3::new(origin.x(), origin.y(), origin.z())
    }

    fn direction(&self) -> Vector3<T> {
        let direction = self.direction_vector();
        Vector3::new(direction.x(), direction.y(), direction.z())
    }

    fn origin_x(&self) -> T {
        self.origin().x()
    }

    fn origin_y(&self) -> T {
        self.origin().y()
    }

    fn origin_z(&self) -> T {
        self.origin().z()
    }

    fn direction_x(&self) -> T {
        self.direction_vector().x()
    }

    fn direction_y(&self) -> T {
        self.direction_vector().y()
    }

    fn direction_z(&self) -> T {
        self.direction_vector().z()
    }

    fn is_valid(&self) -> bool {
        // Ray3D::new がSomeを返した時点で有効性は保証されている
        true
    }
}

// ============================================================================
// Ray3DMeasure トレイト実装
// ============================================================================

impl<T: Scalar> Ray3DMeasure<T> for Ray3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3<T> {
        let point = self.point_at_parameter(t);
        Point3::new(point.x(), point.y(), point.z())
    }

    fn closest_point(&self, point: &Point3<T>) -> Point3<T> {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        let closest = self.closest_point_on_ray(&target_point);
        Point3::new(closest.x(), closest.y(), closest.z())
    }

    fn distance_to_point(&self, point: &Point3<T>) -> T {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        self.distance_to_point(&target_point)
    }

    fn contains_point(&self, point: &Point3<T>) -> bool {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        use geo_foundation::tolerance_migration::DefaultTolerances;
        self.contains_point(&target_point, DefaultTolerances::distance::<T>())
    }

    fn parameter_for_point(&self, point: &Point3<T>) -> T {
        let target_point = Point3D::new(point.x(), point.y(), point.z());
        self.parameter_for_point(&target_point)
    }

    fn points_towards(&self, direction: &Vector3<T>) -> bool {
        let target_direction = Vector3D::new(direction.x(), direction.y(), direction.z());
        let dot = self.direction_vector().dot(&target_direction);
        dot > T::ZERO
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        let this_dir = self.direction_vector();
        let other_dir = other.direction_vector();

        // ベクトルの外積が0に近い場合は平行
        let cross = this_dir.cross(&other_dir);
        use geo_foundation::tolerance_migration::DefaultTolerances;
        cross.length() < DefaultTolerances::distance::<T>()
    }

    fn is_same_direction(&self, other: &Self) -> bool {
        if !self.is_parallel_to(other) {
            return false;
        }

        let dot = self.direction_vector().dot(&other.direction_vector());
        dot > T::ZERO
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        if !self.is_parallel_to(other) {
            return false;
        }

        let dot = self.direction_vector().dot(&other.direction_vector());
        dot < T::ZERO
    }

    fn reverse(&self) -> Self
    where
        Self: Sized,
    {
        self.reverse_direction()
    }

    fn translate(&self, offset: Vector3<T>) -> Self
    where
        Self: Sized,
    {
        let offset_vector = Vector3D::new(offset.x(), offset.y(), offset.z());
        let new_origin = self.origin() + offset_vector;

        Ray3D::new(new_origin, self.direction_vector()).unwrap()
    }
}

// ============================================================================
// Ray3DCore 統合トレイト実装（自動実装）
// Foundation層で自動実装されるため、ここでは明示的実装は不要
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::tolerance_migration::DefaultTolerances;

    #[test]
    fn test_ray3d_constructor() {
        // Origin とベクトルから作成（Core Traitsを使用）
        let origin = Point3::<f64>::new(1.0, 2.0, 3.0);
        let direction = Vector3::<f64>::new(1.0, 0.0, 0.0);
        let ray: Ray3D<f64> = Ray3DConstructor::new(origin, direction).unwrap();

        let expected_origin = Ray3DProperties::origin(&ray);
        assert_eq!(expected_origin.x(), 1.0);
        assert_eq!(expected_origin.y(), 2.0);
        assert_eq!(expected_origin.z(), 3.0);

        // 2点から作成
        let start = Point3::<f64>::new(0.0, 0.0, 0.0);
        let through = Point3::<f64>::new(1.0, 1.0, 1.0);
        let ray2: Ray3D<f64> = Ray3DConstructor::from_points(start, through).unwrap();

        assert!(Ray3DProperties::is_valid(&ray2));

        // 軸方向Ray作成
        let x_ray: Ray3D<f64> = Ray3DConstructor::x_axis();
        let y_ray: Ray3D<f64> = Ray3DConstructor::y_axis();
        let z_ray: Ray3D<f64> = Ray3DConstructor::z_axis();

        assert_eq!(Ray3DProperties::origin_x(&x_ray), 0.0);
        assert_eq!(Ray3DProperties::direction_x(&x_ray), 1.0);
        assert_eq!(Ray3DProperties::direction_y(&y_ray), 1.0);
        assert_eq!(Ray3DProperties::direction_z(&z_ray), 1.0);
    }

    #[test]
    fn test_ray3d_properties() {
        let ray = Ray3D::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        // 起点取得
        assert_eq!(Ray3DProperties::origin_x(&ray), 1.0);
        assert_eq!(Ray3DProperties::origin_y(&ray), 2.0);
        assert_eq!(Ray3DProperties::origin_z(&ray), 3.0);

        // 方向取得
        assert_eq!(Ray3DProperties::direction_x(&ray), 1.0);
        assert_eq!(Ray3DProperties::direction_y(&ray), 0.0);
        assert_eq!(Ray3DProperties::direction_z(&ray), 0.0);

        // 有効性チェック
        assert!(Ray3DProperties::is_valid(&ray));
        assert_eq!(Ray3DProperties::dimension(&ray), 3);
    }

    #[test]
    fn test_ray3d_measure() {
        let ray = Ray3D::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        // パラメータt での点
        let point_at_2 = Ray3DMeasure::point_at_parameter(&ray, 2.0);
        assert_eq!(point_at_2.x(), 3.0);
        assert_eq!(point_at_2.y(), 2.0);
        assert_eq!(point_at_2.z(), 3.0);

        // 距離計算
        let test_point = Point3::new(1.0, 3.0, 3.0);
        let distance = Ray3DMeasure::distance_to_point(&ray, &test_point);
        assert!((distance - 1.0).abs() < DefaultTolerances::distance::<f64>());

        // 平行性チェック
        let parallel_ray = Ray3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(2.0, 0.0, 0.0), // 同じ方向だが倍率が違う
        )
        .unwrap();

        assert!(Ray3DMeasure::is_parallel_to(&ray, &parallel_ray));
        assert!(Ray3DMeasure::is_same_direction(&ray, &parallel_ray));

        // 逆方向チェック
        let reverse_ray = Ray3DMeasure::reverse(&ray);
        assert!(Ray3DMeasure::is_opposite_direction(&ray, &reverse_ray));

        // 平行移動
        let offset = Vector3::new(1.0, 1.0, 1.0);
        let translated = Ray3DMeasure::translate(&ray, offset);
        assert_eq!(Ray3DProperties::origin_x(&translated), 2.0);
        assert_eq!(Ray3DProperties::origin_y(&translated), 3.0);
        assert_eq!(Ray3DProperties::origin_z(&translated), 4.0);
    }

    #[test]
    fn test_ray3d_core_integration() {
        // Core traitの統合テスト
        let ray: Ray3D<f64> = Ray3DConstructor::x_axis();

        // Constructorメソッド経由での作成
        let origin = Point3::new(1.0, 0.0, 0.0);
        let ray2: Ray3D<f64> = Ray3DConstructor::along_positive_x(origin);

        // Measureメソッド経由での計算
        assert!(Ray3DMeasure::is_parallel_to(&ray, &ray2));
        assert!(Ray3DMeasure::is_same_direction(&ray, &ray2));
    }
}
