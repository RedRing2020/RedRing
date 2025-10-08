//! 幾何計算用ユーティリティ関数
//!
//! f64ベース幾何プリミティブ間の変換とヘルパー関数を提供

// geo_foundation参照を使用 - Scalarを抽象化
use crate::geometry2d::{Point2D, Point2DF64};
use crate::geometry3d::point::Point3DF64 as Point3D;
use geo_foundation::Scalar;

/// Point2Dからf64タプルに変換
pub fn point2d_to_f64(point: &Point2DF64) -> (f64, f64) {
    (point.x(), point.y())
}

/// Point3Dからf64タプルに変換
pub fn point3d_to_f64(point: &Point3D) -> (f64, f64, f64) {
    (point.x(), point.y(), point.z())
}

/// f64から新しいPoint2Dを作成
pub fn point2d_from_f64(x: f64, y: f64) -> Point2DF64 {
    Point2D::new(x, y)
}

/// f64値からPoint3Dを作成
pub fn point3d_from_f64(x: f64, y: f64, z: f64) -> Point3D {
    Point3D::new(x, y, z)
}

/// 2つのScalar値の距離（f64として計算）
pub fn scalar_distance<T: Scalar>(a: T, b: T) -> T {
    (a - b).abs()
}

/// 2つのf64値の最小値
pub fn f64_min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

/// 2つのf64値の最大値
pub fn f64_max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Point2Dのf64値でのbounding box計算
pub fn point2d_bounding_box(points: &[Point2DF64]) -> Option<(f64, f64, f64, f64)> {
    if points.is_empty() {
        return None;
    }

    let first = point2d_to_f64(&points[0]);
    let mut min_x = first.0;
    let mut min_y = first.1;
    let mut max_x = first.0;
    let mut max_y = first.1;

    for point in points.iter().skip(1) {
        let (x, y) = point2d_to_f64(point);
        min_x = f64_min(min_x, x);
        min_y = f64_min(min_y, y);
        max_x = f64_max(max_x, x);
        max_y = f64_max(max_y, y);
    }

    Some((min_x, min_y, max_x, max_y))
}

/// Point3Dのf64値でのbounding box計算
pub fn point3d_bounding_box(points: &[Point3D]) -> Option<(f64, f64, f64, f64, f64, f64)> {
    if points.is_empty() {
        return None;
    }

    let first = point3d_to_f64(&points[0]);
    let mut min_x = first.0;
    let mut min_y = first.1;
    let mut min_z = first.2;
    let mut max_x = first.0;
    let mut max_y = first.1;
    let mut max_z = first.2;

    for point in points.iter().skip(1) {
        let (x, y, z) = point3d_to_f64(point);
        min_x = f64_min(min_x, x);
        min_y = f64_min(min_y, y);
        min_z = f64_min(min_z, z);
        max_x = f64_max(max_x, x);
        max_y = f64_max(max_y, y);
        max_z = f64_max(max_z, z);
    }

    Some((min_x, min_y, min_z, max_x, max_y, max_z))
}

/// Point2Dの重心をf64で計算
pub fn point2d_centroid(points: &[Point2DF64]) -> Option<Point2DF64> {
    if points.is_empty() {
        return None;
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;

    for point in points {
        let (x, y) = point2d_to_f64(point);
        sum_x += x;
        sum_y += y;
    }

    let count = points.len() as f64;
    Some(point2d_from_f64(sum_x / count, sum_y / count))
}

/// Point3Dの重心をf64で計算
pub fn point3d_centroid(points: &[Point3D]) -> Option<Point3D> {
    if points.is_empty() {
        return None;
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_z = 0.0;

    for point in points {
        let (x, y, z) = point3d_to_f64(point);
        sum_x += x;
        sum_y += y;
        sum_z += z;
    }

    let count = points.len() as f64;
    Some(point3d_from_f64(
        sum_x / count,
        sum_y / count,
        sum_z / count,
    ))
}
