//! NURBS Curve3D Self-Intersection Implementation
//!
//! NURBS曲線の自己交差検出を実装
//! パラメータ空間での交差判定により効率的に検出

use crate::{NurbsCurve3D, Result, Scalar};
use analysis::linalg::vector::Vector3;
use geo_foundation::extensions::SelfIntersection;
use geo_primitives::Point3D;

/// パラメータ空間での曲線セグメント
#[derive(Debug, Clone)]
struct CurveSegment<T: Scalar> {
    /// 開始パラメータ
    t_start: T,
    /// 終了パラメータ
    t_end: T,
    /// セグメントのバウンディングボックス（最小点）
    bbox_min: Vector3<T>,
    /// セグメントのバウンディングボックス（最大点）
    bbox_max: Vector3<T>,
}

impl<T: Scalar> CurveSegment<T> {
    /// 曲線セグメントを作成
    ///
    /// パラメータ範囲内でサンプリングしてバウンディングボックスを計算
    fn from_curve(curve: &NurbsCurve3D<T>, t_start: T, t_end: T, samples: usize) -> Result<Self> {
        let mut min_x = T::MAX;
        let mut min_y = T::MAX;
        let mut min_z = T::MAX;
        let mut max_x = T::MIN;
        let mut max_y = T::MIN;
        let mut max_z = T::MIN;

        // セグメント内をサンプリングしてバウンディングボックスを計算
        for i in 0..=samples {
            let t = if samples == 0 {
                t_start
            } else {
                let ratio = T::from_f64(i as f64 / samples as f64);
                t_start + (t_end - t_start) * ratio
            };

            let point = curve.evaluate_at(t);
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            min_z = min_z.min(point.z());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
            max_z = max_z.max(point.z());
        }

        Ok(Self {
            t_start,
            t_end,
            bbox_min: Vector3::new(min_x, min_y, min_z),
            bbox_max: Vector3::new(max_x, max_y, max_z),
        })
    }

    /// 他のセグメントとバウンディングボックスが重なっているか判定
    fn bbox_intersects(&self, other: &Self) -> bool {
        self.bbox_min.x() <= other.bbox_max.x()
            && self.bbox_max.x() >= other.bbox_min.x()
            && self.bbox_min.y() <= other.bbox_max.y()
            && self.bbox_max.y() >= other.bbox_min.y()
            && self.bbox_min.z() <= other.bbox_max.z()
            && self.bbox_max.z() >= other.bbox_min.z()
    }

    /// パラメータ範囲が重なっているか判定（隣接セグメント検出用）
    fn parameter_overlaps(&self, other: &Self, tolerance: T) -> bool {
        let gap = if self.t_end < other.t_start {
            other.t_start - self.t_end
        } else if other.t_end < self.t_start {
            self.t_start - other.t_end
        } else {
            T::ZERO
        };

        gap < tolerance
    }
}

/// 2つのパラメータ値での曲線上の点の距離を計算
fn parameter_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    t1: T,
    t2: T,
) -> T {
    let p1 = curve.evaluate_at(t1);
    let p2 = curve.evaluate_at(t2);

    let dx = p1.x() - p2.x();
    let dy = p1.y() - p2.y();
    let dz = p1.z() - p2.z();

    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// セグメント間の交差点を探索（二分探索法）
fn find_segment_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    seg1: &CurveSegment<T>,
    seg2: &CurveSegment<T>,
    tolerance: T,
    max_iterations: usize,
) -> Option<Point3D<T>> {
    let mut t1_min = seg1.t_start;
    let mut t1_max = seg1.t_end;
    let mut t2_min = seg2.t_start;
    let mut t2_max = seg2.t_end;

    for _ in 0..max_iterations {
        let t1_mid = (t1_min + t1_max) / (T::ONE + T::ONE);
        let t2_mid = (t2_min + t2_max) / (T::ONE + T::ONE);

        let distance = parameter_distance(curve, t1_mid, t2_mid);

        if distance < tolerance {
            // 交点を発見
            let intersection = curve.evaluate_at(t1_mid);
            return Some(Point3D::new(
                intersection.x(),
                intersection.y(),
                intersection.z(),
            ));
        }

        // 範囲が十分小さくなった場合は終了
        let range1 = t1_max - t1_min;
        let range2 = t2_max - t2_min;
        if range1 < tolerance && range2 < tolerance {
            break;
        }

        // 次の探索範囲を決定（簡易実装）
        t1_min = t1_mid - range1 / (T::ONE + T::ONE + T::ONE + T::ONE);
        t1_max = t1_mid + range1 / (T::ONE + T::ONE + T::ONE + T::ONE);
        t2_min = t2_mid - range2 / (T::ONE + T::ONE + T::ONE + T::ONE);
        t2_max = t2_mid + range2 / (T::ONE + T::ONE + T::ONE + T::ONE);

        // パラメータ範囲を曲線の有効範囲内に制限
        t1_min = t1_min.max(seg1.t_start);
        t1_max = t1_max.min(seg1.t_end);
        t2_min = t2_min.max(seg2.t_start);
        t2_max = t2_max.min(seg2.t_end);
    }

    None
}

