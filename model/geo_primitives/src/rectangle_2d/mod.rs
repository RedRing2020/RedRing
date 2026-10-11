//! Rect2D Core 実装

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::Point2D;
use geo_contracts::{
    default_distance_tolerance, PointClassification, PrimitiveKind, PrimitiveMetadata,
    Rect2DConstructor, Rect2DContainment, Rect2DDerived, Rect2DProperties, Scalar,
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
    pub fn new(origin: Point2D<T>, width: T, height: T) -> Option<Self> {
        if width < T::ZERO || height < T::ZERO {
            return None;
        }
        Some(Self {
            origin,
            width,
            height,
        })
    }

    pub fn origin_point(&self) -> Point2D<T> {
        self.origin
    }

    pub fn width_value(&self) -> T {
        self.width
    }

    pub fn height_value(&self) -> T {
        self.height
    }

    pub fn max_point(&self) -> Point2D<T> {
        Point2D::new(self.origin.x() + self.width, self.origin.y() + self.height)
    }

    pub fn center_point(&self) -> Point2D<T> {
        let two = T::ONE + T::ONE;
        Point2D::new(
            self.origin.x() + self.width / two,
            self.origin.y() + self.height / two,
        )
    }

    pub fn resize(&mut self, width: T, height: T) -> bool {
        if width < T::ZERO || height < T::ZERO {
            return false;
        }
        self.width = width;
        self.height = height;
        true
    }

    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        let max = self.max_point();
        point.x() >= self.origin.x()
            && point.x() <= max.x()
            && point.y() >= self.origin.y()
            && point.y() <= max.y()
    }

    /// 矩形の領域に対する点の位置を分類する
    ///
    /// 辺までの距離が `tolerance` 以内なら `OnBoundary`、それ以外は内部・外部に分ける。
    /// `contains_point` は `classify_point(point, 0)` が `Outside` でないことと一致する。
    pub fn classify_point(&self, point: &Point2D<T>, tolerance: T) -> PointClassification {
        geo_core::Aabb2D::new(self.origin, self.max_point()).classify_point(point, tolerance)
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn corners(&self) -> [Point2D<T>; 4] {
        let max = self.max_point();
        [
            self.origin,
            Point2D::new(max.x(), self.origin.y()),
            max,
            Point2D::new(self.origin.x(), max.y()),
        ]
    }
}

impl<T: Scalar> Rect2DConstructor<T> for Rect2D<T> {
    fn new(origin: (T, T), width: T, height: T) -> Option<Self> {
        Self::new(Point2D::new(origin.0, origin.1), width, height)
    }

    fn resize(&mut self, width: T, height: T) -> bool {
        Self::resize(self, width, height)
    }
}

impl<T: Scalar> Rect2DProperties<T> for Rect2D<T> {
    fn origin(&self) -> (T, T) {
        (self.origin.x(), self.origin.y())
    }

    fn width(&self) -> T {
        self.width
    }

    fn height(&self) -> T {
        self.height
    }

    fn center(&self) -> (T, T) {
        let center = self.center_point();
        (center.x(), center.y())
    }
}

impl<T: Scalar> Rect2DContainment<T> for Rect2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        Self::contains_point(self, &Point2D::new(point.0, point.1))
    }

    fn classify_point(&self, point: (T, T)) -> PointClassification {
        Self::classify_point(
            self,
            &Point2D::new(point.0, point.1),
            default_distance_tolerance::<T>(),
        )
    }
}

impl<T: Scalar> Rect2DDerived<T> for Rect2D<T> {
    fn is_valid(&self) -> bool {
        self.width >= T::ZERO && self.height >= T::ZERO
    }
}
