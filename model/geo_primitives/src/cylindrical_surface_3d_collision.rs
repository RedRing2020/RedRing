//! CylindricalSurface3D - Collision Implementation
//!
//! 3次元円柱サーフェスの衝突判定実装
//!
//! 円柱サーフェスは無限に延びる円柱面であり、軸からの距離が
//! 半径に等しい点の集合として定義される。

use crate::{
    Circle3D, CylindricalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D,
    Triangle3D, Vector3D,
};
use geo_contracts::Scalar;
use geo_contracts::{AdvancedCollision, BasicCollision};

// ============================================================================
// CylindricalSurface3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to(point);
        distance <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 点から軸への垂直距離を計算
        let center = self.center_internal();
        let axis = self.axis();
        let to_point = Vector3D::from_points(&center, point);

        // 軸方向成分を除去して、軸に垂直な成分のみを取得
        let axis_component = to_point.dot(&axis.as_vector());
        let perpendicular = to_point - axis.as_vector() * axis_component;
        let radial_distance = perpendicular.magnitude();

        // 円柱面までの距離は、軸からの距離と半径の差の絶対値
        (radial_distance - self.radius()).abs()
    }
}

// ============================================================================
// CylindricalSurface3D vs Circle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Circle3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        use geo_contracts::Circle3DProperties;
        // 円の中心点との距離をチェック
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point) <= tolerance
    }

    fn overlaps(&self, circle: &Circle3D<T>, tolerance: T) -> bool {
        self.intersects(circle, tolerance)
    }

    fn distance_to(&self, circle: &Circle3D<T>) -> T {
        use geo_contracts::Circle3DProperties;
        // 簡易実装: 円の中心点との距離を返す
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.distance_to(&center_point)
    }
}

// ============================================================================
// CylindricalSurface3D vs LineSegment3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, LineSegment3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        // 線分の端点との距離をチェック
        self.distance_to(&segment.start()) <= tolerance
            || self.distance_to(&segment.end()) <= tolerance
    }

    fn overlaps(&self, segment: &LineSegment3D<T>, tolerance: T) -> bool {
        self.intersects(segment, tolerance)
    }

    fn distance_to(&self, segment: &LineSegment3D<T>) -> T {
        // 簡易実装: 両端点との距離の最小値
        let dist_start = self.distance_to(&segment.start());
        let dist_end = self.distance_to(&segment.end());
        dist_start.min(dist_end)
    }
}

// ============================================================================
// CylindricalSurface3D vs Ray3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Ray3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        // 光線の始点との距離をチェック
        self.distance_to(&ray.origin_internal()) <= tolerance
    }

    fn overlaps(&self, ray: &Ray3D<T>, tolerance: T) -> bool {
        self.intersects(ray, tolerance)
    }

    fn distance_to(&self, ray: &Ray3D<T>) -> T {
        // 簡易実装: 光線の始点との距離
        self.distance_to(&ray.origin_internal())
    }
}

// ============================================================================
// CylindricalSurface3D vs InfiniteLine3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, InfiniteLine3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        // 直線上の点との距離をチェック
        self.distance_to(&line.point_internal()) <= tolerance
    }

    fn overlaps(&self, line: &InfiniteLine3D<T>, tolerance: T) -> bool {
        self.intersects(line, tolerance)
    }

    fn distance_to(&self, line: &InfiniteLine3D<T>) -> T {
        // 簡易実装: 直線上の基準点との距離
        self.distance_to(&line.point_internal())
    }
}

// ============================================================================
// CylindricalSurface3D vs Triangle3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Triangle3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        use geo_contracts::Triangle3DProperties;

        // 三角形の各頂点との距離をチェック
        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        self.distance_to(&va) <= tolerance
            || self.distance_to(&vb) <= tolerance
            || self.distance_to(&vc) <= tolerance
    }

    fn overlaps(&self, triangle: &Triangle3D<T>, tolerance: T) -> bool {
        self.intersects(triangle, tolerance)
    }

    fn distance_to(&self, triangle: &Triangle3D<T>) -> T {
        use geo_contracts::Triangle3DProperties;

        // 簡易実装: 三角形の頂点との距離の最小値
        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        let dist_a = self.distance_to(&va);
        let dist_b = self.distance_to(&vb);
        let dist_c = self.distance_to(&vc);
        dist_a.min(dist_b).min(dist_c)
    }
}

