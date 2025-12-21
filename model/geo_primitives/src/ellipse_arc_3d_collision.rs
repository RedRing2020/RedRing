//! EllipseArc3D Collision 実装
//!
//! BasicCollision トレイトの実装
//! 委譲パターンで Ellipse3D の実装を再利用

use crate::{
    Arc3D, Circle3D, Ellipse3D, EllipseArc3D, LineSegment3D, Plane3D, Point3D, Triangle3D,
};
use geo_foundation::{
    core::{arc_core_traits::Arc3DProperties, triangle_core_traits::Triangle3DProperties},
    extensions::BasicCollision,
    Circle3DProperties, Scalar,
};

// ============================================================================
// EllipseArc3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // 1. 基底楕円との判定に委譲
        if !self.ellipse().intersects(point, tolerance) {
            return false;
        }
        // 2. 角度範囲内かチェック
        self.point_in_angle_range(point, tolerance)
    }

    fn overlaps(&self, _point: &Point3D<T>, _tolerance: T) -> bool {
        false // 点は重なりを持たない
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // distance_to_point メソッドを使用
        self.distance_to_point(point)
    }
}

// ============================================================================
// EllipseArc3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        // 1. 基底楕円との判定に委譲
        if !self.ellipse().intersects(circle, tolerance) {
            return false;
        }

        // 2. 円の中心が楕円弧の角度範囲に近い場合は交差の可能性あり
        // 簡易実装: 楕円との交差があれば、楕円弧とも交差する可能性
        true
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        // 簡易実装: 楕円の overlap に委譲
        self.ellipse().overlaps(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        // 円の中心点への距離から半径を引く
        let (cx, cy, cz) = circle.center();
        let center = Point3D::new(cx, cy, cz);
        let dist_to_center = self.distance_to_point(&center);
        (dist_to_center - circle.radius()).max(T::ZERO)
    }
}

// ============================================================================
// EllipseArc3D vs Arc3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Arc3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, arc: &Arc3D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円と円弧の基底円の判定
        let (cx, cy, cz) = <Arc3D<T> as Arc3DProperties<T>>::center(arc);
        let center = Point3D::new(cx, cy, cz);

        let dist = self.distance_to_point(&center);
        dist <= arc.radius() + tolerance
    }

    fn overlaps(&self, _arc: &Arc3D<T>, _tolerance: T) -> bool {
        false // 簡易実装: 重なりなし
    }

    fn distance_to(&self, arc: &Arc3D<T>) -> T {
        // 円弧の中心点への距離から半径を引く
        let (cx, cy, cz) = <Arc3D<T> as Arc3DProperties<T>>::center(arc);
        let center = Point3D::new(cx, cy, cz);
        let dist_to_center = self.distance_to_point(&center);
        (dist_to_center - arc.radius()).max(T::ZERO)
    }
}

// ============================================================================
// EllipseArc3D vs Ellipse3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ellipse3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ellipse: &Ellipse3D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円同士の判定に委譲
        self.ellipse().intersects(ellipse, tolerance)
    }

    fn overlaps(&self, ellipse: &Ellipse3D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円の overlap に委譲
        self.ellipse().overlaps(ellipse, tolerance)
    }

    fn distance_to(&self, ellipse: &Ellipse3D<T>) -> T {
        // 楕円中心への距離を使用
        let center = ellipse.center();
        self.distance_to_point(&center)
    }
}

// ============================================================================
// EllipseArc3D vs EllipseArc3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, EllipseArc3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &EllipseArc3D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円同士の判定
        self.ellipse().intersects(other.ellipse(), tolerance)
    }

    fn overlaps(&self, other: &EllipseArc3D<T>, tolerance: T) -> bool {
        // 簡易実装: 基底楕円の overlap
        self.ellipse().overlaps(other.ellipse(), tolerance)
    }

    fn distance_to(&self, other: &EllipseArc3D<T>) -> T {
        // 他方の中心への距離
        let other_center = other.center();
        self.distance_to_point(&other_center)
    }
}

// ============================================================================
// EllipseArc3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の端点が楕円弧と交差するか判定
        let start = line.start();
        let end = line.end();

        if self.intersects(&start, tolerance) || self.intersects(&end, tolerance) {
            return true;
        }

        // 簡易実装: 線分と基底楕円の交差判定
        self.ellipse().intersects(line, tolerance)
    }

    fn overlaps(&self, _line: &LineSegment3D<T>, _tolerance: T) -> bool {
        false // 線分との重なりはなし
    }

    fn distance_to(&self, line: &LineSegment3D<T>) -> T {
        // 線分の端点との最小距離
        let start = line.start();
        let end = line.end();

        let dist_to_start = self.distance_to_point(&start);
        let dist_to_end = self.distance_to_point(&end);

        dist_to_start.min(dist_to_end)
    }
}

// ============================================================================
// EllipseArc3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        // 三角形の頂点が楕円弧と交差するか判定
        let (ax, ay, az) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_a(triangle);
        let (bx, by, bz) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_b(triangle);
        let (cx, cy, cz) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_c(triangle);

        let vertex_a = Point3D::new(ax, ay, az);
        let vertex_b = Point3D::new(bx, by, bz);
        let vertex_c = Point3D::new(cx, cy, cz);

        if BasicCollision::<T, Point3D<T>>::intersects(self, &vertex_a, tolerance)
            || BasicCollision::<T, Point3D<T>>::intersects(self, &vertex_b, tolerance)
            || BasicCollision::<T, Point3D<T>>::intersects(self, &vertex_c, tolerance)
        {
            return true;
        }

        // 簡易実装: 三角形と基底楕円の交差判定
        self.ellipse().intersects(triangle, tolerance)
    }

    fn overlaps(&self, _triangle: &Triangle3D<T>, _tolerance: T) -> bool {
        false // 三角形との重なりはなし（簡易実装）
    }

    fn distance_to(&self, triangle: &Triangle3D<T>) -> T {
        // 三角形の頂点との最小距離
        let (ax, ay, az) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_a(triangle);
        let (bx, by, bz) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_b(triangle);
        let (cx, cy, cz) = <Triangle3D<T> as Triangle3DProperties<T>>::vertex_c(triangle);

        let vertex_a = Point3D::new(ax, ay, az);
        let vertex_b = Point3D::new(bx, by, bz);
        let vertex_c = Point3D::new(cx, cy, cz);

        let dist_a = self.distance_to_point(&vertex_a);
        let dist_b = self.distance_to_point(&vertex_b);
        let dist_c = self.distance_to_point(&vertex_c);

        dist_a.min(dist_b).min(dist_c)
    }
}

// ============================================================================
// EllipseArc3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for EllipseArc3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        // 楕円弧の中心と平面の距離を計算
        let center = self.center();
        let distance = plane.distance_to_point(center);

        // 平面との距離が許容範囲内なら交差
        if distance <= tolerance {
            return true;
        }

        // 楕円弧の端点をチェック
        let start = self.start_point();
        let end = self.end_point();

        plane.distance_to_point(start) <= tolerance || plane.distance_to_point(end) <= tolerance
    }

    fn overlaps(&self, _plane: &Plane3D<T>, _tolerance: T) -> bool {
        false // 平面との重なりは特殊ケース（今回は未実装）
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        // 中心点から平面への距離
        let center = self.center();
        plane.distance_to_point(center)
    }
}
