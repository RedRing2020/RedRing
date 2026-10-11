//! Arc contracts.

use crate::Scalar;

pub trait Arc2DConstructor<T: Scalar> {
    fn new(center: (T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
    where
        Self: Sized;

    fn from_three_points(start: (T, T), mid: (T, T), end: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn full_circle(center: (T, T), radius: T) -> Self
    where
        Self: Sized;
}

pub trait Arc3DConstructor<T: Scalar> {
    fn new(
        center: (T, T, T),
        radius: T,
        normal: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    fn xy_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
    where
        Self: Sized;

    fn from_three_points(start: (T, T, T), mid: (T, T, T), end: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn full_circle(center: (T, T, T), normal: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait Arc2DProperties<T: Scalar> {
    fn center(&self) -> (T, T);
    fn radius(&self) -> T;
    fn start_angle(&self) -> T;
    fn end_angle(&self) -> T;
    fn dimension(&self) -> u32;
    fn angle_span(&self) -> T;
    fn is_full_circle(&self) -> bool;
}

pub trait Arc3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn start_angle(&self) -> T;
    fn end_angle(&self) -> T;
    fn dimension(&self) -> u32;
    fn angle_span(&self) -> T;
    fn is_full_circle(&self) -> bool;
}

pub trait Arc2DDerived<T: Scalar> {
    fn length(&self) -> T;
}

pub trait Arc2DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> (T, T);

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> (T, T);
}

pub trait Arc2DEvaluation<T: Scalar> {
    /// Arc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は Arc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母円の local angle parameter へ線形写像される。
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 母円の native angle parameter を直接指定して ideal evaluation point を取得
    ///
    /// 角度範囲内かどうかの判定は `contains_angle` 側の責務とし、この API は角度指定評価を表す。
    fn point_at_angle(&self, angle: T) -> (T, T);
}

pub trait Arc2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Arc2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
}

pub trait Arc2DTrimRange<T: Scalar> {
    fn contains_angle(&self, angle: T) -> bool;
}

pub trait Arc3DDerived<T: Scalar> {
    fn length(&self) -> T;
}

pub trait Arc3DEndpoint<T: Scalar> {
    /// 母曲線上の ideal な開始点を取得
    fn start_point(&self) -> (T, T, T);

    /// 母曲線上の ideal な終了点を取得
    fn end_point(&self) -> (T, T, T);
}

pub trait Arc3DEvaluation<T: Scalar> {
    /// Arc trim-local parameter `t` (`0 <= t <= 1`) に対応する ideal evaluation point を取得
    ///
    /// `t=0/1` は Arc のトリム区間の両端を指すが、拘束端点補間を意味しない。
    /// 評価時には母円の local angle parameter へ線形写像される。
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 母円の native angle parameter を直接指定して ideal evaluation point を取得
    fn point_at_angle(&self, angle: T) -> (T, T, T);
}

pub trait Arc3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait Arc3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait Arc3DTrimRange<T: Scalar> {
    fn contains_angle(&self, angle: T) -> bool;
}

pub trait Arc2DSampling<T: Scalar> {
    fn sample_points(&self, num_points: usize) -> Vec<(T, T)>;
}