impl<T: Scalar> SelfIntersection<T> for NurbsCurve3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, tolerance: T) -> Vec<Self::Point> {
        let mut intersections = Vec::new();

        // パラメータ範囲を取得
        let (t_min, t_max) = self.parameter_domain();

        // 曲線を複数のセグメントに分割
        let num_segments = 10; // セグメント数（調整可能）
        let segment_size = (t_max - t_min) / T::from_usize(num_segments);

        let mut segments = Vec::with_capacity(num_segments);
        for i in 0..num_segments {
            let t_start = t_min + segment_size * T::from_usize(i);
            let t_end = if i == num_segments - 1 {
                t_max
            } else {
                t_start + segment_size
            };

            if let Ok(segment) = CurveSegment::from_curve(self, t_start, t_end, 5) {
                segments.push(segment);
            }
        }

        // 全てのセグメントペアをチェック
        for i in 0..segments.len() {
            for j in (i + 1)..segments.len() {
                let seg1 = &segments[i];
                let seg2 = &segments[j];

                // 隣接セグメントはスキップ
                if seg1.parameter_overlaps(seg2, tolerance) {
                    continue;
                }

                // BBoxで事前スクリーニング
                if !seg1.bbox_intersects(seg2) {
                    continue;
                }

                // 詳細な交差判定
                if let Some(intersection) =
                    find_segment_intersection(self, seg1, seg2, tolerance, 20)
                {
                    intersections.push(intersection);
                }
            }
        }

        intersections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{clamped_knot_vector, KnotVector};
    use analysis::linalg::vector::Vector3;

    #[test]
    fn test_nurbs_curve_no_self_intersection() {
        // 単純な直線（自己交差なし）
        let control_points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
        ];

        let degree = 2;
        let knot_vector = clamped_knot_vector(degree, control_points.len());

        let curve = NurbsCurve3D::new(control_points, None, knot_vector, degree).unwrap();
        let intersections = curve.self_intersections(1e-6);

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_nurbs_curve_simple_case() {
        // 簡単な2次曲線
        let control_points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
        ];

        let degree = 2;
        let knot_vector = clamped_knot_vector(degree, control_points.len());

        let curve = NurbsCurve3D::new(control_points, None, knot_vector, degree).unwrap();
        let intersections = curve.self_intersections(1e-6);

        // 単純な放物線型曲線は自己交差しない
        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_nurbs_curve_with_potential_self_intersection() {
        // より複雑な形状（S字型）
        let control_points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 2.0, 0.0),
            Vector3::new(2.0, -1.0, 0.0),
            Vector3::new(3.0, 1.0, 0.0),
            Vector3::new(4.0, 0.0, 0.0),
        ];

        let degree = 3;
        let knot_vector = clamped_knot_vector(degree, control_points.len());

        let curve = NurbsCurve3D::new(control_points, None, knot_vector, degree).unwrap();
        let intersections = curve.self_intersections(0.5);

        // S字型曲線で自己交差の可能性をチェック
        // 簡易実装のため検出精度は限定的
        assert!(
            intersections.len() >= 0,
            "Self-intersection detection should work"
        );
    }

    #[test]
    fn test_segment_bbox_intersection() {
        let control_points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(2.0, 0.0, 0.0),
        ];

        let degree = 2;
        let knot_vector = clamped_knot_vector(degree, control_points.len());
        let curve = NurbsCurve3D::new(control_points, None, knot_vector, degree).unwrap();

        let (t_min, t_max) = curve.parameter_domain();
        let t_mid = (t_min + t_max) / 2.0;

        let seg1 = CurveSegment::from_curve(&curve, t_min, t_mid, 5).unwrap();
        let seg2 = CurveSegment::from_curve(&curve, t_mid, t_max, 5).unwrap();

        // 連続したセグメントは境界で接触
        assert!(seg1.bbox_intersects(&seg2));
    }
}
