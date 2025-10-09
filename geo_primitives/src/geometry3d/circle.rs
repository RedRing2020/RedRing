//! 3D Circle implementation
//!
//! 3次元空間における円の具体的な実装

use crate::geometry2d;
use crate::geometry3d::{BBox3D, Direction3D, Point3D, Vector};
use crate::traits::{Circle2D, Circle3D, Direction};
use geo_foundation::abstract_types::Scalar;
use geo_foundation::abstract_types::geometry::common::{CurveAnalysis3D, AnalyticalCurve, CurveType, DifferentialGeometry};

/// 3D空間上の円を表現する構造体
/// 円は指定された平面上に存在する
#[derive(Debug, Clone, PartialEq)]
pub struct Circle<T: Scalar> {
    center: Point3D<T>,
    radius: T,
    normal: Direction3D<T>, // 円が存在する平面の法線ベクトル
    u_axis: Direction3D<T>, // 円の局所X軸（0度方向）
    v_axis: Direction3D<T>, // 円の局所Y軸（90度方向）
}

impl<T: Scalar> Circle<T> {
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
    pub fn new(
        center: Point3D<T>,
        radius: T,
        normal: Direction3D<T>,
        u_axis: Direction3D<T>,
    ) -> Self {
        assert!(radius >= T::ZERO, "半径は非負の値である必要があります");

        // 法線とu_axisが垂直であることを確認
        let dot = normal.x() * u_axis.x() + normal.y() * u_axis.y() + normal.z() * u_axis.z();
        assert!(
            dot.abs() < T::TOLERANCE,
            "法線ベクトルとu_axisは垂直である必要があります"
        );

        // v_axisを計算（右手座標系）
        let cross_product = Vector::new(
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
    pub fn xy_plane_circle(center: Point3D<T>, radius: T) -> Self {
        let normal = Direction3D::positive_z();
        let u_axis = Direction3D::positive_x();
        Self::new(center, radius, normal, u_axis)
    }

    /// XZ平面上の円を作成
    pub fn xz_plane_circle(center: Point3D<T>, radius: T) -> Self {
        let normal = Direction3D::positive_y();
        let u_axis = Direction3D::positive_x();
        Self::new(center, radius, normal, u_axis)
    }

    /// YZ平面上の円を作成
    pub fn yz_plane_circle(center: Point3D<T>, radius: T) -> Self {
        let normal = Direction3D::positive_x();
        let u_axis = Direction3D::positive_y();
        Self::new(center, radius, normal, u_axis)
    }

    /// 単位円（半径1、原点中心、XY平面）を作成
    pub fn unit_circle() -> Self {
        Self::xy_plane_circle(Point3D::origin(), T::ONE)
    }

    /// 円が退化しているか（半径が0）を判定
    pub fn is_degenerate(&self) -> bool {
        self.radius <= T::TOLERANCE
    }

    /// 指定された点が円の平面上にあるかを判定
    pub fn point_on_plane(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let to_point = Vector::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let distance = to_point.dot(&self.normal.to_vector()).abs();
        distance <= tolerance
    }

    /// 点を円の平面に投影
    pub fn project_point_to_plane(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = Vector::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let normal_vec = self.normal.to_vector();
        let distance_to_plane = to_point.dot(&normal_vec);

        Point3D::new(
            point.x() - distance_to_plane * self.normal.x(),
            point.y() - distance_to_plane * self.normal.y(),
            point.z() - distance_to_plane * self.normal.z(),
        )
    }

    /// 指定された点までの最短距離を取得
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let projected = self.project_point_to_plane(point);
        let center_distance = self.center.distance_to(&projected);
        let circle_distance = center_distance - self.radius;

        // 平面からの距離も考慮
        let to_point = Vector::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );
        let normal_vec = Vector::new(self.normal.x(), self.normal.y(), self.normal.z());
        let plane_distance = to_point.dot(&normal_vec).abs();

        (circle_distance * circle_distance + plane_distance * plane_distance).sqrt()
    }

    /// 円を指定倍率で拡大縮小
    pub fn scale(&self, factor: T) -> Self {
        assert!(
            factor >= T::ZERO,
            "拡大縮小係数は非負の値である必要があります"
        );
        Self::new(self.center, self.radius * factor, self.normal, self.u_axis)
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        let new_center = Point3D::new(
            self.center.x() + vector.x(),
            self.center.y() + vector.y(),
            self.center.z() + vector.z(),
        );
        Self::new(new_center, self.radius, self.normal, self.u_axis)
    }
}

// Circle3Dトレイトの実装はf64特化版のみ
impl Circle3D for Circle<f64> {
    type Point = Point3D<f64>;
    type Vector = Vector<f64>;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> f64 {
        self.radius
    }

