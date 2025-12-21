/// 球形状と線分形状間の距離計算ヘルパー関数
///
/// このモジュールは geo_commons の距離計算関数を型安全にラップします。
/// Point3D, Vector3D などの型を使用して API の可読性と型安全性を提供します。
///
/// # アーキテクチャ
///
/// ```text
/// geo_commons (タプルベース) → geo_primitives (型安全ラッパー)
/// ```
///
/// # 作成日
/// 2025年12月21日

use crate::{Direction3D, Point3D};
use analysis::Scalar;
use geo_commons::metrics::sphere_to_infinite_line_distance as commons_infinite_line;
use geo_commons::metrics::sphere_to_line_segment_distance as commons_line_segment;
use geo_commons::metrics::sphere_to_ray_distance as commons_ray;

/// 球から無限直線までの最短距離を計算（型安全版）
///
/// # Arguments
///
/// * `center` - 球の中心座標
/// * `radius` - 球の半径
/// * `line_point` - 直線上の任意の点
/// * `line_direction` - 直線の方向（正規化済み）
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から無限直線までの最短距離
///
/// # Example
///
/// ```ignore
/// use geo_primitives::{Point3D, Direction3D};
///
/// let center = Point3D::new(0.0, 0.0, 0.0);
/// let radius = 1.0;
/// let point = Point3D::new(-2.0, 0.0, 0.0);
/// let direction = Direction3D::x_axis();
///
/// let distance = sphere_to_infinite_line_distance(
///     &center, radius, &point, &direction, true
/// );
/// ```
pub fn sphere_to_infinite_line_distance<T: Scalar>(
    center: &Point3D<T>,
    radius: T,
    line_point: &Point3D<T>,
    line_direction: &Direction3D<T>,
    is_solid: bool,
) -> T {
    commons_infinite_line(
        (center.x(), center.y(), center.z()),
        radius,
        (line_point.x(), line_point.y(), line_point.z()),
        (line_direction.x(), line_direction.y(), line_direction.z()),
        is_solid,
    )
}

/// 球から光線（Ray）までの最短距離を計算（型安全版）
///
/// # Arguments
///
/// * `center` - 球の中心座標
/// * `radius` - 球の半径
/// * `ray_origin` - 光線の始点
/// * `ray_direction` - 光線の方向（正規化済み）
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から光線までの最短距離
pub fn sphere_to_ray_distance<T: Scalar>(
    center: &Point3D<T>,
    radius: T,
    ray_origin: &Point3D<T>,
    ray_direction: &Direction3D<T>,
    is_solid: bool,
) -> T {
    commons_ray(
        (center.x(), center.y(), center.z()),
        radius,
        (ray_origin.x(), ray_origin.y(), ray_origin.z()),
        (ray_direction.x(), ray_direction.y(), ray_direction.z()),
        is_solid,
    )
}

/// 球から線分までの最短距離を計算（型安全版）
///
/// # Arguments
///
/// * `center` - 球の中心座標
/// * `radius` - 球の半径
/// * `segment_start` - 線分の始点
/// * `segment_end` - 線分の終点
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から線分までの最短距離
pub fn sphere_to_line_segment_distance<T: Scalar>(
    center: &Point3D<T>,
    radius: T,
    segment_start: &Point3D<T>,
    segment_end: &Point3D<T>,
    is_solid: bool,
) -> T {
    commons_line_segment(
        (center.x(), center.y(), center.z()),
        radius,
        (segment_start.x(), segment_start.y(), segment_start.z()),
        (segment_end.x(), segment_end.y(), segment_end.z()),
        is_solid,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_to_infinite_line_type_safe() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let line_point = Point3D::new(-2.0, 0.0, 0.0);
        let line_direction = Direction3D::positive_x();

        let dist = sphere_to_infinite_line_distance(&center, radius, &line_point, &line_direction, true);
        assert!(dist.abs() < 1e-10);
    }

    #[test]
    fn test_sphere_to_ray_type_safe() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let ray_origin = Point3D::new(-2.0, 0.0, 0.0);
        let ray_direction = Direction3D::positive_x();

        let dist = sphere_to_ray_distance(&center, radius, &ray_origin, &ray_direction, true);
        assert!(dist.abs() < 1e-10);
    }

    #[test]
    fn test_sphere_to_line_segment_type_safe() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let segment_start = Point3D::new(-2.0, 0.0, 0.0);
        let segment_end = Point3D::new(2.0, 0.0, 0.0);

        let dist = sphere_to_line_segment_distance(&center, radius, &segment_start, &segment_end, true);
        assert!(dist.abs() < 1e-10);
    }
}
