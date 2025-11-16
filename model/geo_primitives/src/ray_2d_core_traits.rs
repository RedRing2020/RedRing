//! Ray2D Core Traits Implementation
//!
//! Foundation Pattern に基づく Ray2D の Core traits 実装
//! 統一された3つのCore機能（Constructor/Properties/Measure）を提供

use crate::{Point2D, Ray2D, Vector2D};
use analysis::linalg::{point2::Point2, vector::Vector2};
use geo_foundation::{
    core::ray_core_traits::{Ray2DConstructor, Ray2DMeasure, Ray2DProperties},
    Scalar,
};

// ============================================================================
// Ray2DConstructor トレイト実装
// ============================================================================

impl<T: Scalar> Ray2DConstructor<T> for Ray2D<T> {
    fn new(origin: Point2<T>, direction: Vector2<T>) -> Option<Self>
    where
        Self: Sized,
    {
        // Vector2からDirection2Dを作成
        let direction_vector = Vector2D::new(direction.x(), direction.y());
        let origin_point = Point2D::new(origin.x(), origin.y());

        Ray2D::new(origin_point, direction_vector)
    }

    fn from_points(start: Point2<T>, through: Point2<T>) -> Option<Self>
    where
        Self: Sized,
    {
        let start_point = Point2D::new(start.x(), start.y());
        let through_point = Point2D::new(through.x(), through.y());

        Ray2D::from_points(start_point, through_point)
    }

    fn along_positive_x(origin: Point2<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.x(), origin.y());
        let x_direction = Vector2D::new(T::ONE, T::ZERO);
        Ray2D::new(origin_point, x_direction).unwrap()
    }

    fn along_positive_y(origin: Point2<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.x(), origin.y());
        let y_direction = Vector2D::new(T::ZERO, T::ONE);
        Ray2D::new(origin_point, y_direction).unwrap()
    }

    fn along_negative_x(origin: Point2<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.x(), origin.y());
        let neg_x_direction = Vector2D::new(-T::ONE, T::ZERO);
        Ray2D::new(origin_point, neg_x_direction).unwrap()
    }

    fn along_negative_y(origin: Point2<T>) -> Self
    where
        Self: Sized,
    {
        let origin_point = Point2D::new(origin.x(), origin.y());
        let neg_y_direction = Vector2D::new(T::ZERO, -T::ONE);
        Ray2D::new(origin_point, neg_y_direction).unwrap()
    }

    fn x_axis() -> Self
    where
        Self: Sized,
    {
        Self::along_positive_x(Point2::origin())
    }

    fn y_axis() -> Self
    where
        Self: Sized,
    {
        Self::along_positive_y(Point2::origin())
    }
}

// ============================================================================
// Ray2DProperties トレイト実装
// ============================================================================

impl<T: Scalar> Ray2DProperties<T> for Ray2D<T> {
    fn origin(&self) -> Point2<T> {
        let origin = self.origin();
        Point2::new(origin.x(), origin.y())
    }

    fn direction(&self) -> Vector2<T> {
        let direction = self.direction();
        Vector2::new(direction.x(), direction.y())
    }

    fn origin_x(&self) -> T {
        self.origin().x()
    }

    fn origin_y(&self) -> T {
        self.origin().y()
    }

    fn direction_x(&self) -> T {
        self.direction().x()
    }

    fn direction_y(&self) -> T {
        self.direction().y()
    }

    fn is_valid(&self) -> bool {
        // Ray2D::new がSomeを返した時点で有効性は保証されている
        true
    }
}

// ============================================================================
// Ray2DMeasure トレイト実装
// ============================================================================

