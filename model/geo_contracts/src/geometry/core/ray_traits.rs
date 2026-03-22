//! Ray 形状の契約定義
//!
//! geo_primitives が実装すべき Ray 系の公開 trait。
//! analysis との結合を避けるため origin / direction はタプルで返す。

use crate::Scalar;

pub trait Ray2DConstructor<T: Scalar> {
    fn new(origin: (T, T), direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_points(start: (T, T), through: (T, T)) -> Option<Self>
    where
        Self: Sized;

    fn along_positive_x(origin: (T, T)) -> Self
    where
        Self: Sized;

    fn along_positive_y(origin: (T, T)) -> Self
    where
        Self: Sized;

    fn along_negative_x(origin: (T, T)) -> Self
    where
        Self: Sized;

    fn along_negative_y(origin: (T, T)) -> Self
    where
        Self: Sized;

    fn x_axis() -> Self
    where
        Self: Sized;

    fn y_axis() -> Self
    where
        Self: Sized;

    fn from_angle(origin: (T, T), angle: T) -> Self
    where
        Self: Sized;

    fn horizontal_right() -> Self
    where
        Self: Sized;

    fn vertical_up() -> Self
    where
        Self: Sized;
}

pub trait Ray3DConstructor<T: Scalar> {
    fn new(origin: (T, T, T), direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn from_points(start: (T, T, T), through: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    fn along_positive_x(origin: (T, T, T)) -> Self
    where
        Self: Sized;

    fn along_positive_y(origin: (T, T, T)) -> Self
    where
        Self: Sized;

    fn along_positive_z(origin: (T, T, T)) -> Self
    where
        Self: Sized;

    fn along_negative_x(origin: (T, T, T)) -> Self
    where
        Self: Sized;

    fn along_negative_y(origin: (T, T, T)) -> Self
    where
        Self: Sized;

    fn along_negative_z(origin: (T, T, T)) -> Self
    where
        Self: Sized;

    fn x_axis() -> Self
    where
        Self: Sized;

    fn y_axis() -> Self
    where
        Self: Sized;

    fn z_axis() -> Self
    where
        Self: Sized;

    fn from_spherical(origin: (T, T, T), azimuth: T, elevation: T) -> Self
    where
        Self: Sized;

    fn xy_plane_angle(origin: (T, T, T), angle: T) -> Self
    where
        Self: Sized;

    fn xz_plane_angle(origin: (T, T, T), angle: T) -> Self
    where
        Self: Sized;
}

pub trait Ray2DProperties<T: Scalar> {
    fn origin(&self) -> (T, T);
    fn direction(&self) -> (T, T);
    fn origin_x(&self) -> T;
    fn origin_y(&self) -> T;
    fn direction_x(&self) -> T;
    fn direction_y(&self) -> T;
    fn is_valid(&self) -> bool;
    fn dimension(&self) -> usize {
        2
    }
    fn angle(&self) -> T;
    fn is_horizontal(&self) -> bool;
    fn is_vertical(&self) -> bool;
}

pub trait Ray3DProperties<T: Scalar> {
    fn origin(&self) -> (T, T, T);
    fn direction(&self) -> (T, T, T);
    fn origin_x(&self) -> T;
    fn origin_y(&self) -> T;
    fn origin_z(&self) -> T;
    fn direction_x(&self) -> T;
    fn direction_y(&self) -> T;
    fn direction_z(&self) -> T;
    fn is_valid(&self) -> bool;
    fn dimension(&self) -> usize {
        3
    }
    fn azimuth(&self) -> T;
    fn elevation(&self) -> T;
    fn is_on_xy_plane(&self) -> bool;
}

pub trait Ray2DMeasure<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T);
    fn closest_point(&self, point: (T, T)) -> (T, T);
    fn distance_to_point(&self, point: (T, T)) -> T;
    fn contains_point(&self, point: (T, T)) -> bool;
    fn parameter_for_point(&self, point: (T, T)) -> T;
    fn points_towards(&self, direction: (T, T)) -> bool;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_same_direction(&self, other: &Self) -> bool;
    fn is_opposite_direction(&self, other: &Self) -> bool;
    fn reverse(&self) -> Self
    where
        Self: Sized;
    fn translate(&self, offset: (T, T)) -> Self
    where
        Self: Sized;
    fn intersection_with_ray(&self, other: &Self) -> Option<(T, T)>;
    fn point_at_distance(&self, distance: T) -> (T, T);
    fn angle_between(&self, other: &Self) -> T;
    fn rotate_around_origin(&self, angle: T) -> Self
    where
        Self: Sized;
}

pub trait Ray3DMeasure<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T, T);
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T);
    fn distance_to_point(&self, point: (T, T, T)) -> T;
    fn contains_point(&self, point: (T, T, T)) -> bool;
    fn parameter_for_point(&self, point: (T, T, T)) -> T;
    fn points_towards(&self, direction: (T, T, T)) -> bool;
    fn is_parallel_to(&self, other: &Self) -> bool;
    fn is_same_direction(&self, other: &Self) -> bool;
    fn is_opposite_direction(&self, other: &Self) -> bool;
    fn reverse(&self) -> Self
    where
        Self: Sized;
    fn translate(&self, offset: (T, T, T)) -> Self
    where
        Self: Sized;
    #[deprecated(
        since = "0.1.0",
        note = "cross-shape distance contracts are migrating to geometry::operations::CrossDistance"
    )]
    fn distance_to_ray(&self, other: &Self) -> T;
    fn point_at_distance(&self, distance: T) -> (T, T, T);
    fn angle_between(&self, other: &Self) -> T;
    fn rotate_around_axis(&self, axis: (T, T, T), angle: T) -> Option<Self>
    where
        Self: Sized;
}

pub trait Ray2DCore<T: Scalar>: Ray2DConstructor<T> + Ray2DProperties<T> + Ray2DMeasure<T> {}

pub trait Ray3DCore<T: Scalar>: Ray3DConstructor<T> + Ray3DProperties<T> + Ray3DMeasure<T> {}

impl<T: Scalar, R> Ray2DCore<T> for R where
    R: Ray2DConstructor<T> + Ray2DProperties<T> + Ray2DMeasure<T>
{
}

impl<T: Scalar, R> Ray3DCore<T> for R where
    R: Ray3DConstructor<T> + Ray3DProperties<T> + Ray3DMeasure<T>
{
}
