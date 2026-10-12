//! 3次元楕円弧（EllipseArc3D）の実装
//!
//! 母楕円と、開始角から反時計回りに終了角までの角度範囲（母楕円の local angle parameter）で定義する。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Ellipse3D, Point3D};
use analysis::AngleRange;
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, Angle, Ellipse3DConstructor,
    Ellipse3DDistance, Ellipse3DEvaluation, Ellipse3DProperties, EllipseArc3DConstructor,
    EllipseArc3DContainment, EllipseArc3DDerived, EllipseArc3DDistance, EllipseArc3DEndpoint,
    EllipseArc3DEvaluation, EllipseArc3DProperties, EllipseArc3DTrimRange, PrimitiveKind,
    PrimitiveMetadata, Scalar,
};

/// 3次元楕円弧
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseArc3D<T: Scalar> {
    ellipse: Ellipse3D<T>, // 基底楕円
    start_angle: Angle<T>, // 開始角度
    end_angle: Angle<T>,   // 終了角度
}

impl<T: Scalar> PrimitiveMetadata for EllipseArc3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }
}

impl<T: Scalar> EllipseArc3D<T> {
    /// 開始角から反時計回りに終了角まで進む角度範囲（開始角と終了角が等しい場合は `None`）
    fn angle_range(&self) -> Option<AngleRange<T>> {
        AngleRange::from_ccw_bounds(self.start_angle, self.end_angle)
    }
}

impl<T: Scalar> EllipseArc3DConstructor<T> for EllipseArc3D<T> {
    fn new(
        center: Point3D<T>,
        semi_major: T,
        semi_minor: T,
        normal: Direction3D<T>,
        major_axis_direction: Direction3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Some(Self {
            ellipse: Ellipse3D::new(center, semi_major, semi_minor, normal, major_axis_direction)?,
            start_angle,
            end_angle,
        })
    }
}

impl<T: Scalar> EllipseArc3DProperties<T> for EllipseArc3D<T> {
    fn center(&self) -> Point3D<T> {
        self.ellipse.center()
    }

    fn semi_major_axis(&self) -> T {
        self.ellipse.semi_major_axis()
    }

    fn semi_minor_axis(&self) -> T {
        self.ellipse.semi_minor_axis()
    }

    fn normal(&self) -> Direction3D<T> {
        self.ellipse.normal()
    }

    fn major_axis_direction(&self) -> Direction3D<T> {
        self.ellipse.major_axis_direction()
    }

    fn minor_axis_direction(&self) -> Direction3D<T> {
        self.ellipse.minor_axis_direction()
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    fn angle_span(&self) -> Angle<T> {
        let span = self.end_angle.to_radians() - self.start_angle.to_radians();
        if span < T::ZERO {
            Angle::from_radians(span + T::TAU)
        } else {
            Angle::from_radians(span)
        }
    }
}

impl<T: Scalar> EllipseArc3DDerived<T> for EllipseArc3D<T> {
    /// 母楕円の周長（ラマヌジャンの近似）に掃引角の比を掛けた値
    fn length(&self) -> T {
        self.ellipse.circumference() * (self.angle_span().to_radians() / T::TAU)
    }
}

impl<T: Scalar> EllipseArc3DEndpoint<T> for EllipseArc3D<T> {
    fn start_point(&self) -> Point3D<T> {
        self.point_at_angle(self.start_angle)
    }

    fn end_point(&self) -> Point3D<T> {
        self.point_at_angle(self.end_angle)
    }
}

impl<T: Scalar> EllipseArc3DEvaluation<T> for EllipseArc3D<T> {
    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        self.point_at_angle(Angle::from_radians(
            self.start_angle.to_radians() + self.angle_span().to_radians() * t,
        ))
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        self.ellipse.point_at_parameter(angle.to_radians())
    }
}

impl<T: Scalar> EllipseArc3DDistance<T> for EllipseArc3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        if self.contains_point_angle(point) {
            self.ellipse.distance_to_point(point)
        } else {
            point
                .distance_to(self.start_point())
                .min(point.distance_to(self.end_point()))
        }
    }
}

impl<T: Scalar> EllipseArc3DContainment<T> for EllipseArc3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> EllipseArc3DTrimRange<T> for EllipseArc3D<T> {
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        let tolerance = default_angle_tolerance::<T>();
        match self.angle_range() {
            Some(range) => range.contains(angle, tolerance),
            None => angle.is_equivalent(&self.start_angle, tolerance),
        }
    }

    fn contains_point_angle(&self, point: Point3D<T>) -> bool {
        self.contains_angle(Angle::from_radians(self.ellipse.parameter_for_point(point)))
    }
}