// ============================================================================
// CylindricalSurface3D vs Plane3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Plane3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        // 円柱の中心点が平面に近いかチェック
        let center = self.center_internal();
        plane.distance_to_point(center).abs() <= tolerance + self.radius()
    }

    fn overlaps(&self, plane: &Plane3D<T>, tolerance: T) -> bool {
        self.intersects(plane, tolerance)
    }

    fn distance_to(&self, plane: &Plane3D<T>) -> T {
        // 簡易実装: 中心点から平面への距離から半径を引いた値
        let center = self.center_internal();
        let dist_to_plane = plane.distance_to_point(center).abs();
        if dist_to_plane > self.radius() {
            dist_to_plane - self.radius()
        } else {
            T::ZERO
        }
    }
}

// ============================================================================
// CylindricalSurface3D vs CylindricalSurface3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, CylindricalSurface3D<T>> for CylindricalSurface3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, other: &CylindricalSurface3D<T>, tolerance: T) -> bool {
        // 2つの円柱サーフェスの軸が平行かどうかで場合分け
        let axis1 = self.axis();
        let axis2 = other.axis();

        // 軸の平行度チェック
        let cross = axis1.as_vector().cross(&axis2.as_vector());
        let is_parallel = cross.magnitude() < tolerance;

        if is_parallel {
            // 軸が平行な場合: 軸間の距離が半径の和以下なら交差
            let center1 = self.center_internal();
            let center2 = other.center_internal();
            let between_centers = Vector3D::from_points(&center1, &center2);

            // 軸方向成分を除去
            let axis_component = between_centers.dot(&axis1.as_vector());
            let perpendicular = between_centers - axis1.as_vector() * axis_component;
            let axis_distance = perpendicular.magnitude();

            let radius_sum = self.radius() + other.radius();
            let radius_diff = (self.radius() - other.radius()).abs();

            axis_distance <= radius_sum + tolerance && axis_distance >= radius_diff - tolerance
        } else {
            // 軸が交差する場合: 簡易実装として、中心点間の距離をチェック
            let center1 = self.center_internal();
            let center2 = other.center_internal();
            let dist = Vector3D::from_points(&center1, &center2).magnitude();
            dist <= (self.radius() + other.radius()) + tolerance
        }
    }

    fn overlaps(&self, other: &CylindricalSurface3D<T>, tolerance: T) -> bool {
        self.intersects(other, tolerance)
    }

    fn distance_to(&self, other: &CylindricalSurface3D<T>) -> T {
        // 簡易実装: 中心点間の距離から両半径の和を引いた値
        let center1 = self.center_internal();
        let center2 = other.center_internal();
        let dist = Vector3D::from_points(&center1, &center2).magnitude();
        let radius_sum = self.radius() + other.radius();
        if dist > radius_sum {
            dist - radius_sum
        } else {
            T::ZERO
        }
    }
}

// ============================================================================
// AdvancedCollision Implementations
// ============================================================================

// CylindricalSurface3D vs Point3D
impl<T: Scalar> AdvancedCollision<T, Point3D<T>> for CylindricalSurface3D<T> {
    type PointPair = (Point3D<T>, Point3D<T>);
    type Vector2D = Vector3D<T>;

    fn closest_points(&self, point: &Point3D<T>) -> Self::PointPair {
        // 点から軸への最近点を計算
        let center = self.center_internal();
        let axis = self.axis();
        let to_point = Vector3D::from_points(&center, point);

        // 軸方向成分を計算
        let axis_component = to_point.dot(&axis.as_vector());
        let projection_on_axis = center + axis.as_vector() * axis_component;

        // 軸に垂直な方向の単位ベクトルを求める
        let radial = Vector3D::from_points(&projection_on_axis, point);
        let radial_length = radial.magnitude();

        let closest_on_surface = if radial_length.is_zero() {
            // 点が軸上にある場合は任意の方向に半径分移動
            projection_on_axis + Vector3D::new(self.radius(), T::ZERO, T::ZERO)
        } else {
            // 軸上の点から半径分だけ点の方向に移動
            projection_on_axis + radial * (self.radius() / radial_length)
        };

        (closest_on_surface, *point)
    }

