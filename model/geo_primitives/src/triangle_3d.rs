//! Triangle3D 実装

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, default_kernel_numerical_zero_tolerance, PointClassification,
    PrimitiveKind, PrimitiveMetadata, Scalar, TolerantEq, Triangle3DBoundaryAccess,
    Triangle3DConstructor, Triangle3DContainment, Triangle3DDerived, Triangle3DDistance,
};

/// 3次元三角形
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle3D<T: Scalar> {
    vertex_a: Point3D<T>,
    vertex_b: Point3D<T>,
    vertex_c: Point3D<T>,
}

impl<T: Scalar> PrimitiveMetadata for Triangle3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }
}

impl<T: Scalar> TolerantEq<T> for Triangle3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        self.vertex_a.distance_to(other.vertex_a) <= tolerance
            && self.vertex_b.distance_to(other.vertex_b) <= tolerance
            && self.vertex_c.distance_to(other.vertex_c) <= tolerance
    }
}

impl<T: Scalar> std::fmt::Display for Triangle3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Triangle3D(A: {:?}, B: {:?}, C: {:?})",
            self.vertex_a, self.vertex_b, self.vertex_c
        )
    }
}

impl<T: Scalar> Triangle3D<T> {
    /// 三角形の平面上の点が三角形の内部または境界上にあるか（重心座標）
    fn contains_point_on_plane(&self, point: Point3D<T>) -> bool {
        let v0 = Vector3D::from_points(self.vertex_c, self.vertex_a);
        let v1 = Vector3D::from_points(self.vertex_c, self.vertex_b);
        let v2 = Vector3D::from_points(self.vertex_c, point);

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = T::ONE / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        u >= T::ZERO && v >= T::ZERO && (u + v) <= T::ONE
    }

    /// 3辺までの距離の最小値
    fn distance_to_boundary(&self, point: Point3D<T>) -> T {
        Self::distance_to_edge(point, self.vertex_a, self.vertex_b)
            .min(Self::distance_to_edge(point, self.vertex_b, self.vertex_c))
            .min(Self::distance_to_edge(point, self.vertex_c, self.vertex_a))
    }

    /// 点から線分までの距離
    fn distance_to_edge(point: Point3D<T>, p1: Point3D<T>, p2: Point3D<T>) -> T {
        let edge = Vector3D::from_points(p1, p2);
        let to_point = Vector3D::from_points(p1, point);

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

impl<T: Scalar> Triangle3DConstructor<T> for Triangle3D<T> {
    fn new(a: Point3D<T>, b: Point3D<T>, c: Point3D<T>) -> Option<Self> {
        let cross = Vector3D::from_points(a, b).cross(Vector3D::from_points(a, c));
        if cross.length() < default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        Some(Self {
            vertex_a: a,
            vertex_b: b,
            vertex_c: c,
        })
    }
}

impl<T: Scalar> Triangle3DBoundaryAccess<T> for Triangle3D<T> {
    fn vertex_a(&self) -> Point3D<T> {
        self.vertex_a
    }

    fn vertex_b(&self) -> Point3D<T> {
        self.vertex_b
    }

    fn vertex_c(&self) -> Point3D<T> {
        self.vertex_c
    }
}

impl<T: Scalar> Triangle3DDerived<T> for Triangle3D<T> {
    fn normal(&self) -> Direction3D<T> {
        let cross = Vector3D::from_points(self.vertex_a, self.vertex_b)
            .cross(Vector3D::from_points(self.vertex_a, self.vertex_c));
        Direction3D::from_vector(cross).expect("Non-degenerate triangle should have a normal")
    }
}

impl<T: Scalar> Triangle3DContainment<T> for Triangle3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.classify_point(point) != PointClassification::Outside
    }

    /// 辺までの距離が既定の許容誤差以内なら、平面外でも `OnBoundary` とする。それ以外は、平面からの距離が
    /// 許容誤差以内で、平面へ投影した点が三角形の内側にあれば `Inside`、それ以外を `Outside` とする。
    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        let tolerance = default_distance_tolerance::<T>();
        if self.distance_to_boundary(point) <= tolerance {
            return PointClassification::OnBoundary;
        }

        let plane_distance =
            Vector3D::from_points(self.vertex_a, point).dot(self.normal().as_vector());
        if plane_distance.abs() <= tolerance && self.contains_point_on_plane(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}

impl<T: Scalar> Triangle3DDistance<T> for Triangle3D<T> {
    /// 平面へ投影した点が三角形の内側なら平面までの距離、それ以外は辺までの距離
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        let normal = self.normal().as_vector();
        let plane_distance = Vector3D::from_points(self.vertex_a, point).dot(normal);
        let projected = point - normal * plane_distance;

        if self.contains_point_on_plane(projected) {
            plane_distance.abs()
        } else {
            self.distance_to_boundary(point)
        }
    }
}
