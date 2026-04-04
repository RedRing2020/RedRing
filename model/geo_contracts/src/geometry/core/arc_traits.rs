//! Arc contracts.

use crate::Scalar;

pub trait Arc2DConstructor<T: Scalar> {
    fn new(center: (T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
    where
        Self: Sized;

    fn from_three_points(start: (T, T), mid: (T, T), end: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn semicircle(center: (T, T), radius: T) -> Self
    where
        Self: Sized;

    fn from_center_and_points(center: (T, T), start: (T, T), end: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn full_circle(center: (T, T), radius: T) -> Self
    where
        Self: Sized;

    fn unit_semicircle() -> Self
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

    fn xz_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
    where
        Self: Sized;

    fn yz_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
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
    fn is_semicircle(&self) -> bool;
}

pub trait Arc3DProperties<T: Scalar> {
    fn center(&self) -> (T, T, T);
    fn radius(&self) -> T;
    fn start_angle(&self) -> T;
    fn end_angle(&self) -> T;
    fn dimension(&self) -> u32;
    fn angle_span(&self) -> T;
    fn is_full_circle(&self) -> bool;
    fn is_on_xy_plane(&self) -> bool;
}

// TODO(#318): Measure 契約は責務分離フェーズで shape definition から分離する。
pub trait Arc2DDerived<T: Scalar> {
    fn measure(&self) -> T;
}

pub trait Arc2DEndpoint<T: Scalar> {
    fn start_point(&self) -> (T, T);
    fn end_point(&self) -> (T, T);
    fn midpoint(&self) -> (T, T);
}

pub trait Arc2DEvaluation<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T);
    fn point_at_angle(&self, angle: T) -> (T, T);
}

pub trait Arc2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Arc2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
    fn contains_angle(&self, angle: T) -> bool;
}

pub trait Arc3DDerived<T: Scalar> {
    fn measure(&self) -> T;
}

pub trait Arc3DEndpoint<T: Scalar> {
    fn start_point(&self) -> (T, T, T);
    fn end_point(&self) -> (T, T, T);
    fn midpoint(&self) -> (T, T, T);
}

pub trait Arc3DEvaluation<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T, T);
    fn point_at_angle(&self, angle: T) -> (T, T, T);
}

pub trait Arc3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait Arc3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait Arc2DSampling<T: Scalar> {
    fn sample_points(&self, num_points: usize) -> Vec<(T, T)>;
    fn sample_by_arc_length(&self, arc_length_step: T) -> Vec<(T, T)>;
}

pub trait Arc2DCore<T: Scalar>: Arc2DConstructor<T> + Arc2DProperties<T> {}
pub trait Arc3DCore<T: Scalar>: Arc3DConstructor<T> + Arc3DProperties<T> {}

impl<T: Scalar, A> Arc2DCore<T> for A where A: Arc2DConstructor<T> + Arc2DProperties<T> {}

impl<T: Scalar, A> Arc3DCore<T> for A where A: Arc3DConstructor<T> + Arc3DProperties<T> {}