    fn overlap_measure(&self, _point: &Point3D<T>) -> Option<T> {
        // 点との重なり測定値は定義されない（点は面積を持たない）
        None
    }

    fn separated_by_axis(&self, point: &Point3D<T>, axis: Self::Vector2D) -> bool {
        // 簡易実装: 軸方向の射影を比較
        let center = self.center_internal();
        let self_proj = center.x() * axis.x() + center.y() * axis.y() + center.z() * axis.z();
        let point_proj = point.x() * axis.x() + point.y() * axis.y() + point.z() * axis.z();
        let radius_proj = self.radius() * axis.magnitude();

        (point_proj - self_proj).abs() > radius_proj
    }

    fn containment_relation(&self, point: &Point3D<T>, tolerance: T) -> (bool, bool) {
        // 円柱面は点を包含しない（無限に薄い面）
        // 点も円柱面を包含できない
        let on_surface = self.intersects(point, tolerance);
        (false, !on_surface)
    }
}

// CylindricalSurface3D vs Circle3D
impl<T: Scalar> AdvancedCollision<T, Circle3D<T>> for CylindricalSurface3D<T> {
    type PointPair = (Point3D<T>, Point3D<T>);
    type Vector2D = Vector3D<T>;

    fn closest_points(&self, circle: &Circle3D<T>) -> Self::PointPair {
        use geo_contracts::Circle3DProperties;
        // 簡易実装: 円の中心点に対する最近点を返す
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.closest_points(&center_point)
    }

    fn overlap_measure(&self, _circle: &Circle3D<T>) -> Option<T> {
        // 簡易実装: 重なり測定未サポート
        None
    }

    fn separated_by_axis(&self, circle: &Circle3D<T>, axis: Self::Vector2D) -> bool {
        use geo_contracts::Circle3DProperties;
        let (cx, cy, cz) = circle.center();
        let center_point = Point3D::new(cx, cy, cz);
        self.separated_by_axis(&center_point, axis)
    }

    fn containment_relation(&self, _circle: &Circle3D<T>, _tolerance: T) -> (bool, bool) {
        // 円柱面と円の包含関係は複雑なため簡易実装
        (false, false)
    }
}

// CylindricalSurface3D vs LineSegment3D
impl<T: Scalar> AdvancedCollision<T, LineSegment3D<T>> for CylindricalSurface3D<T> {
    type PointPair = (Point3D<T>, Point3D<T>);
    type Vector2D = Vector3D<T>;

    fn closest_points(&self, segment: &LineSegment3D<T>) -> Self::PointPair {
        // 簡易実装: 始点に対する最近点を返す
        self.closest_points(&segment.start())
    }

    fn overlap_measure(&self, _segment: &LineSegment3D<T>) -> Option<T> {
        None
    }

    fn separated_by_axis(&self, segment: &LineSegment3D<T>, axis: Self::Vector2D) -> bool {
        // 線分の両端点が分離軸で分離されているかチェック
        self.separated_by_axis(&segment.start(), axis)
            && self.separated_by_axis(&segment.end(), axis)
    }

    fn containment_relation(&self, _segment: &LineSegment3D<T>, _tolerance: T) -> (bool, bool) {
        (false, false)
    }
}

// CylindricalSurface3D vs Triangle3D
impl<T: Scalar> AdvancedCollision<T, Triangle3D<T>> for CylindricalSurface3D<T> {
    type PointPair = (Point3D<T>, Point3D<T>);
    type Vector2D = Vector3D<T>;

    fn closest_points(&self, triangle: &Triangle3D<T>) -> Self::PointPair {
        use geo_contracts::Triangle3DProperties;
        // 簡易実装: 頂点Aに対する最近点を返す
        let (ax, ay, az) = triangle.vertex_a();
        let va = Point3D::new(ax, ay, az);
        self.closest_points(&va)
    }

    fn overlap_measure(&self, _triangle: &Triangle3D<T>) -> Option<T> {
        None
    }

