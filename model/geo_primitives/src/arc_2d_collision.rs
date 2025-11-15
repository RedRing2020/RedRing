//! Arc衝突検出・距離計算統一Foundation実装
//!
//! 統一Collision Foundation システムによる衝突検出・距離計算
//! 全幾何プリミティブで共通利用可能な統一インターフェース

// use crate::{Arc2D, Circle2D, Point2D, Vector2D}; // Arc2D一時的にコメントアウト
use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::{
    extensions::{BasicCollision, PointDistance},
    Angle, Scalar,
};

// ============================================================================
// PointDistance Trait Implementation (統一Foundation)
// ============================================================================

// impl<T: Scalar> PointDistance<T> for Arc2D<T> {
//     type Point2D = Point2D<T>;

//     /// 点までの距離
//     fn distance_to_point(&self, point: &Self::Point2D) -> T {
//         // 1. 点から円の中心への距離
//         let center = self.circle().center();
//         let center_to_point = Vector2D::new(point.x() - center.x(), point.y() - center.y());
//         let distance_to_center = center_to_point.magnitude();
//
//         // 2. 点が円弧の角度範囲内にあるかを確認
//         let angle_to_point = center_to_point.angle();
//
//         if self.angle_in_range(angle_to_point) {
//             // 点が円弧の角度範囲内にある場合
//             // 円の半径との差が最短距離
//             (distance_to_center - self.circle().radius()).abs()
//         } else {
//             // 点が円弧の角度範囲外にある場合
//             // 開始点または終了点までの距離
//             let start_point = self.start_point();
//             let end_point = self.end_point();
//
//             let dist_to_start = point.distance_to(&start_point);
//             let dist_to_end = point.distance_to(&end_point);
//
//             dist_to_start.min(dist_to_end)
//         }
//     }
//
//     /// 点が内部にあるか（円弧の場合は扇形領域）
//     fn contains_point(&self, point: &Self::Point2D, tolerance: T) -> bool {
//         // 円弧の扇形領域内かを判定
//         let center = self.circle().center();
//         let center_to_point = Vector2D::new(point.x() - center.x(), point.y() - center.y());
//         let distance_to_center = center_to_point.magnitude();
//         let angle_to_point = center_to_point.angle();
//
//         // 1. 円の内部にあるか
//         let within_circle = distance_to_center <= self.circle().radius() + tolerance;
//
//         // 2. 角度範囲内にあるか
//         let within_angle_range = self.angle_in_range_with_tolerance(angle_to_point, tolerance);
//
//         within_circle && within_angle_range
//     }
//
//     /// 点が境界上にあるか（円弧上）
//     fn point_on_boundary(&self, point: &Self::Point2D, tolerance: T) -> bool {
//         let center = self.circle().center();
//         let center_to_point = Vector2D::new(point.x() - center.x(), point.y() - center.y());
//         let distance_to_center = center_to_point.magnitude();
//         let angle_to_point = center_to_point.angle();
//
//         // 1. 円の境界上にあるか
//         let on_circle = (distance_to_center - self.circle().radius()).abs() <= tolerance;
//
//         // 2. 角度範囲内にあるか
//         let within_angle_range = self.angle_in_range_with_tolerance(angle_to_point, tolerance);
//
//         on_circle && within_angle_range
//     }
//
//     /// 最近点を取得
//     fn closest_point(&self, point: &Self::Point2D) -> Self::Point2D {
//         let center = self.circle().center();
//         let center_to_point = Vector2D::new(point.x() - center.x(), point.y() - center.y());
//         let distance_to_center = center_to_point.magnitude();
//
//         if distance_to_center == T::ZERO {
//             // 点が中心にある場合は開始点を返す
//             return self.start_point();
//         }
//
//         let angle_to_point = center_to_point.angle();
//
//         if self.angle_in_range(angle_to_point) {
//             // 点が円弧の角度範囲内にある場合
//             // 円周上の対応点
//             let radius = self.circle().radius();
//             let unit_vector = center_to_point.normalize();
//             Point2D::new(
//                 center.x() + unit_vector.x() * radius,
//                 center.y() + unit_vector.y() * radius,
//             )
//         } else {
//             // 点が円弧の角度範囲外にある場合
//             // 開始点または終了点の近い方
//             let start_point = self.start_point();
//             let end_point = self.end_point();
//
//             let dist_to_start = point.distance_to(&start_point);
//             let dist_to_end = point.distance_to(&end_point);
//
//             if dist_to_start < dist_to_end {
//                 start_point
//             } else {
//                 end_point
//             }
//         }
//     }
// }

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// Arc vs Point
impl<T: Scalar> BasicCollision<T, Point2D<T>> for Arc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Point2D<T>, tolerance: T) -> bool {
        self.point_on_boundary(other, tolerance)
    }

    fn overlaps(&self, other: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(other, tolerance) || self.point_on_boundary(other, tolerance)
    }

    fn distance_to(&self, other: &Point2D<T>) -> T {
        self.distance_to_point(other)
    }
}