    fn normal(&self) -> Self::Vector {
        Vector::new(self.normal.x(), self.normal.y(), self.normal.z())
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        // まず点が円の平面上にあるかチェック
        if !self.point_on_plane(point, 1e-10) {
            return false;
        }

        // 平面に投影した点が円内にあるかチェック
        let projected = self.project_point_to_plane(point);
        let distance_squared = (projected.x() - self.center.x()).powi(2)
            + (projected.y() - self.center.y()).powi(2)
            + (projected.z() - self.center.z()).powi(2);
        distance_squared <= self.radius.powi(2) + 1e-10
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
        if !self.on_circumference(point, 1e-10) {
            return None;
        }

        // 円の平面内での接線方向を計算
        let projected = self.project_point_to_plane(point);
        let radial = Vector::new(
            projected.x() - self.center.x(),
            projected.y() - self.center.y(),
            projected.z() - self.center.z(),
        );

        // 接線ベクトルは法線と半径ベクトルの外積
        let normal_vec = Vector::new(self.normal.x(), self.normal.y(), self.normal.z());
        Some(normal_vec.cross(&radial))
    }

    fn tangent_at_angle(&self, angle: f64) -> Self::Vector {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 接線ベクトルは (-sin, cos) 方向
        let local_tangent_x = -self.radius * sin_angle;
        let local_tangent_y = self.radius * cos_angle;

        Vector::new(
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
            crate::geometry2d::Point::new(self.center.x(), self.center.y()),
            self.radius,
        )
    }
}

impl<T: Scalar> Circle<T> {
    /// 指定された点が円周上にあるかを判定（許容誤差内）
    pub fn on_circumference(&self, point: &Point3D<T>, tolerance: T) -> bool {
        if !self.point_on_plane(point, tolerance) {
            return false;
        }

        let projected = self.project_point_to_plane(point);
        let distance = self.center.distance_to(&projected);
        (distance - self.radius).abs() <= tolerance
    }

    /// 境界上の点を含む点の包含判定（許容誤差付き）
    pub fn contains_point_on_boundary(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.on_circumference(point, tolerance)
    }

    /// 他の3D円との交差判定（簡略化実装）
    pub fn intersects_with_circle(&self, other: &Circle<T>) -> bool {
        // 同一平面かつ交差する場合のみtrue（簡略化）
        let distance = self.center.distance_to(&other.center);
        let sum_radii = self.radius + other.radius;
        let diff_radii = (self.radius - other.radius).abs();

        distance <= sum_radii && distance >= diff_radii
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 法線方向を取得
    pub fn normal(&self) -> Vector<T> {
        Vector::new(self.normal.x(), self.normal.y(), self.normal.z())
    }

    /// U軸（局所X軸）を取得
    pub fn u_axis(&self) -> Direction3D<T> {
        self.u_axis
    }

    /// V軸（局所Y軸）を取得
    pub fn v_axis(&self) -> Direction3D<T> {
        self.v_axis
    }
}

impl Circle<f64> {
    /// 3点から円を作成（f64特化、簡略化実装）
    pub fn from_three_points(p1: Point3D<f64>, p2: Point3D<f64>, p3: Point3D<f64>) -> Option<Self> {
        // 簡略化：XY平面上の円として作成
        let p1_2d = crate::geometry2d::Point::new(p1.x(), p1.y());
        let p2_2d = crate::geometry2d::Point::new(p2.x(), p2.y());
        let p3_2d = crate::geometry2d::Point::new(p3.x(), p3.y());

        let circle_2d = crate::geometry2d::Circle::from_three_points(p1_2d, p2_2d, p3_2d)?;
        let center_3d = Point3D::new(circle_2d.center().x(), circle_2d.center().y(), p1.z());

        Some(Self::xy_plane_circle(center_3d, circle_2d.radius()))
    }
}

impl From<Circle<f64>> for BBox3D<f64> {
    fn from(circle: Circle<f64>) -> Self {
        let (min, max) = circle.bounding_box();
        BBox3D::new_from_tuples((min.x(), min.y(), min.z()), (max.x(), max.y(), max.z()))
    }
}

/// f64特化版Circle3Dエイリアス
pub type Circle3DF64 = Circle<f64>;

/// f32特化版Circle3Dエイリアス
pub type Circle3DF32 = Circle<f32>;

// =============================================================================
// 統一曲線解析インターフェイスの実装
// =============================================================================

/// Circle<T>に統一曲線解析インターフェイスを実装
impl<T: Scalar> CurveAnalysis3D<T> for Circle<T> {
    type Point = Point3D<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>;

