//! Arc包含・角度判定拡張トレイト実装
//!
//! 点の包含判定や角度範囲チェック機能
//! 他の幾何プリミティブでも共通利用可能な抽象化

use crate::{Arc2D, Point2D};
use geo_foundation::{
    abstracts::arc_traits::ArcContainment, tolerance_migration::DefaultTolerances, Angle, Scalar,
};

// ============================================================================
// ArcContainment Trait Implementation
// ============================================================================

impl<T: Scalar> ArcContainment<T> for Arc2D<T> {
    /// 点が円弧上にあるかを判定
    fn contains_point(&self, point: &Point2D<T>) -> bool {
        // まず基底円上にあるかチェック
        let distance_to_center = self.center().distance_to(point);
        let radius_diff = (distance_to_center - self.radius()).abs();

        if radius_diff > DefaultTolerances::distance::<T>() {
            return false;
        }

        // 角度範囲内にあるかチェック
        let point_angle = self.point_to_angle(point);
        self.contains_angle(point_angle)
    }

    /// 角度が円弧の角度範囲内にあるかを判定
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        let normalized_angle = self.normalize_angle(angle);
        let normalized_start = self.normalize_angle(self.start_angle());
        let normalized_end = self.normalize_angle(self.end_angle());

        if normalized_start <= normalized_end {
            // 通常の範囲（例：30°から90°）
            normalized_angle >= normalized_start && normalized_angle <= normalized_end
        } else {
            // 0°をまたぐ範囲（例：300°から60°）
            normalized_angle >= normalized_start || normalized_angle <= normalized_end
        }
    }

    /// 指定角度での円弧上の点を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Point2D<T> {
        self.point_at_angle(angle.to_radians())
    }
}

// ============================================================================
// Arc2D用の包含判定ヘルパーメソッド
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    /// 点から角度を計算
    pub fn point_to_angle(&self, point: &Point2D<T>) -> Angle<T> {
        let center = self.center();
        let dx = point.x() - center.x();
        let dy = point.y() - center.y();
        Angle::from_radians(dy.atan2(dx))
    }

    /// 角度を [0, 2π) 範囲に正規化
    pub fn normalize_angle(&self, angle: Angle<T>) -> Angle<T> {
        let two_pi = Angle::from_radians(T::TAU);
        let mut normalized = angle;

        // 負の角度を正に変換
        while normalized.to_radians() < T::ZERO {
            normalized += two_pi;
        }

        // 2π以上の角度を削減
        while normalized >= two_pi {
            normalized -= two_pi;
        }

        normalized
    }

    /// 点が円弧に最も近い点を取得
    pub fn closest_point_on_arc(&self, point: &Point2D<T>) -> Point2D<T> {
        let point_angle = self.point_to_angle(point);

        if self.contains_angle(point_angle) {
            // 点の角度が円弧範囲内の場合、基底円上の最近点
            self.point_at_angle(point_angle.to_radians())
        } else {
            // 範囲外の場合、開始点または終了点のうち近い方
            let start_point = self.start_point();
            let end_point = self.end_point();
            let dist_to_start = point.distance_to(&start_point);
            let dist_to_end = point.distance_to(&end_point);

            if dist_to_start < dist_to_end {
                start_point
            } else {
                end_point
            }
        }
    }

    /// 指定した tolerance での包含判定
    pub fn contains_point_with_tolerance(&self, point: &Point2D<T>, tolerance: T) -> bool {
        // 基底円からの距離チェック
        let distance_to_center = self.center().distance_to(point);
        let radius_diff = (distance_to_center - self.radius()).abs();

        if radius_diff > tolerance {
            return false;
        }

        // 角度範囲チェック
        let point_angle = self.point_to_angle(point);
        self.contains_angle(point_angle)
    }
}
