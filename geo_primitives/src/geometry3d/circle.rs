//! 3D Circle implementation
//!
//! 3次元空間における円の具体的な実装

use crate::geometry2d;
use crate::geometry3d::{BBox3D, Direction3D, Point3D, Vector3D};
use crate::traits::{Circle2D, Circle3D, Direction};
use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;

/// 3D空間上の円を表現する構造体
/// 円は指定された平面上に存在する
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point3D,
    radius: f64,
    normal: Direction3D, // 円が存在する平面の法線ベクトル
    u_axis: Direction3D, // 円の局所X軸（0度方向）
    v_axis: Direction3D, // 円の局所Y軸（90度方向）
}

impl Circle {
    /// 新しい3D円を作成
    ///
    /// # Arguments
    /// * `center` - 円の中心点
    /// * `radius` - 円の半径（正の値）
    /// * `normal` - 円が存在する平面の法線ベクトル
    /// * `u_axis` - 円の局所座標系のX軸方向（0度方向）
    ///
    /// # Panics
    /// 半径が負の値またはNaNの場合、または法線とu_axisが垂直でない場合にパニックする
    pub fn new(center: Point3D, radius: f64, normal: Direction3D, u_axis: Direction3D) -> Self {
        assert!(
            radius >= 0.0 && radius.is_finite(),
            "半径は非負の有限値である必要があります"
        );

        // 法線とu_axisが垂直であることを確認
        let dot = normal.x() * u_axis.x() + normal.y() * u_axis.y() + normal.z() * u_axis.z();
        assert!(
            dot.abs() < GEOMETRIC_TOLERANCE,
            "法線ベクトルとu_axisは垂直である必要があります"
        );

        // v_axisを計算（右手座標系）
        let cross_product = Vector3D::new(
            normal.y() * u_axis.z() - normal.z() * u_axis.y(),
            normal.z() * u_axis.x() - normal.x() * u_axis.z(),
            normal.x() * u_axis.y() - normal.y() * u_axis.x(),
        );
        let v_axis = Direction3D::from_vector(cross_product).expect("v_axisの計算に失敗しました");

        Self {
            center,
            radius,
            normal,
            u_axis,
            v_axis,
        }
    }

    /// XY平面上の円を作成
    ///
    /// # Arguments
    /// * `center` - 円の中心点
    /// * `radius` - 円の半径
    pub fn xy_plane_circle(center: Point3D, radius: f64) -> Self {
        let normal = Direction3D::positive_z();
        let u_axis = Direction3D::positive_x();
        Self::new(center, radius, normal, u_axis)
    }

    /// XZ平面上の円を作成
    pub fn xz_plane_circle(center: Point3D, radius: f64) -> Self {
        let normal = Direction3D::positive_y();
        let u_axis = Direction3D::positive_x();
        Self::new(center, radius, normal, u_axis)
    }

    /// YZ平面上の円を作成
    pub fn yz_plane_circle(center: Point3D, radius: f64) -> Self {
        let normal = Direction3D::positive_x();
        let u_axis = Direction3D::positive_y();
        Self::new(center, radius, normal, u_axis)
    }

    /// 単位円（半径1、原点中心、XY平面）を作成
    pub fn unit_circle() -> Self {
        Self::xy_plane_circle(Point3D::origin(), 1.0)
    }

    /// 円が退化しているか（半径が0）を判定
    pub fn is_degenerate(&self) -> bool {
        self.radius < GEOMETRIC_TOLERANCE
    }

    /// 指定された点が円の平面上にあるかを判定
    pub fn point_on_plane(&self, point: &Point3D, tolerance: f64) -> bool {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let distance = to_point
            .dot(&Vector3D::new(
                self.normal.x(),
                self.normal.y(),
                self.normal.z(),
            ))
            .abs();
        distance <= tolerance
    }

    /// 点を円の平面に投影
    pub fn project_point_to_plane(&self, point: &Point3D) -> Point3D {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let normal_vec = Vector3D::new(self.normal.x(), self.normal.y(), self.normal.z());
        let distance_to_plane = to_point.dot(&normal_vec);

        Point3D::new(
            point.x() - distance_to_plane * self.normal.x(),
            point.y() - distance_to_plane * self.normal.y(),
            point.z() - distance_to_plane * self.normal.z(),
        )
    }

    /// 指定された点までの最短距離を取得
    pub fn distance_to_point(&self, point: &Point3D) -> f64 {
        let projected = self.project_point_to_plane(point);
        let center_distance = self.center.distance_to(&projected);
        let circle_distance = center_distance - self.radius;

        // 平面からの距離も考慮
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );
        let normal_vec = Vector3D::new(self.normal.x(), self.normal.y(), self.normal.z());
        let plane_distance = to_point.dot(&normal_vec).abs();

