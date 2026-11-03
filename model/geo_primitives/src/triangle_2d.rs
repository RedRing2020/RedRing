//! Triangle2D 実装

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Point2D, Vector2D};
use geo_contracts::{
    default_distance_tolerance, default_kernel_numerical_zero_tolerance, PointClassification,
    PrimitiveKind, PrimitiveMetadata, Scalar, Triangle2DBoundaryAccess, Triangle2DConstructor,
    Triangle2DContainment, Triangle2DDistance,
};

/// 2次元三角形
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle2D<T: Scalar> {
    vertex_a: Point2D<T>,
    vertex_b: Point2D<T>,
    vertex_c: Point2D<T>,
}

impl<T: Scalar> PrimitiveMetadata for Triangle2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }
}

impl<T: Scalar> Triangle2D<T> {
    /// 頂点 A を基準とした重心座標（C 方向の重み・B 方向の重み・A の重み）
    fn barycentric_coordinates(&self, point: Point2D<T>) -> (T, T, T) {
        let v0 = Vector2D::from_points(self.vertex_a, self.vertex_c);
        let v1 = Vector2D::from_points(self.vertex_a, self.vertex_b);
        let v2 = Vector2D::from_points(self.vertex_a, point);

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = T::ONE / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
        let w = T::ONE - u - v;

        (u, v, w)
    }

    /// 3辺までの距離の最小値
    fn distance_to_boundary(&self, point: Point2D<T>) -> T {
        Self::distance_to_edge(point, self.vertex_a, self.vertex_b)
            .min(Self::distance_to_edge(point, self.vertex_b, self.vertex_c))
            .min(Self::distance_to_edge(point, self.vertex_c, self.vertex_a))
    }

    /// 点から線分までの距離
    fn distance_to_edge(point: Point2D<T>, p1: Point2D<T>, p2: Point2D<T>) -> T {
        let edge = Vector2D::from_points(p1, p2);
        let to_point = Vector2D::from_points(p1, point);

        let edge_length_sq = edge.dot(edge);
        if edge_length_sq < default_kernel_numerical_zero_tolerance::<T>() {
            return to_point.length();
        }

        let t = (to_point.dot(edge) / edge_length_sq)
            .max(T::ZERO)
            .min(T::ONE);
        point.distance_to(p1 + edge * t)
    }
}

impl<T: Scalar> Triangle2DConstructor<T> for Triangle2D<T> {
    fn new(a: Point2D<T>, b: Point2D<T>, c: Point2D<T>) -> Option<Self> {
        let ab = Vector2D::from_points(a, b);
        let ac = Vector2D::from_points(a, c);
        if ab.cross(ac).abs() < default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        Some(Self {
            vertex_a: a,
            vertex_b: b,
            vertex_c: c,
        })
    }
}

impl<T: Scalar> Triangle2DBoundaryAccess<T> for Triangle2D<T> {
    fn vertex_a(&self) -> Point2D<T> {
        self.vertex_a
    }

    fn vertex_b(&self) -> Point2D<T> {
        self.vertex_b
    }

    fn vertex_c(&self) -> Point2D<T> {
        self.vertex_c
    }
}

impl<T: Scalar> Triangle2DContainment<T> for Triangle2D<T> {
    fn contains_point(&self, point: Point2D<T>) -> bool {
        let (u, v, w) = self.barycentric_coordinates(point);
        u >= T::ZERO && v >= T::ZERO && w >= T::ZERO
    }

    /// 辺までの距離が既定の許容誤差以内なら `OnBoundary`、それ以外は内部・外部に分ける。
    fn classify_point(&self, point: Point2D<T>) -> PointClassification {
        if self.distance_to_boundary(point) <= default_distance_tolerance::<T>() {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> Triangle2DDistance<T> for Triangle2D<T> {
    fn distance_to_point(&self, point: Point2D<T>) -> T {
        if self.contains_point(point) {
            T::ZERO
        } else {
            self.distance_to_boundary(point)
        }
    }
}
