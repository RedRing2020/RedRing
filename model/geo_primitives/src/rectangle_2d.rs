//! Rect2D Core 実装

use crate::Point2D;
use geo_foundation::{
    geometry::core::rectangle_traits::{Rect2DConstructor, Rect2DMeasure, Rect2DProperties},
    Scalar,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect2D<T: Scalar> {
    origin: Point2D<T>,
    width: T,
    height: T,
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

    pub fn from_corners(min: Point2D<T>, max: Point2D<T>) -> Option<Self> {
        let width = max.x() - min.x();
        let height = max.y() - min.y();
        Self::new(min, width, height)
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

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn perimeter(&self) -> T {
        let two = T::ONE + T::ONE;
        two * (self.width + self.height)
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

    fn from_corners(min: (T, T), max: (T, T)) -> Option<Self> {
        Self::from_corners(Point2D::new(min.0, min.1), Point2D::new(max.0, max.1))
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

    fn max_corner(&self) -> (T, T) {
        let max = self.max_point();
        (max.x(), max.y())
    }

    fn center(&self) -> (T, T) {
        let center = self.center_point();
        (center.x(), center.y())
    }
}

impl<T: Scalar> Rect2DMeasure<T> for Rect2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        Self::contains_point(self, &Point2D::new(point.0, point.1))
    }

    fn area(&self) -> T {
        Self::area(self)
    }

    fn perimeter(&self) -> T {
        Self::perimeter(self)
    }

    fn is_valid(&self) -> bool {
        self.width >= T::ZERO && self.height >= T::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_and_resize() {
        let mut rect = Rect2D::new(Point2D::new(10.0, 20.0), 30.0, 40.0).unwrap();
        assert!(rect.contains_point(&Point2D::new(10.0, 20.0)));
        assert!(rect.contains_point(&Point2D::new(40.0, 60.0)));
        assert!(!rect.contains_point(&Point2D::new(41.0, 60.0)));

        assert!(rect.resize(50.0, 10.0));
        assert_eq!(rect.width_value(), 50.0);
        assert_eq!(rect.height_value(), 10.0);
        assert!(!rect.resize(-1.0, 10.0));
    }
}