        (circle_distance.powi(2) + plane_distance.powi(2)).sqrt()
    }

    /// 円を指定倍率で拡大縮小
    pub fn scale(&self, factor: f64) -> Self {
        assert!(
            factor >= 0.0 && factor.is_finite(),
            "拡大縮小係数は非負の有限値である必要があります"
        );
        Self::new(self.center, self.radius * factor, self.normal, self.u_axis)
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector3D) -> Self {
        let new_center = Point3D::new(
            self.center.x() + vector.x(),
            self.center.y() + vector.y(),
            self.center.z() + vector.z(),
        );
        Self::new(new_center, self.radius, self.normal, self.u_axis)
    }
}

impl Circle3D for Circle {
    type Point = Point3D;
    type Vector = Vector3D;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> f64 {
        self.radius
    }

    fn normal(&self) -> Self::Vector {
        Vector3D::new(self.normal.x(), self.normal.y(), self.normal.z())
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        // まず点が円の平面上にあるかチェック
        if !self.point_on_plane(point, GEOMETRIC_TOLERANCE) {
            return false;
        }

        // 平面に投影した点が円内にあるかチェック
        let projected = self.project_point_to_plane(point);
        let distance_squared = (projected.x() - self.center.x()).powi(2)
            + (projected.y() - self.center.y()).powi(2)
            + (projected.z() - self.center.z()).powi(2);
        distance_squared <= self.radius.powi(2) + GEOMETRIC_TOLERANCE
    }

    fn on_circumference(&self, point: &Self::Point, tolerance: f64) -> bool {
        if !self.point_on_plane(point, tolerance) {
            return false;
        }

        let projected = self.project_point_to_plane(point);
        let distance = self.center.distance_to(&projected);
        (distance - self.radius).abs() <= tolerance
    }

    fn point_at_angle(&self, angle: f64) -> Self::Point {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        let local_x = self.radius * cos_angle;
        let local_y = self.radius * sin_angle;

        Point3D::new(
            self.center.x() + local_x * self.u_axis.x() + local_y * self.v_axis.x(),
            self.center.y() + local_x * self.u_axis.y() + local_y * self.v_axis.y(),
            self.center.z() + local_x * self.u_axis.z() + local_y * self.v_axis.z(),
        )
    }

    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector> {
        if !self.on_circumference(point, GEOMETRIC_TOLERANCE) {
            return None;
        }

        // 円の平面内での接線方向を計算
        let projected = self.project_point_to_plane(point);
        let radial = Vector3D::new(
            projected.x() - self.center.x(),
            projected.y() - self.center.y(),
            projected.z() - self.center.z(),
        );

        // 接線ベクトルは法線と半径ベクトルの外積
        let normal_vec = Vector3D::new(self.normal.x(), self.normal.y(), self.normal.z());
        Some(normal_vec.cross(&radial))
    }

    fn tangent_at_angle(&self, angle: f64) -> Self::Vector {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 接線ベクトルは (-sin, cos) 方向
        let local_tangent_x = -self.radius * sin_angle;
        let local_tangent_y = self.radius * cos_angle;

        Vector3D::new(
            local_tangent_x * self.u_axis.x() + local_tangent_y * self.v_axis.x(),
            local_tangent_x * self.u_axis.y() + local_tangent_y * self.v_axis.y(),
            local_tangent_x * self.u_axis.z() + local_tangent_y * self.v_axis.z(),
        )
    }

