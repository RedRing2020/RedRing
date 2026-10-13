//! Rect2D 実装（軸平行の矩形）

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::Point2D;
use geo_contracts::{
    default_distance_tolerance, PointClassification, PrimitiveKind, PrimitiveMetadata,
    Rect2DConstructor, Rect2DContainment, Rect2DProperties, Scalar,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect2D<T: Scalar> {
    origin: Point2D<T>,
    width: T,
    height: T,
}

impl<T: Scalar> PrimitiveMetadata for Rect2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }
}

impl<T: Scalar> Rect2D<T> {
    /// 原点の対角の角
    fn max_point(&self) -> Point2D<T> {
        Point2D::new(self.origin.x() + self.width, self.origin.y() + self.height)
    }
}

impl<T: Scalar> Rect2DConstructor<T> for Rect2D<T> {
    fn new(origin: Point2D<T>, width: T, height: T) -> Option<Self> {
        (width >= T::ZERO && height >= T::ZERO).then_some(Self {
            origin,
            width,
            height,
        })
    }
}

impl<T: Scalar> Rect2DProperties<T> for Rect2D<T> {
    fn origin(&self) -> Point2D<T> {
        self.origin
    }

    fn width(&self) -> T {
        self.width
    }

    fn height(&self) -> T {
        self.height
    }

    fn center(&self) -> Point2D<T> {
        let two = T::ONE + T::ONE;
        Point2D::new(
            self.origin.x() + self.width / two,
            self.origin.y() + self.height / two,
        )
    }
}

impl<T: Scalar> Rect2DContainment<T> for Rect2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        let max = self.max_point();
        point.x() >= self.origin.x()
            && point.x() <= max.x()
            && point.y() >= self.origin.y()
            && point.y() <= max.y()
    }

    /// 辺までの距離が既定の許容誤差以内なら `OnBoundary`、それ以外は内部・外部に分ける。
    fn classify_point(&self, point: Point2D<T>) -> PointClassification {
        geo_core::Aabb2D::new(self.origin, self.max_point())
            .classify_point(point, default_distance_tolerance::<T>())
    }
}