// Arc vs Circle
impl<T: Scalar> BasicCollision<T, Circle2D<T>> for Arc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Circle2D<T>, tolerance: T) -> bool {
        // 円と円弧の中心距離計算
        let arc_center = self.circle().center();
        let circle_center = other.center();
        let center_distance = arc_center.distance_to(&circle_center);

        let arc_radius = self.circle().radius();
        let circle_radius = other.radius();
        let radii_sum = arc_radius + circle_radius;
        let radii_diff = (arc_radius - circle_radius).abs();

        // 円同士が交差する条件
        let circles_intersect =
            center_distance <= radii_sum + tolerance && center_distance >= radii_diff - tolerance;

        if !circles_intersect {
            return false;
        }

        // TODO: 具体的な交点が円弧の角度範囲内にあるかの詳細判定
        // 現在は簡易実装
        true
    }

    fn overlaps(&self, other: &Circle2D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &Circle2D<T>) -> T {
        // 中心間距離から半径を差し引いた値
        let center_distance = self.circle().center().distance_to(&other.center());
        let min_distance = center_distance - self.circle().radius() - other.radius();

        min_distance.max(T::ZERO)
    }
}

// Arc vs Line (TODO: Line2D実装後に追加)
// impl<T: Scalar> BasicCollision<T, Line2D<T>> for Arc2D<T> { ... }

// Arc vs Arc
impl<T: Scalar> BasicCollision<T, Arc2D<T>> for Arc2D<T> {
    type Point2D = Point2D<T>;

    fn intersects(&self, other: &Arc2D<T>, tolerance: T) -> bool {
        // まず基底円同士の交差判定
        let circles_intersect =
            <Self as BasicCollision<T, Circle2D<T>>>::intersects(self, other.circle(), tolerance);

        if !circles_intersect {
            return false;
        }

        // TODO: 具体的な交点が両方の円弧の角度範囲内にあるかの詳細判定
        // 現在は簡易実装
        true
    }

    fn overlaps(&self, other: &Arc2D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &Arc2D<T>) -> T {
        // 簡易実装: 基底円同士の距離
        <Self as BasicCollision<T, Circle2D<T>>>::distance_to(self, other.circle())
    }
}

// ============================================================================
// Arc2D 固有ヘルパーメソッド
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// 角度が円弧の範囲内にあるかを判定
    pub fn angle_in_range(&self, angle: Angle<T>) -> bool {
        // TODO: Angle<T>での角度正規化とrange判定
        // 現在は簡易実装
        let start_rad = self.start_angle().to_radians();
        let end_rad = self.end_angle().to_radians();
        let angle_rad = angle.to_radians();

        if start_rad <= end_rad {
            angle_rad >= start_rad && angle_rad <= end_rad
        } else {
            // 角度が0を跨ぐ場合
            angle_rad >= start_rad || angle_rad <= end_rad
        }
    }

    /// tolerance付きで角度が円弧の範囲内にあるかを判定
    pub fn angle_in_range_with_tolerance(&self, angle: Angle<T>, _tolerance: T) -> bool {
        // TODO: より正確なtolerance判定実装
        self.angle_in_range(angle)
    }
}
