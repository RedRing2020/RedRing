//! EllipsoidalSolid3D の交差判定実装
//!
//! 楕円体ソリッドとの交点計算を提供する。

use crate::{EllipsoidalSolid3D, InfiniteLine3D, Plane3D, Point3D, Ray3D};
use geo_contracts::{
    BasicIntersection, InfiniteLine3DProperties, MultipleIntersection, Scalar, SelfIntersection,
};

// ============================================================================
// BasicIntersection implementations for EllipsoidalSolid3D
// ============================================================================

/// EllipsoidalSolid3D と Point3D の交差判定
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for EllipsoidalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, _tolerance: T) -> Option<Self::Point> {
        if self.contains_point(point) {
            // 点が楕円体ソリッド内または表面上にある場合、その点を返す
            Some(*point)
        } else {
            None
        }
    }
}

/// EllipsoidalSolid3D と Plane3D の交差判定
impl<T: Scalar> BasicIntersection<T, Plane3D<T>> for EllipsoidalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, plane: &Plane3D<T>, _tolerance: T) -> Option<Self::Point> {
        // 簡易実装：中心と平面の交点のみ考慮
        // TODO: 平面と楕円体ソリッドの楕円交差を返す
        let center = self.center_internal();
        let distance = plane.distance_to_point(center).abs();

        // 最大半径（簡易的に3軸の最大値を使用）
        let max_radius = self
            .a_radius_internal()
            .max(self.b_radius_internal())
            .max(self.c_radius_internal());

        if distance <= max_radius {
            Some(center)
        } else {
            None
        }
    }
}

// ============================================================================
// MultipleIntersection implementations for EllipsoidalSolid3D
// ============================================================================

/// EllipsoidalSolid3D と InfiniteLine3D の複数交差判定
impl<T: Scalar> MultipleIntersection<T, InfiniteLine3D<T>> for EllipsoidalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, line: &InfiniteLine3D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装: 直線のパラメトリック表現と楕円体方程式の連立
        // 正確な実装には2次方程式の解が必要
        // TODO: より正確な楕円体と直線の交点計算

        let (px, py, pz) = line.point();
        let point_on_line = Point3D::new(px, py, pz);

        // 直線上の点が楕円体内にあるかチェック
        if self.contains_point(&point_on_line) {
            vec![point_on_line]
        } else {
            vec![]
        }
    }
}

/// EllipsoidalSolid3D と Ray3D の複数交差判定
impl<T: Scalar> MultipleIntersection<T, Ray3D<T>> for EllipsoidalSolid3D<T> {
    type Point = Point3D<T>;

    fn intersections_with(&self, ray: &Ray3D<T>, _tolerance: T) -> Vec<Self::Point> {
        // 簡易実装: Ray の起点が楕円体内にあるかチェック
        // TODO: より正確な楕円体と Ray の交点計算

        let origin = ray.origin_internal();

        if self.contains_point(&origin) {
            vec![origin]
        } else {
            vec![]
        }
    }
}

// ============================================================================
// SelfIntersection implementations for EllipsoidalSolid3D
// ============================================================================

/// EllipsoidalSolid3D と EllipsoidalSolid3D の自己交差判定
impl<T: Scalar> SelfIntersection<T> for EllipsoidalSolid3D<T> {
    type Point = Point3D<T>;

    fn self_intersections(&self, _tolerance: T) -> Vec<Self::Point> {
        // 楕円体ソリッド単体では自己交差は発生しない
        vec![]
    }
}