impl<T: Scalar> Ray2DMeasure<T> for Ray2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2<T> {
        let point = self.point_at_parameter(t);
        Point2::new(point.x(), point.y())
    }

    fn closest_point(&self, point: &Point2<T>) -> Point2<T> {
        let target_point = Point2D::new(point.x(), point.y());
        let t = self.parameter_for_point(&target_point);
        let clamped_t = if t < T::ZERO { T::ZERO } else { t };
        let closest = self.point_at_parameter(clamped_t);
        Point2::new(closest.x(), closest.y())
    }

    fn distance_to_point(&self, point: &Point2<T>) -> T {
        let target_point = Point2D::new(point.x(), point.y());
        self.distance_to_point(&target_point)
    }

    fn contains_point(&self, point: &Point2<T>) -> bool {
        let target_point = Point2D::new(point.x(), point.y());
        use geo_foundation::tolerance_migration::DefaultTolerances;
        self.contains_point(&target_point, DefaultTolerances::distance::<T>())
    }

    fn parameter_for_point(&self, point: &Point2<T>) -> T {
        let target_point = Point2D::new(point.x(), point.y());
        self.parameter_for_point(&target_point)
    }

    fn points_towards(&self, direction: &Vector2<T>) -> bool {
        let target_direction = Vector2D::new(direction.x(), direction.y());
        let self_direction = Vector2D::new(self.direction().x(), self.direction().y());
        let dot = self_direction.dot(&target_direction);
        dot > T::ZERO
    }

    fn is_parallel_to(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction().x(), self.direction().y());
        let other_dir = Vector2D::new(other.direction().x(), other.direction().y());

        // ベクトルの外積が0に近い場合は平行
        let cross = this_dir.cross(&other_dir);
        use geo_foundation::tolerance_migration::DefaultTolerances;
        cross.abs() < DefaultTolerances::distance::<T>()
    }

    fn is_same_direction(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction().x(), self.direction().y());
        let other_dir = Vector2D::new(other.direction().x(), other.direction().y());

        // まず平行性をチェック
        let cross = this_dir.cross(&other_dir);
        use geo_foundation::tolerance_migration::DefaultTolerances;
        if cross.abs() >= DefaultTolerances::distance::<T>() {
            return false;
        }

        // 平行な場合、内積で方向をチェック
        let dot = this_dir.dot(&other_dir);
        dot > T::ZERO
    }

    fn is_opposite_direction(&self, other: &Self) -> bool {
        let this_dir = Vector2D::new(self.direction().x(), self.direction().y());
        let other_dir = Vector2D::new(other.direction().x(), other.direction().y());

        // まず平行性をチェック
        let cross = this_dir.cross(&other_dir);
        use geo_foundation::tolerance_migration::DefaultTolerances;
        if cross.abs() >= DefaultTolerances::distance::<T>() {
            return false;
        }

        // 平行な場合、内積で方向をチェック
        let dot = this_dir.dot(&other_dir);
        dot < T::ZERO
    }

    fn reverse(&self) -> Self
    where
        Self: Sized,
    {
        let direction_vec = Vector2D::new(self.direction().x(), self.direction().y());
        let reversed_direction = -direction_vec;
        Ray2D::new(self.origin(), reversed_direction).unwrap()
    }

    fn translate(&self, offset: Vector2<T>) -> Self
    where
        Self: Sized,
    {
        let offset_vector = Vector2D::new(offset.x(), offset.y());
        let new_origin = self.origin() + offset_vector;
        let direction_vec = Vector2D::new(self.direction().x(), self.direction().y());

        Ray2D::new(new_origin, direction_vec).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::tolerance_migration::DefaultTolerances;

    #[test]
    fn test_ray2d_constructor() {
        // Origin とベクトルから作成（Core Traitsを使用）
        let origin = Point2::<f64>::new(1.0, 2.0);
        let direction = Vector2::<f64>::new(1.0, 0.0);
        let ray: Ray2D<f64> = Ray2DConstructor::new(origin, direction).unwrap();

        let expected_origin = Ray2DProperties::origin(&ray);
        assert_eq!(expected_origin.x(), 1.0);
        assert_eq!(expected_origin.y(), 2.0);

        // 2点から作成
        let start = Point2::<f64>::new(0.0, 0.0);
        let through = Point2::<f64>::new(1.0, 1.0);
        let ray2: Ray2D<f64> = Ray2DConstructor::from_points(start, through).unwrap();

        assert!(Ray2DProperties::is_valid(&ray2));

        // 軸方向Ray作成
        let x_ray: Ray2D<f64> = Ray2DConstructor::x_axis();
        let y_ray: Ray2D<f64> = Ray2DConstructor::y_axis();

        assert_eq!(Ray2DProperties::origin_x(&x_ray), 0.0);
        assert_eq!(Ray2DProperties::direction_x(&x_ray), 1.0);
        assert_eq!(Ray2DProperties::direction_y(&y_ray), 1.0);
    }

    #[test]
    fn test_ray2d_properties() {
        let ray = Ray2D::new(Point2D::new(1.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();

        // 起点取得
        assert_eq!(Ray2DProperties::origin_x(&ray), 1.0);
        assert_eq!(Ray2DProperties::origin_y(&ray), 2.0);

        // 方向取得
        assert_eq!(Ray2DProperties::direction_x(&ray), 1.0);
        assert_eq!(Ray2DProperties::direction_y(&ray), 0.0);

        // 有効性チェック
        assert!(Ray2DProperties::is_valid(&ray));
        assert_eq!(Ray2DProperties::dimension(&ray), 2);
    }

    #[test]
    fn test_ray2d_measure() {
        let ray = Ray2D::new(Point2D::new(1.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();

        // パラメータt での点
        let point_at_2 = Ray2DMeasure::point_at_parameter(&ray, 2.0);
        assert_eq!(point_at_2.x(), 3.0);
        assert_eq!(point_at_2.y(), 2.0);

        // 距離計算
        let test_point = Point2::new(1.0, 3.0);
        let distance = Ray2DMeasure::distance_to_point(&ray, &test_point);
        assert!((distance - 1.0).abs() < DefaultTolerances::distance::<f64>());

        // 平行性チェック
        let parallel_ray = Ray2D::new(
            Point2D::new(0.0, 0.0),
            Vector2D::new(2.0, 0.0), // 同じ方向だが倍率が違う
        )
        .unwrap();

        assert!(Ray2DMeasure::is_parallel_to(&ray, &parallel_ray));
        assert!(Ray2DMeasure::is_same_direction(&ray, &parallel_ray));

        // 逆方向チェック
        let reverse_ray = Ray2DMeasure::reverse(&ray);
        assert!(Ray2DMeasure::is_opposite_direction(&ray, &reverse_ray));

        // 平行移動
        let offset = Vector2::new(1.0, 1.0);
        let translated = Ray2DMeasure::translate(&ray, offset);
        assert_eq!(Ray2DProperties::origin_x(&translated), 2.0);
        assert_eq!(Ray2DProperties::origin_y(&translated), 3.0);
    }

    #[test]
    fn test_ray2d_core_integration() {
        // Core traitの統合テスト
        let ray: Ray2D<f64> = Ray2DConstructor::x_axis();

        // Constructorメソッド経由での作成
        let origin = Point2::new(1.0, 0.0);
        let ray2: Ray2D<f64> = Ray2DConstructor::along_positive_x(origin);

        // Measureメソッド経由での計算
        assert!(Ray2DMeasure::is_parallel_to(&ray, &ray2));
        assert!(Ray2DMeasure::is_same_direction(&ray, &ray2));
    }
}
