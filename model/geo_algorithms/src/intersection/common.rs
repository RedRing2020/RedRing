//! 交点計算の共通ヘルパー関数
//!
//! pair_base と primitive_3d が共有する基礎計算を提供する。

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_contracts::{default_distance_tolerance, InfiniteLine3DProperties, Scalar};

/// 2つの無限直線の交点計算（生の計算）
///
/// 平行またはほぼ平行の場合は None を返す。
/// coplanar/contains 判定は呼び出し元で行う想定。
///
/// # 注記
/// ほぼ平行なケースで分母が極小になることを防ぐため、
/// 分母の大きさに対して既定の距離トレランスを基にした閾値チェックを実施する。
pub(crate) fn line_line_intersection_raw<T: Scalar>(
    line1: &InfiniteLine3D<T>,
    line2: &InfiniteLine3D<T>,
) -> Option<Point3D<T>> {
    if line1.is_parallel_to(line2) {
        return None;
    }
    let (px1, py1, pz1) = InfiniteLine3DProperties::point(line1);
    let (dx1, dy1, dz1) = InfiniteLine3DProperties::direction(line1);
    let (px2, py2, pz2) = InfiniteLine3DProperties::point(line2);
    let (dx2, dy2, dz2) = InfiniteLine3DProperties::direction(line2);
    let p1 = Point3D::new(px1, py1, pz1);
    let d1 = Vector3D::new(dx1, dy1, dz1);
    let p2 = Point3D::new(px2, py2, pz2);
    let d2 = Vector3D::new(dx2, dy2, dz2);
    let dp = Vector3D::from_points(&p1, &p2);
    let cross_d1_d2 = d1.cross(&d2);
    let cross_dp_d2 = dp.cross(&d2);

    // 分母が極小になることを防ぐ
    let denom_sq = cross_d1_d2.dot(&cross_d1_d2);
    let tolerance_sq = default_distance_tolerance::<T>() * default_distance_tolerance::<T>();
    if denom_sq <= tolerance_sq {
        return None;
    }

    let t = cross_dp_d2.dot(&cross_d1_d2) / denom_sq;
    Some(Point3D::new(px1 + t * dx1, py1 + t * dy1, pz1 + t * dz1))
}
