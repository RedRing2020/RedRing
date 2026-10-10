//! LineSegment3D Extensions 実装
//!
//! Foundation統一システムに基づくLineSegment3Dの拡張機能
//! Core機能は line_segment_3d.rs を参照

use crate::{LineSegment3D, Point3D, Vector3D};
use geo_contracts::{default_distance_tolerance, Scalar};

// Note: Copy trait cannot be implemented due to InfiniteLine3D field not implementing Copy

impl<T: Scalar> std::fmt::Display for LineSegment3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LineSegment3D(start: {:?}, end: {:?})",
            self.start_point(),
            self.end_point()
        )
    }
}

impl<T: Scalar> LineSegment3D<T> {
    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        let start = self.start_point();
        let end = self.end_point();
        geo_core::Aabb3D::from_points(&[start, end])
            .unwrap_or_else(|| geo_core::Aabb3D::new(start, start))
    }

    /// 正規化パラメータ `t`（始点 0・終点 1）での点を取得
    ///
    /// `[0, 1]` から距離トレランス分を超えて外れる `t` と、有限でない `t` は `None` を返す。
    /// 距離トレランス内のはみ出しは support line 上に外挿する。
    pub fn point_at_parameter(&self, t: T) -> Option<Point3D<T>> {
        if !self.is_parameter_in_domain(t) {
            return None;
        }

        let param = self.start_param + t * (self.end_param - self.start_param);
        Some(self.line.point_at_parameter(param))
    }

    /// 正規化パラメータ `t` が評価できる範囲内かを判定
    ///
    /// 許容するはみ出しは距離トレランスを線分の長さで割った値とし、評価点と線分の距離が
    /// 距離トレランス以下となるようにする。
    fn is_parameter_in_domain(&self, t: T) -> bool {
        if !t.is_finite() {
            return false;
        }
        let margin = default_distance_tolerance::<T>() / self.length();
        t >= -margin && t <= T::ONE + margin
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    /// パラメータでの接線方向を取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector3D<T> {
        self.direction()
    }

    /// 線分の逆方向を作成
    pub fn reverse(&self) -> Self {
        Self {
            line: self.line,
            start_param: self.end_param,
            end_param: self.start_param,
            start_point: self.constraint_end_point(),
            end_point: self.constraint_start_point(),
        }
    }

    /// 点を線分に投影
    pub fn closest_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = Vector3D::from_points(&self.line().point_internal(), point);
        let t = to_point.dot(&self.line().direction_internal());

        // パラメータを線分の範囲内に制限する。reverse した線分は start_param > end_param となるため、
        // 小さい側・大きい側のパラメータで制限する
        let (min_param, max_param) = self.ordered_params();
        let clamped_param = t.max(min_param).min(max_param);

        self.line().point_at_parameter(clamped_param)
    }

    /// 点を support line に投影した位置の正規化パラメータを取得
    ///
    /// 始点 0・終点 1 とし、`[0, 1]` に制限しない。
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let line_param = self.line.parameter_for_point(point);
        (line_param - self.start_param) / (self.end_param - self.start_param)
    }

    /// 線分上で点に最も近い点の正規化パラメータを取得
    ///
    /// 始点 0・終点 1 とし、`[0, 1]` に制限する。reverse した線分（start_param > end_param）でも
    /// 始点からの比率になるよう、support line のパラメータではなく正規化後の値で制限する。
    pub fn closest_parameter(&self, point: &Point3D<T>) -> T {
        self.parameter_for_point(point).max(T::ZERO).min(T::ONE)
    }
}