    fn bounding_box(&self) -> (Self::Point, Self::Point) {
        // 3D空間での軸平行境界ボックスを計算
        // 各軸方向での円の最大・最小座標を求める

        let mut min_x = self.center.x();
        let mut max_x = self.center.x();
        let mut min_y = self.center.y();
        let mut max_y = self.center.y();
        let mut min_z = self.center.z();
        let mut max_z = self.center.z();

        // u_axis, v_axis方向の寄与を計算
        let u_extent_x = self.radius * self.u_axis.x().abs();
        let u_extent_y = self.radius * self.u_axis.y().abs();
        let u_extent_z = self.radius * self.u_axis.z().abs();

        let v_extent_x = self.radius * self.v_axis.x().abs();
        let v_extent_y = self.radius * self.v_axis.y().abs();
        let v_extent_z = self.radius * self.v_axis.z().abs();

        let extent_x = u_extent_x + v_extent_x;
        let extent_y = u_extent_y + v_extent_y;
        let extent_z = u_extent_z + v_extent_z;

        min_x -= extent_x;
        max_x += extent_x;
        min_y -= extent_y;
        max_y += extent_y;
        min_z -= extent_z;
        max_z += extent_z;

        (
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    fn to_2d(&self) -> impl Circle2D {
        // XY平面への投影として2D円を返す
        geometry2d::circle::Circle::new(
            crate::geometry2d::point::Point::new(self.center.x(), self.center.y()),
            self.radius,
        )
    }
}

impl Circle {
    /// 境界上の点を含む点の包含判定（許容誤差付き）
    pub fn contains_point_on_boundary(&self, point: &Point3D, tolerance: f64) -> bool {
        self.on_circumference(point, tolerance)
    }

    /// 他の3D円との交差判定（簡略化実装）
    pub fn intersects_with_circle(&self, other: &Circle) -> bool {
        // 同一平面かつ交差する場合のみtrue（簡略化）
        let distance = self.center.distance_to(&other.center);
        let sum_radii = self.radius + other.radius;
        let diff_radii = (self.radius - other.radius).abs();

        distance <= sum_radii && distance >= diff_radii
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// 法線方向を取得
    pub fn normal(&self) -> Vector3D {
        Vector3D::new(self.normal.x(), self.normal.y(), self.normal.z())
    }

    /// U軸（局所X軸）を取得
    pub fn u_axis(&self) -> Direction3D {
        self.u_axis
    }

    /// V軸（局所Y軸）を取得
    pub fn v_axis(&self) -> Direction3D {
        self.v_axis
    }

    /// 3点から円を作成（簡略化実装）
    pub fn from_three_points(p1: Point3D, p2: Point3D, p3: Point3D) -> Option<Self> {
        // 簡略化：XY平面上の円として作成
        let p1_2d = crate::geometry2d::Point::new(p1.x(), p1.y());
        let p2_2d = crate::geometry2d::Point::new(p2.x(), p2.y());
        let p3_2d = crate::geometry2d::Point::new(p3.x(), p3.y());

        let circle_2d = crate::geometry2d::Circle::from_three_points(p1_2d, p2_2d, p3_2d)?;
        let center_3d = Point3D::new(circle_2d.center().x(), circle_2d.center().y(), p1.z());

        Some(Self::xy_plane_circle(center_3d, circle_2d.radius()))
    }
}

impl From<Circle> for BBox3D {
    fn from(circle: Circle) -> Self {
        let (min, max) = circle.bounding_box();
        BBox3D::new((min.x(), min.y(), min.z()), (max.x(), max.y(), max.z()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_xy_plane_circle() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let circle = Circle::xy_plane_circle(center, 5.0);

        assert_eq!(circle.center(), center);
        assert_eq!(circle.radius(), 5.0);
        assert_eq!(circle.area(), PI * 25.0);
        assert_eq!(circle.circumference(), TAU * 5.0);

        // 法線がZ軸方向であることを確認
        let normal = circle.normal();
        assert!((normal.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((normal.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((normal.z() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_unit_circle() {
        let circle = Circle::unit_circle();

        assert_eq!(circle.center(), Point3D::origin());
        assert_eq!(circle.radius(), 1.0);
        assert_eq!(circle.area(), PI);
        assert_eq!(circle.circumference(), TAU);
    }

    #[test]
    fn test_contains_point() {
        let circle = Circle::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 5.0);

        assert!(circle.contains_point(&Point3D::new(0.0, 0.0, 0.0))); // 中心
        assert!(circle.contains_point(&Point3D::new(3.0, 4.0, 0.0))); // 内部
        assert!(circle.contains_point(&Point3D::new(5.0, 0.0, 0.0))); // 円周上
        assert!(!circle.contains_point(&Point3D::new(6.0, 0.0, 0.0))); // 外部
        assert!(!circle.contains_point(&Point3D::new(0.0, 0.0, 1.0))); // 平面外
    }

    #[test]
    fn test_point_at_angle() {
        let circle = Circle::xy_plane_circle(Point3D::new(0.0, 0.0, 0.0), 2.0);

        let point = circle.point_at_angle(0.0);
        assert!((point.x() - 2.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        let point = circle.point_at_angle(PI / 2.0);
        assert!((point.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point.y() - 2.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_bounding_box() {
        let circle = Circle::xy_plane_circle(Point3D::new(1.0, 2.0, 3.0), 4.0);
        let (min, max) = circle.bounding_box();

        assert!((min.x() - (-3.0)).abs() < GEOMETRIC_TOLERANCE);
        assert!((min.y() - (-2.0)).abs() < GEOMETRIC_TOLERANCE);
        assert!((min.z() - 3.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((max.x() - 5.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((max.y() - 6.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((max.z() - 3.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_to_2d() {
        let circle_3d = Circle::xy_plane_circle(Point3D::new(1.0, 2.0, 3.0), 4.0);
        let circle_2d = circle_3d.to_2d();

        // 基本的なプロパティの確認
        assert_eq!(circle_2d.radius(), 4.0);
        assert_eq!(circle_2d.area(), PI * 16.0);
        assert_eq!(circle_2d.circumference(), TAU * 4.0);
    }
}
