//! 交点計算の共通ヘルパー関数
//!
//! pair_base と primitive_3d が共有する基礎計算を提供する。

use crate::{InfiniteLine3D, LineSegment3D, Point3D, Ray3D, Vector3D};
use geo_contracts::{default_parallel_cross_error_tolerance, InfiniteLine3DProperties, Scalar};

/// 分母が有効（非ゼロ・非極小）かチェック
///
/// 外積ベースで無次元化した分母 `denom_sq`（`|d1 × d2|²` 相当）を `tolerance_sq` と比較する。
///
/// # 引数
/// - `denom_sq`: 無次元の分母（通常は外積の大きさの二乗）
/// - `tolerance_sq`: 比較閾値の二乗（通常は `default_parallel_cross_error_tolerance()²`）
///
/// # 戻り値
/// 分母が閾値より十分に大きい場合 `true`（有効）。平行またはほぼ平行の場合 `false`（無効）。
#[inline]
pub(crate) fn check_denominator_validity<T: Scalar>(denom_sq: T, tolerance_sq: T) -> bool {
    denom_sq > tolerance_sq
}

/// 2つの無限直線の交点計算（生の計算）
///
/// 平行またはほぼ平行の場合は `None` を返す。
///
/// # 戻り値の注意
/// 非共面（スキュー）な直線に対しても `Some` を返すことがある。
/// その場合の返却点は「交点」ではなく `line1` 上の最近接点候補であり、
/// 真の交点かどうかは呼び出し元で `line.distance_to_point` 等により検証すること。
///
/// coplanar/contains の最終判定は呼び出し元の責務とする。
///
/// # 注記
/// `denom_sq`（= `|d1 × d2|²`、無次元の sin²θ 相当）を
/// `default_parallel_cross_error_tolerance` の二乗と比較して平行判定を一箇所に統一する。
pub(crate) fn line_line_intersection_raw<T: Scalar>(
    line1: &InfiniteLine3D<T>,
    line2: &InfiniteLine3D<T>,
) -> Option<Point3D<T>> {
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

    // denom_sq は無次元量（sin²θ 相当）。par_tol と同じ無次元軸で比較する。
    let denom_sq = cross_d1_d2.dot(&cross_d1_d2);
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if !check_denominator_validity(denom_sq, par_tol * par_tol) {
        return None;
    }

    let t = cross_dp_d2.dot(&cross_d1_d2) / denom_sq;
    Some(Point3D::new(px1 + t * dx1, py1 + t * dy1, pz1 + t * dz1))
}