    fn separated_by_axis(&self, triangle: &Triangle3D<T>, axis: Self::Vector2D) -> bool {
        use geo_contracts::Triangle3DProperties;
        // 三角形の全頂点が分離軸で分離されているかチェック
        let (ax, ay, az) = triangle.vertex_a();
        let (bx, by, bz) = triangle.vertex_b();
        let (cx, cy, cz) = triangle.vertex_c();
        let va = Point3D::new(ax, ay, az);
        let vb = Point3D::new(bx, by, bz);
        let vc = Point3D::new(cx, cy, cz);

        self.separated_by_axis(&va, axis)
            && self.separated_by_axis(&vb, axis)
            && self.separated_by_axis(&vc, axis)
    }

    fn containment_relation(&self, _triangle: &Triangle3D<T>, _tolerance: T) -> (bool, bool) {
        (false, false)
    }
}

// CylindricalSurface3D vs Plane3D
impl<T: Scalar> AdvancedCollision<T, Plane3D<T>> for CylindricalSurface3D<T> {
    type PointPair = (Point3D<T>, Point3D<T>);
    type Vector2D = Vector3D<T>;

    fn closest_points(&self, plane: &Plane3D<T>) -> Self::PointPair {
        // 円柱の中心点から平面への最近点を計算
        let center = self.center_internal();
        let normal = plane.normal();
        let point_on_plane = plane.point();

        // 中心点から平面への垂直距離
        let to_center = Vector3D::from_points(&point_on_plane, &center);
        let dist = to_center.dot(&normal.as_vector());
        let closest_on_plane = center - normal.as_vector() * dist;

        // 円柱面上の最近点を簡易計算
        let (surf_pt, _) = self.closest_points(&closest_on_plane);

        (surf_pt, closest_on_plane)
    }

    fn overlap_measure(&self, _plane: &Plane3D<T>) -> Option<T> {
        None
    }

    fn separated_by_axis(&self, _plane: &Plane3D<T>, _axis: Self::Vector2D) -> bool {
        // 平面との分離判定は複雑なため簡易実装
        false
    }

    fn containment_relation(&self, _plane: &Plane3D<T>, _tolerance: T) -> (bool, bool) {
        // 無限平面と無限円柱面はどちらも包含関係を持たない
        (false, false)
    }
}

// CylindricalSurface3D vs CylindricalSurface3D
impl<T: Scalar> AdvancedCollision<T, CylindricalSurface3D<T>> for CylindricalSurface3D<T> {
    type PointPair = (Point3D<T>, Point3D<T>);
    type Vector2D = Vector3D<T>;

    fn closest_points(&self, other: &CylindricalSurface3D<T>) -> Self::PointPair {
        // 簡易実装: 両方の中心点から最近点を計算
        let center1 = self.center_internal();
        let center2 = other.center_internal();

        let (pt1, _) = self.closest_points(&center2);
        let (pt2, _) = other.closest_points(&center1);

        (pt1, pt2)
    }

    fn overlap_measure(&self, _other: &CylindricalSurface3D<T>) -> Option<T> {
        // 2つの円柱面の重なり測定は複雑なため未実装
        None
    }

    fn separated_by_axis(&self, other: &CylindricalSurface3D<T>, axis: Self::Vector2D) -> bool {
        // 両方の中心点の射影を比較
        let center1 = self.center_internal();
        let center2 = other.center_internal();

        let proj1 = center1.x() * axis.x() + center1.y() * axis.y() + center1.z() * axis.z();
        let proj2 = center2.x() * axis.x() + center2.y() * axis.y() + center2.z() * axis.z();

        let radius_proj1 = self.radius() * axis.magnitude();
        let radius_proj2 = other.radius() * axis.magnitude();

        (proj2 - proj1).abs() > (radius_proj1 + radius_proj2)
    }

    fn containment_relation(&self, other: &CylindricalSurface3D<T>, tolerance: T) -> (bool, bool) {
        // 円柱面同士の包含関係: 軸が一致し、一方の半径が他方より大きい場合のみ
        let axis1 = self.axis();
        let axis2 = other.axis();

        // 軸の平行度チェック
        let cross = axis1.as_vector().cross(&axis2.as_vector());
        let is_parallel = cross.magnitude() < tolerance;

        if !is_parallel {
            return (false, false);
        }

        let r1 = self.radius();
        let r2 = other.radius();

        // 半径の比較
        let self_contains = r1 > r2 + tolerance;
        let other_contains = r2 > r1 + tolerance;

        (self_contains, other_contains)
    }
}
