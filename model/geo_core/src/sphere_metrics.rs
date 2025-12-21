//! 球形状の距離計算関数
//!
//! Point3D/Vector3D型を使用した型安全な球距離計算を提供します。

use crate::{Point3D, Vector3D};
use analysis::Scalar;

/// 球から無限直線までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標
/// * `radius` - 球の半径
/// * `line_point` - 直線上の任意の点
/// * `line_direction` - 直線の方向ベクトル（正規化不要）
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から無限直線までの最短距離
pub fn sphere_to_infinite_line_distance<T: Scalar>(
    center: &Point3D<T>,
    radius: T,
    line_point: &Point3D<T>,
    line_direction: &Vector3D<T>,
    is_solid: bool,
) -> T {
    // ベクトル to_center = center - line_point
    let to_center = *center - *line_point;

    // direction の内積
    let dir_dot = line_direction.dot(line_direction);

    // パラメータ t = to_center · direction / |direction|²
    let t = to_center.dot(line_direction) / dir_dot;

    // 最近点 = line_point + t * direction
    let closest = *line_point + *line_direction * t;

    // 中心から最近点までの距離
    let distance_from_center = (*center - closest).length();

    if is_solid {
        // 球体: 内部なら0、外側なら表面までの距離
        if distance_from_center <= radius {
            T::ZERO
        } else {
            distance_from_center - radius
        }
    } else {
        // 球面: 表面までの最短距離
        (distance_from_center - radius).abs()
    }
}

/// 球から光線（Ray）までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標
/// * `radius` - 球の半径
/// * `ray_origin` - 光線の始点
/// * `ray_direction` - 光線の方向ベクトル（正規化不要）
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から光線までの最短距離
pub fn sphere_to_ray_distance<T: Scalar>(
    center: &Point3D<T>,
    radius: T,
    ray_origin: &Point3D<T>,
    ray_direction: &Vector3D<T>,
    is_solid: bool,
) -> T {
    // ベクトル to_center = center - origin
    let to_center = *center - *ray_origin;

    // direction の内積
    let dir_dot = ray_direction.dot(ray_direction);

    // パラメータ t
    let t = to_center.dot(ray_direction) / dir_dot;

    if t < T::ZERO {
        // 光線の始点が最近点
        let dist_to_origin = to_center.length();

        if is_solid {
            if dist_to_origin <= radius {
                T::ZERO
            } else {
                dist_to_origin - radius
            }
        } else {
            (dist_to_origin - radius).abs()
        }
    } else {
        // t ≥ 0: 無限直線と同じ処理
        let closest = *ray_origin + *ray_direction * t;

        let distance_from_center = (*center - closest).length();

        if is_solid {
            if distance_from_center <= radius {
                T::ZERO
            } else {
                distance_from_center - radius
            }
        } else {
            (distance_from_center - radius).abs()
        }
    }
}

/// 球から線分までの最短距離を計算
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
    // 線分の方向ベクトル
    let direction = *segment_end - *segment_start;

    // ベクトル to_center = center - start
    let to_center = *center - *segment_start;

    // direction の内積
    let dir_dot = direction.dot(&direction);

    // パラメータ t
    let t = to_center.dot(&direction) / dir_dot;

    // 最近点の座標を求める
    let nearest = if t < T::ZERO {
        // 始点が最近点
        *segment_start
    } else if t > T::ONE {
        // 終点が最近点
        *segment_end
    } else {
        // 線分上の点が最近点
        *segment_start + direction * t
    };

    // 中心から最近点までの距離
    let distance_from_center = (*center - nearest).length();

    if is_solid {
        if distance_from_center <= radius {
            T::ZERO
        } else {
            distance_from_center - radius
        }
    } else {
        (distance_from_center - radius).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_to_infinite_line_tangent() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let line_point = Point3D::new(0.0, 1.0, 0.0);
        let line_direction = Vector3D::new(1.0, 0.0, 0.0);

        let dist =
            sphere_to_infinite_line_distance(&center, radius, &line_point, &line_direction, true);
        assert!(dist.abs() < 1e-10);
    }

    #[test]
    fn test_sphere_to_ray_behind_origin() {
        let center = Point3D::new(-2.0, 0.0, 0.0);
        let radius = 1.0;
        let ray_origin = Point3D::new(0.0, 0.0, 0.0);
        let ray_direction = Vector3D::new(1.0, 0.0, 0.0);

        let dist = sphere_to_ray_distance(&center, radius, &ray_origin, &ray_direction, true);
        assert!((dist - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_to_line_segment_through_center() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let segment_start = Point3D::new(-2.0, 0.0, 0.0);
        let segment_end = Point3D::new(2.0, 0.0, 0.0);

        let dist =
            sphere_to_line_segment_distance(&center, radius, &segment_start, &segment_end, true);
        assert!(dist.abs() < 1e-10);
    }
}
