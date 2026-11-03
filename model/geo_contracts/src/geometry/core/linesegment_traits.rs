//! LineSegment 形状の契約定義
//!
//! geo_primitives が実装すべき LineSegment 系の公開 trait。

use crate::{Point2D, Point3D, Scalar};

/// LineSegment2D生成のためのConstructorトレイト
pub trait LineSegment2DConstructor<T: Scalar> {
    /// 始点と終点から線分を作成する（2点が一致する場合は `None`）
    fn new(start: Point2D<T>, end: Point2D<T>) -> Option<Self>
    where
        Self: Sized;
}

/// LineSegment3D生成のためのConstructorトレイト
pub trait LineSegment3DConstructor<T: Scalar> {
    /// 始点と終点から線分を作成する（2点が一致する場合は `None`）
    fn new(start: Point3D<T>, end: Point3D<T>) -> Option<Self>
    where
        Self: Sized;
}

/// LineSegment2D基本プロパティ取得トレイト
pub trait LineSegment2DProperties<T: Scalar> {
    /// bounded curve primitive の ideal start endpoint を取得
    fn start_point(&self) -> Point2D<T>;

    /// bounded curve primitive の ideal end endpoint を取得
    fn end_point(&self) -> Point2D<T>;

    /// bounded curve primitive としての長さを取得
    fn length(&self) -> T;
}

/// LineSegment3D基本プロパティ取得トレイト
pub trait LineSegment3DProperties<T: Scalar> {
    /// bounded curve primitive の ideal start endpoint を取得
    fn start_point(&self) -> Point3D<T>;

    /// bounded curve primitive の ideal end endpoint を取得
    fn end_point(&self) -> Point3D<T>;

    /// bounded curve primitive としての長さを取得
    fn length(&self) -> T;
}

/// LineSegment3D の support line と拘束端点
///
/// 線分は support line と trim 区間（support line 上のパラメータ）を正本とし、
/// topology 連携用の拘束端点を別に保持できる。
pub trait LineSegment3DSupportLine<T: Scalar> {
    /// support line の型
    type Line;

    /// support line と拘束端点から線分を作成する（2点の support line 上のパラメータが一致する場合は `None`）
    fn from_support_line_and_constraint_points(
        line: Self::Line,
        start_point: Point3D<T>,
        end_point: Point3D<T>,
    ) -> Option<Self>
    where
        Self: Sized;

    /// support line を取得
    fn support_line(&self) -> &Self::Line;

    /// 始点の support line 上のパラメータを取得
    fn start_param(&self) -> T;

    /// 終点の support line 上のパラメータを取得
    fn end_param(&self) -> T;

    /// topology 連携用の拘束始点を取得
    fn constraint_start_point(&self) -> Point3D<T>;

    /// topology 連携用の拘束終点を取得
    fn constraint_end_point(&self) -> Point3D<T>;
}

pub trait LineSegment2DDistance<T: Scalar> {
    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

pub trait LineSegment2DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が線分上にあるかを判定する
    fn contains_point(&self, point: Point2D<T>) -> bool;
}

pub trait LineSegment2DEvaluation<T: Scalar> {
    /// 正規化 parameter `t`（始点 0・終点 1）に対応する support line 上の evaluation point を取得
    ///
    /// `t=0/1` は bounded curve primitive の ideal start/end endpoint と整合する。
    /// `[0, 1]` から距離トレランス分を超えて外れる `t` と、有限でない `t` は `None` を返す。
    fn point_at_parameter(&self, t: T) -> Option<Point2D<T>>;
}

pub trait LineSegment2DProjection<T: Scalar> {
    /// 点から線分への最近点を計算
    fn closest_point(&self, point: Point2D<T>) -> Point2D<T>;
}

pub trait LineSegment3DDistance<T: Scalar> {
    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}

pub trait LineSegment3DContainment<T: Scalar> {
    /// 既定の距離の許容誤差で、点が線分上にあるかを判定する
    fn contains_point(&self, point: Point3D<T>) -> bool;
}

pub trait LineSegment3DEvaluation<T: Scalar> {
    /// 正規化 parameter `t`（始点 0・終点 1）に対応する support line 上の evaluation point を取得
    ///
    /// `t=0/1` は bounded curve primitive の ideal start/end endpoint と整合する。
    /// `[0, 1]` から距離トレランス分を超えて外れる `t` と、有限でない `t` は `None` を返す。
    fn point_at_parameter(&self, t: T) -> Option<Point3D<T>>;
}

pub trait LineSegment3DProjection<T: Scalar> {
    /// 点から線分への最近点を計算
    fn closest_point(&self, point: Point3D<T>) -> Point3D<T>;
}