    /// 指定されたパラメータ位置での点を取得
    /// t: 0.0〜1.0 で一周（0.0=開始点、1.0=終了点=開始点）
    fn point_at_parameter(&self, t: T) -> Self::Point {
        let angle = t * T::TAU; // 0.0〜1.0 を 0〜2π に変換
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        // 局所座標系での点を計算
        let local_x = self.radius * cos_angle;
        let local_y = self.radius * sin_angle;
        
        // ワールド座標系に変換
        let world_offset = self.u_axis.to_vector() * local_x + self.v_axis.to_vector() * local_y;
        self.center + world_offset
    }

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        let angle = t * T::TAU;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        // 局所座標系での接線ベクトル（反時計回り）
        let local_tangent_x = -sin_angle;
        let local_tangent_y = cos_angle;
        
        // ワールド座標系に変換して正規化
        let tangent = self.u_axis.to_vector() * local_tangent_x + self.v_axis.to_vector() * local_tangent_y;
        tangent.normalize().unwrap_or(tangent) // normalizeに失敗した場合は元のベクトルを返す
    }

    /// 指定されたパラメータ位置での主法線ベクトルを取得（正規化済み）
    fn normal_at_parameter(&self, t: T) -> Self::Vector {
        let angle = t * T::TAU;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        // 局所座標系での法線ベクトル（中心向き）
        let local_normal_x = cos_angle;
        let local_normal_y = sin_angle;
        
        // ワールド座標系に変換（既に正規化済み）
        self.u_axis.to_vector() * local_normal_x + self.v_axis.to_vector() * local_normal_y
    }

    /// 指定されたパラメータ位置での双法線ベクトルを取得（正規化済み）
    fn binormal_at_parameter(&self, _t: T) -> Self::Vector {
        // 円の双法線は常に平面の法線ベクトル
        self.normal.to_vector()
    }

    /// 指定されたパラメータ位置での曲率を取得
    fn curvature_at_parameter(&self, _t: T) -> T {
        // 円の曲率は一定: κ = 1/半径
        T::ONE / self.radius
    }

    /// 指定されたパラメータ位置での捩率（ねじれ）を取得
    fn torsion_at_parameter(&self, _t: T) -> T {
        // 平面曲線（円）の捩率は常にゼロ
        T::ZERO
    }

    /// 指定されたパラメータ位置での微分幾何学的情報を一括取得（最も効率的）
    fn differential_geometry_at_parameter(&self, t: T) -> DifferentialGeometry<T, Self::Vector> {
        let angle = t * T::TAU;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        // 局所座標系で一括計算
        let local_tangent_x = -sin_angle;
        let local_tangent_y = cos_angle;
        let local_normal_x = cos_angle;
        let local_normal_y = sin_angle;
        
        // ワールド座標系に変換
        let tangent_vec = self.u_axis.to_vector() * local_tangent_x + self.v_axis.to_vector() * local_tangent_y;
        let tangent = tangent_vec.normalize().unwrap_or(tangent_vec);
        let normal = self.u_axis.to_vector() * local_normal_x + self.v_axis.to_vector() * local_normal_y;
        let curvature = T::ONE / self.radius;
        
        DifferentialGeometry::new(tangent, normal, curvature)
    }

    /// 最大曲率の位置と値を取得（円は一定曲率）
    fn max_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ONE / self.radius)) // 任意の位置で一定曲率
    }

    /// 最小曲率の位置と値を取得（円は一定曲率）
    fn min_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ONE / self.radius)) // 任意の位置で一定曲率
    }

    /// 曲率がゼロになる位置を取得（円では存在しない）
    fn inflection_points(&self) -> Vec<T> {
        Vec::new() // 円に変曲点は存在しない
    }

    /// 曲線が平面曲線かどうかを判定（円は常に平面曲線）
    fn is_planar(&self) -> bool {
        true
    }
}

/// Circle<T>に解析的曲線インターフェイスを実装
impl<T: Scalar> AnalyticalCurve<T> for Circle<T> {
    /// 曲線の種類（円）
    fn curve_type(&self) -> CurveType {
        CurveType::Circle
    }

    /// 一定曲率かどうか（円は常に一定曲率）
    fn has_constant_curvature(&self) -> bool {
        true
    }

    /// 解析的に計算可能な曲率の定数値（円の場合: 1/半径）
    fn constant_curvature(&self) -> Option<T> {
        Some(T::ONE / self.radius)
    }

    /// 解析的に計算可能な曲率半径の定数値（円の場合: 半径）
    fn constant_curvature_radius(&self) -> Option<T> {
        Some(self.radius)
    }
}