/// 2つのRayの交点計算（生の計算）
///
/// `line_line_intersection_raw` を使い外積ベースで平行判定したうえで、
/// Ray の有効範囲（t >= 0）を `contains_point` で検証する。
///
/// # 戻り値
/// 両 Ray が同一点を共有する場合 `Some(point)`。平行・スキュー・Ray 範囲外は `None`。
pub(crate) fn ray_ray_intersection_raw<T: Scalar>(
    ray1: &Ray3D<T>,
    ray2: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let line1 = InfiniteLine3D::new(ray1.origin(), ray1.direction_vector())?;
    let line2 = InfiniteLine3D::new(ray2.origin(), ray2.direction_vector())?;
    let point = line_line_intersection_raw(&line1, &line2)?;
    if ray1.contains_point(&point, tolerance) && ray2.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

/// 2つのLineSegment3Dの交点計算（生の計算）
///
/// `line_line_intersection_raw` を使い外積ベースで平行判定したうえで、
/// 両セグメントの有効範囲を `contains_point` で検証する。
/// `LineSegment3D::line()` が返す正規化済み `InfiniteLine3D` 経由で計算するため、
/// 外積ベース無次元化が Ray×Ray と同じ軸で一貫して機能する。
///
/// # 戻り値
/// 両セグメントが同一点を共有する場合 `Some(point)`。平行・スキュー・範囲外は `None`。
pub(crate) fn segment_segment_intersection_raw<T: Scalar>(
    seg1: &LineSegment3D<T>,
    seg2: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let point = line_line_intersection_raw(seg1.line(), seg2.line())?;
    if seg1.contains_point(&point, tolerance) && seg2.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STANDARD_TEST_TOLERANCE_F64: f64 = analysis::test_constants::DISTANCE_TOLERANCE_F64;

    #[test]
    fn check_denominator_validity_ゼロ分母は無効() {
        let par_tol = default_parallel_cross_error_tolerance::<f64>();
        let tolerance_sq = par_tol * par_tol;
        assert!(!check_denominator_validity(0.0_f64, tolerance_sq));
    }

    #[test]
    fn check_denominator_validity_極小分母は無効() {
        let par_tol = default_parallel_cross_error_tolerance::<f64>();
        let tolerance_sq = par_tol * par_tol;
        let denom_sq = tolerance_sq * 0.5; // 閾値の半分
        assert!(!check_denominator_validity(denom_sq, tolerance_sq));
    }

    #[test]
    fn check_denominator_validity_閾値と同値は無効() {
        // check_denominator_validity は denom_sq > tolerance_sq の厳密不等号なので
        // 閾値と等しい場合は無効（None 側）
        let par_tol = default_parallel_cross_error_tolerance::<f64>();
        let tolerance_sq = par_tol * par_tol;
        assert!(!check_denominator_validity(tolerance_sq, tolerance_sq));
    }

    #[test]
    fn check_denominator_validity_十分な分母は有効() {
        let par_tol = default_parallel_cross_error_tolerance::<f64>();
        let tolerance_sq = par_tol * par_tol;
        let denom_sq = tolerance_sq * 2.0;
        assert!(check_denominator_validity(denom_sq, tolerance_sq));
    }

    #[test]
    fn ray_ray_intersection_raw_交差するrayは交点を返す() {
        use crate::Vector3D;
        let ray1 = Ray3D::new(
            Point3D::new(-1.0_f64, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let ray2 = Ray3D::new(
            Point3D::new(0.0, -1.0_f64, 0.0),
            Vector3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let result = ray_ray_intersection_raw(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.is_some());
        let p = result.unwrap();
        assert!(p.x().abs() < STANDARD_TEST_TOLERANCE_F64);
        assert!(p.y().abs() < STANDARD_TEST_TOLERANCE_F64);
    }

    #[test]
    fn ray_ray_intersection_raw_平行rayはnoneを返す() {
        use crate::Vector3D;
        let ray1 = Ray3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let ray2 = Ray3D::new(
            Point3D::new(0.0_f64, 1.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let result = ray_ray_intersection_raw(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.is_none());
    }

    #[test]
    fn ray_ray_intersection_raw_ray範囲外はnoneを返す() {
        use crate::Vector3D;
        // 互いに反対方向を向いており、延長線上では交わるが Ray 範囲外
        let ray1 = Ray3D::new(
            Point3D::new(1.0_f64, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let ray2 = Ray3D::new(
            Point3D::new(0.0, 1.0_f64, 0.0),
            Vector3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let result = ray_ray_intersection_raw(&ray1, &ray2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.is_none());
    }

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
        assert!(p.x().abs() < STANDARD_TEST_TOLERANCE_F64);
        assert!(p.y().abs() < STANDARD_TEST_TOLERANCE_F64);
        assert!(p.z().abs() < STANDARD_TEST_TOLERANCE_F64);
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
        // 平行判定閾値の半分の角度を使用（閾値変更に対して頑健）
        // d1 × d2 の長さ ≈ sin(θ) ≈ θ（小角度近似）なので、
        // par_tol / 2 の角度では |d1 × d2| ≈ par_tol / 2 < par_tol となり None が期待される
        let tiny_angle = default_parallel_cross_error_tolerance::<f64>() / 2.0;
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
        assert!(result.is_none());
    }

    #[test]
    fn line_line_intersection_raw_スキュー線でも最近接点候補を返す() {
        // スキュー線（非共面）：真の交点は存在しないが、この関数は
        // line1 上の最近接点候補を返す。coplanar チェックは呼び出し元の責務。
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

        let result = line_line_intersection_raw(&line1, &line2);
        assert!(result.is_some());
    }

    #[test]
    fn segment_segment_intersection_raw_交差するセグメントは交点を返す() {
        use crate::LineSegment3D;
        let seg1 = LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0))
            .unwrap();
        let seg2 = LineSegment3D::new(Point3D::new(0.0_f64, 1.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();

        let result = segment_segment_intersection_raw(&seg1, &seg2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.is_some());
        let p = result.unwrap();
        assert!((p.x() - 0.5).abs() < STANDARD_TEST_TOLERANCE_F64);
        assert!((p.y() - 0.5).abs() < STANDARD_TEST_TOLERANCE_F64);
    }

    #[test]
    fn segment_segment_intersection_raw_平行セグメントはnoneを返す() {
        use crate::LineSegment3D;
        let seg1 = LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();
        let seg2 = LineSegment3D::new(Point3D::new(0.0_f64, 1.0, 0.0), Point3D::new(1.0, 1.0, 0.0))
            .unwrap();

        let result = segment_segment_intersection_raw(&seg1, &seg2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.is_none());
    }

    #[test]
    fn segment_segment_intersection_raw_セグメント範囲外はnoneを返す() {
        // 延長線上では交わるが、どちらかのセグメント範囲外
        use crate::LineSegment3D;
        let seg1 = LineSegment3D::new(Point3D::new(2.0_f64, 0.0, 0.0), Point3D::new(3.0, 1.0, 0.0))
            .unwrap();
        let seg2 = LineSegment3D::new(Point3D::new(0.0_f64, 1.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .unwrap();

        let result = segment_segment_intersection_raw(&seg1, &seg2, STANDARD_TEST_TOLERANCE_F64);
        assert!(result.is_none());
    }
}
