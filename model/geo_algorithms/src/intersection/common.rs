//! 交点計算の共通ヘルパー関数
//!
//! pair_base と primitive_3d が共有する基礎計算を提供する。

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_contracts::{default_parallel_cross_error_tolerance, InfiniteLine3DProperties, Scalar};

/// 2つの無限直線の交点計算（生の計算）
///
/// 平行またはほぼ平行の場合は None を返す。
/// coplanar/contains 判定は呼び出し元で行う想定。
///
/// # 注記
/// ほぼ平行なケースで分母が極小になることを防ぐため、
/// 無次元の平行判定閾値（`default_parallel_cross_error_tolerance`）を用いた
/// 分母チェックを実施する。
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

    // denom_sq は無次元量（sin^2(θ)相当）。無次元の平行判定閾値と比較する。
    let denom_sq = cross_d1_d2.dot(&cross_d1_d2);
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if denom_sq <= par_tol * par_tol {
        return None;
    }

    let t = cross_dp_d2.dot(&cross_d1_d2) / denom_sq;
    Some(Point3D::new(px1 + t * dx1, py1 + t * dy1, pz1 + t * dz1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn line_line_intersection_raw_交差する直線は交点を返す() {
        // X軸と Y軸が原点で交差
        let line1 = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0_f64, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line2 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, -1.0_f64, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let result = line_line_intersection_raw(&line1, &line2);
        assert!(result.is_some());
        let p = result.unwrap();
        assert!(p.x().abs() < 1e-10);
        assert!(p.y().abs() < 1e-10);
        assert!(p.z().abs() < 1e-10);
    }

    #[test]
    fn line_line_intersection_raw_平行直線はnoneを返す() {
        // 平行な2直線（Y方向にオフセット）
        let line1 = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0_f64, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line2 = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0_f64, 1.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        )
        .unwrap();

        let result = line_line_intersection_raw(&line1, &line2);
        assert!(result.is_none());
    }

    #[test]
    fn line_line_intersection_raw_ほぼ平行な直線はnoneを返す() {
        // 極めて小さな角度（ほぼ平行）
        let tiny_angle: f64 = 1e-10;
        let line1 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line2 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(1.0, tiny_angle, 0.0),
        )
        .unwrap();

        let result = line_line_intersection_raw(&line1, &line2);
        // ほぼ平行の場合は分母チェックまたは is_parallel_to で None
        assert!(result.is_none());
    }

    #[test]
    fn line_line_intersection_raw_スキュー線は交点を計算する() {
        // スキュー線（非共面）：交点の有無は呼び出し元が検証する
        // この関数は共面チェックを行わず最近接点を計算して返す
        let line1 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line2 = InfiniteLine3D::from_two_points(
            Point3D::new(0.0_f64, 1.0, 1.0),
            Point3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        // スキュー線でも None ではなく Some を返す（coplanar チェックは呼び出し元の責務）
        let result = line_line_intersection_raw(&line1, &line2);
        assert!(result.is_some());
    }
}
