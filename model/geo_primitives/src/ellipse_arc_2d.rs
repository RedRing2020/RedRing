//! 2次元楕円弧（EllipseArc2D）の実装
//!
//! 母楕円と、開始角から反時計回りに終了角までの角度範囲（母楕円の local angle parameter）で定義する。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Ellipse2D, Point2D};
use analysis::AngleRange;
use geo_contracts::{
    default_angle_tolerance, default_distance_tolerance, Angle, Ellipse2DConstructor,
    Ellipse2DEvaluation, Ellipse2DProperties, EllipseArc2DConstructor, EllipseArc2DContainment,
    EllipseArc2DDerived, EllipseArc2DDistance, EllipseArc2DEndpoint, EllipseArc2DEvaluation,
    EllipseArc2DProperties, EllipseArc2DTrimRange, PrimitiveKind, PrimitiveMetadata, Scalar,
};

/// 2次元楕円弧
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseArc2D<T: Scalar> {
    ellipse: Ellipse2D<T>, // 基底楕円
    start_angle: Angle<T>, // 開始角度
    end_angle: Angle<T>,   // 終了角度
}

impl<T: Scalar> PrimitiveMetadata for EllipseArc2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }
}

impl<T: Scalar> EllipseArc2D<T> {
    /// 開始角から反時計回りに終了角まで進む角度範囲（開始角と終了角が等しい場合は `None`）
    fn angle_range(&self) -> Option<AngleRange<T>> {
        AngleRange::from_ccw_bounds(self.start_angle, self.end_angle)
    }
}

impl<T: Scalar> EllipseArc2DConstructor<T> for EllipseArc2D<T> {
    fn new(
        center: Point2D<T>,
        semi_major: T,
        semi_minor: T,
        rotation: Angle<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Some(Self {
            ellipse: Ellipse2D::new(center, semi_major, semi_minor, rotation)?,
            start_angle,
            end_angle,
        })
    }
}

impl<T: Scalar> EllipseArc2DProperties<T> for EllipseArc2D<T> {
    fn center(&self) -> Point2D<T> {
        self.ellipse.center()
    }

    fn semi_major_axis(&self) -> T {
        self.ellipse.semi_major_axis()
    }

    fn semi_minor_axis(&self) -> T {
        self.ellipse.semi_minor_axis()
    }

    fn rotation(&self) -> Angle<T> {
        self.ellipse.rotation()
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

impl<T: Scalar> EllipseArc2DDerived<T> for EllipseArc2D<T> {
    /// 母楕円の周長（ラマヌジャンの近似）に掃引角の比を掛けた値
    fn length(&self) -> T {
        self.ellipse.circumference() * (self.angle_span().to_radians() / T::TAU)
    }
}

impl<T: Scalar> EllipseArc2DEndpoint<T> for EllipseArc2D<T> {
    fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle)
    }

    fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle)
    }
}

impl<T: Scalar> EllipseArc2DEvaluation<T> for EllipseArc2D<T> {
    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.point_at_angle(Angle::from_radians(
            self.start_angle.to_radians() + self.angle_span().to_radians() * t,
        ))
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Point2D<T> {
        self.ellipse.point_at_parameter(angle.to_radians())
    }
}

impl<T: Scalar> EllipseArc2DDistance<T> for EllipseArc2D<T> {
    /// 範囲内で距離が極値になる点と、両端の点までの距離の最小値
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        self.ellipse
            .distance_critical_parameters(point)
            .into_iter()
            .filter(|parameter| self.contains_angle(Angle::from_radians(*parameter)))
            .map(|parameter| self.ellipse.point_at_parameter(parameter))
            .chain([self.start_point(), self.end_point()])
            .map(|candidate| point.distance_to(candidate))
            .fold(T::INFINITY, T::min)
    }
}

impl<T: Scalar> EllipseArc2DContainment<T> for EllipseArc2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        self.distance_to_point(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> EllipseArc2DTrimRange<T> for EllipseArc2D<T> {
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        let tolerance = default_angle_tolerance::<T>();
        match self.angle_range() {
            Some(range) => range.contains(angle, tolerance),
            None => angle.is_equivalent(&self.start_angle, tolerance),
        }
    }

    fn contains_point_angle(&self, point: Point2D<T>) -> bool {
        self.contains_angle(Angle::from_radians(self.ellipse.parameter_for_point(point)))
    }
}
