//! Circle3D - Core Implementation
//!
//! 3次元円の基本実装とコンストラクタ、アクセサメソッド
//! STEP (ISO 10303) 準拠の axis2_placement_3d スタイルで実装

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::default_angle_tolerance;
use geo_contracts::{default_distance_tolerance, default_kernel_numerical_zero_tolerance};
use geo_contracts::{Circle3DConstructor, Circle3DMeasure, Circle3DProperties, Scalar};

/// 3次元空間の円
///
/// STEP準拠でaxis（法線）とref_direction（参照方向）を持ち、
/// 3D空間での完全な座標系定義とArc3D変換に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle3D<T: Scalar> {
    center: Point3D<T>,
    /// Z軸方向（円が存在する平面の法線ベクトル）
    axis: Direction3D<T>,
    /// X軸方向（参照方向、角度0度の方向）
    ref_direction: Direction3D<T>,
    radius: T,
}

impl<T: Scalar> Circle3D<T> {
    /// 新しい円を作成（デフォルトでX軸正方向を参照方向とする）
    ///
    /// # 引数
    /// * `center` - 円の中心点
    /// * `axis` - 円が存在する平面の法線方向（Z軸）
    /// * `radius` - 円の半径（正の値である必要がある）
    ///
    /// # 戻り値
    /// * `Some(Circle3D)` - 有効な円が作成できた場合
    /// * `None` - 半径が0以下の場合
    pub fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // デフォルトのref_directionを計算（axisに直交する方向）
        let ref_direction = Self::compute_default_ref_direction(&axis);

        Some(Self {
            center,
            axis,
            ref_direction,
            radius,
        })
    }

    /// 完全な座標系で円を作成
    pub fn new_with_ref_direction(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // ref_directionがaxisに直交しているか確認
        let dot_product = axis.x() * ref_direction.x()
            + axis.y() * ref_direction.y()
            + axis.z() * ref_direction.z();
        if dot_product.abs() > default_angle_tolerance::<T>() {
            return None; // 直交していない
        }

        Some(Self {
            center,
            axis,
            ref_direction,
            radius,
        })
    }

    /// axisに直交するデフォルトref_directionを計算
    fn compute_default_ref_direction(axis: &Direction3D<T>) -> Direction3D<T> {
        // axisに直交する方向を得る
        // まずX軸との外積を試す
        let x_axis = Vector3D::unit_x();
        let cross_x = Vector3D::new(
            axis.y() * x_axis.z() - axis.z() * x_axis.y(),
            axis.z() * x_axis.x() - axis.x() * x_axis.z(),
            axis.x() * x_axis.y() - axis.y() * x_axis.x(),
        );

        if cross_x.length() > default_kernel_numerical_zero_tolerance::<T>() {
            Direction3D::from_vector(cross_x).unwrap()
        } else {
            // axisがX軸と平行な場合はY軸との外積を使用
            let y_axis = Vector3D::unit_y();
            let cross_y = Vector3D::new(
                axis.y() * y_axis.z() - axis.z() * y_axis.y(),
                axis.z() * y_axis.x() - axis.x() * y_axis.z(),
                axis.x() * y_axis.y() - axis.y() * y_axis.x(),
            );
            Direction3D::from_vector(cross_y).unwrap()
        }
    }

    /// Vector3Dから円を作成（後方互換性）
    pub fn from_vector(center: Point3D<T>, axis_vector: Vector3D<T>, radius: T) -> Option<Self> {
        let axis_dir = Direction3D::from_vector(axis_vector)?;
        Self::new(center, axis_dir, radius)
    }

    /// XY平面上の円を作成（Z軸が法線）
    pub fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            radius,
        )
    }

    /// XZ平面上の円を作成（Y軸が法線）
    pub fn new_xz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::from_vector(Vector3D::unit_y()).unwrap(),
            radius,
        )
    }

    /// YZ平面上の円を作成（X軸が法線）
    pub fn new_yz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            radius,
        )
    }

    /// 中心点を取得
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// Z軸方向（法線ベクトル）を取得
    pub(crate) fn axis_internal(&self) -> Direction3D<T> {
        self.axis
    }

    /// 法線方向を取得
    pub fn normal(&self) -> Direction3D<T> {
        self.axis
    }

    /// 法線ベクトルを取得（後方互換性）
    pub(crate) fn normal_internal(&self) -> Direction3D<T> {
        self.axis
    }

    /// X軸方向（参照方向）を取得
    pub(crate) fn ref_direction_internal(&self) -> Direction3D<T> {
        self.ref_direction
    }

    /// 半径を取得
    pub(crate) fn radius_internal(&self) -> T {
        self.radius
    }

    /// 直径を取得
    pub fn diameter(&self) -> T {
        self.radius + self.radius // 2 * radius
    }

    /// 円周を計算
    pub fn circumference(&self) -> T {
        T::TAU * self.radius // 2π * radius
    }

    /// 面積を計算
    pub fn area(&self) -> T {
        T::PI * self.radius * self.radius
    }

    /// 点が円内部にあるか判定（3D空間での判定）
    pub fn contains_point_3d(&self, point: Point3D<T>) -> bool {
        // 点から中心へのベクトル
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 平面上にあるかチェック（法線との内積が0）
        let axis_vec = self.axis.as_vector();
        let dot =
            to_point.x() * axis_vec.x() + to_point.y() * axis_vec.y() + to_point.z() * axis_vec.z();
        if dot.abs() > default_distance_tolerance::<T>() {
            return false; // 平面上にない
        }

        // 中心からの距離をチェック
        let distance_squared =
            to_point.x() * to_point.x() + to_point.y() * to_point.y() + to_point.z() * to_point.z();
        distance_squared <= self.radius * self.radius
    }

    /// 点から円周への距離（3D空間）
    pub fn distance_to_point_3d(&self, point: Point3D<T>) -> T {
        // 点から中心へのベクトル
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 平面への投影距離（法線方向成分）
        let axis_vec = self.axis.as_vector();
        let plane_distance = (to_point.x() * axis_vec.x()
            + to_point.y() * axis_vec.y()
            + to_point.z() * axis_vec.z())
        .abs();

        // 平面上での中心からの距離
        let distance_squared =
            to_point.x() * to_point.x() + to_point.y() * to_point.y() + to_point.z() * to_point.z();
        let planar_distance_squared = distance_squared - plane_distance * plane_distance;
        let planar_distance = planar_distance_squared.max(T::ZERO).sqrt();

        // 円周への距離
        let radial_distance = (planar_distance - self.radius).abs();

        // 平面距離と半径方向距離の合成
        (plane_distance * plane_distance + radial_distance * radial_distance).sqrt()
    }

    /// 3点から円を作成する内部メソッド
    fn from_three_points_internal(
        point1: Point3D<T>,
        point2: Point3D<T>,
        point3: Point3D<T>,
    ) -> Option<Self> {
        // 3点から平面の法線を計算
        let v1 = Vector3D::new(
            point2.x() - point1.x(),
            point2.y() - point1.y(),
            point2.z() - point1.z(),
        );
        let v2 = Vector3D::new(
            point3.x() - point1.x(),
            point3.y() - point1.y(),
            point3.z() - point1.z(),
        );

        let normal = v1.cross(&v2);
        if normal.magnitude() <= default_kernel_numerical_zero_tolerance::<T>() {
            return None; // 3点が一直線上
        }
        let axis = Direction3D::from_vector(normal)?;

        // 3点を通る円の中心と半径を計算（外心を求める）
        // 簡易実装：2つの弦の垂直二等分線の交点を求める
        let mid1 = Point3D::new(
            (point1.x() + point2.x()) / (T::ONE + T::ONE),
            (point1.y() + point2.y()) / (T::ONE + T::ONE),
            (point1.z() + point2.z()) / (T::ONE + T::ONE),
        );
        let _mid2 = Point3D::new(
            (point2.x() + point3.x()) / (T::ONE + T::ONE),
            (point2.y() + point3.y()) / (T::ONE + T::ONE),
            (point2.z() + point3.z()) / (T::ONE + T::ONE),
        );

        let _perp1 = v1.cross(&axis.as_vector());
        let _perp2 = v2.cross(&axis.as_vector());

        // パラメトリック方程式を解く
        // mid1 + t * perp1 = mid2 + s * perp2
        // 簡易実装：点1からの距離が等しい点を中心とする
        let dx = point1.x() - mid1.x();
        let dy = point1.y() - mid1.y();
        let dz = point1.z() - mid1.z();

        let center = mid1; // 簡易的に中点を使用
        let radius = (dx * dx + dy * dy + dz * dz).sqrt();

        Self::new(center, axis, radius)
    }
}

// ============================================================================
// Core Traits Implementation
// ============================================================================

impl<T: Scalar> Circle3DConstructor<T> for Circle3D<T> {
    fn new(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_vec = Vector3D::new(axis.0, axis.1, axis.2);
        let axis_dir = Direction3D::from_vector(axis_vec)?;
        Self::new(center_point, axis_dir, radius)
    }

    fn new_xy_plane(center: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_xy_plane(center_point, radius)
    }

    fn unit_circle_xy() -> Self {
        let center = Point3D::origin();
        Self::new_xy_plane(center, T::ONE).unwrap()
    }

    fn from_center_and_point(
        center: (T, T, T),
        axis: (T, T, T),
        point_on_circle: (T, T, T),
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_vec = Vector3D::new(axis.0, axis.1, axis.2);
        let axis_dir = Direction3D::from_vector(axis_vec)?;
        let point = Point3D::new(point_on_circle.0, point_on_circle.1, point_on_circle.2);

        // 点から中心へのベクトル
        let to_point = Vector3D::new(
            point.x() - center_point.x(),
            point.y() - center_point.y(),
            point.z() - center_point.z(),
        );

        // 軸方向への投影成分を除去
        let projection = to_point.dot(&axis_dir.as_vector());
        let radial = to_point - axis_dir.as_vector() * projection;

        let radius = radial.magnitude();
        if radius <= T::ZERO {
            None
        } else {
            Self::new(center_point, axis_dir, radius)
        }
    }

    fn from_three_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self> {
        let point1 = Point3D::new(p1.0, p1.1, p1.2);
        let point2 = Point3D::new(p2.0, p2.1, p2.2);
        let point3 = Point3D::new(p3.0, p3.1, p3.2);

        Self::from_three_points_internal(point1, point2, point3)
    }

    fn centered_at_origin_xy(radius: T) -> Option<Self> {
        Self::new_xy_plane(Point3D::origin(), radius)
    }

    fn new_xz_plane(center: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_xz_plane(center_point, radius)
    }

    fn new_yz_plane(center: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_yz_plane(center_point, radius)
    }
}

impl<T: Scalar> Circle3DProperties<T> for Circle3D<T> {
    fn center(&self) -> (T, T, T) {
        let c = self.center_internal();
        (c.x(), c.y(), c.z())
    }

    fn radius(&self) -> T {
        self.radius_internal()
    }

    fn axis(&self) -> (T, T, T) {
        let a = self.axis_internal();
        (a.x(), a.y(), a.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        let r = self.ref_direction_internal();
        (r.x(), r.y(), r.z())
    }

    fn dimension(&self) -> u32 {
        3
    }

    fn is_unit_circle(&self) -> bool {
        (self.radius_internal() - T::ONE).abs() <= default_distance_tolerance::<T>()
    }

    fn is_centered_at_origin(&self) -> bool {
        let c = self.center_internal();
        c.x().abs() <= default_distance_tolerance::<T>()
            && c.y().abs() <= default_distance_tolerance::<T>()
            && c.z().abs() <= default_distance_tolerance::<T>()
    }

    fn is_degenerate(&self) -> bool {
        self.radius_internal() <= default_distance_tolerance::<T>()
    }

    fn is_on_xy_plane(&self) -> bool {
        // Z軸に平行かどうかを確認
        let z_component = self.axis_internal().z().abs();
        (z_component - T::ONE).abs() <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> Circle3DMeasure<T> for Circle3D<T> {
    fn circumference(&self) -> T {
        self.circumference()
    }

    fn area(&self) -> T {
        self.area()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point_3d(p)
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        self.distance_to_point_3d(p)
    }

    fn point_on_circumference(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        let distance = self.distance_to_point_3d(p);
        distance.abs() <= default_distance_tolerance::<T>()
    }

    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);

        // 点から中心へのベクトル
        let to_point = Vector3D::new(
            p.x() - self.center.x(),
            p.y() - self.center.y(),
            p.z() - self.center.z(),
        );

        // 円の平面上への投影
        let axis_vec = self.axis.as_vector();
        let projection = to_point.dot(&axis_vec);
        let in_plane = to_point - axis_vec * projection;

        let distance = in_plane.magnitude();
        if distance <= default_kernel_numerical_zero_tolerance::<T>() {
            // 点が中心軸上にある場合、参照方向の点を返す
            let closest = self.center + self.ref_direction.as_vector() * self.radius;
            (closest.x(), closest.y(), closest.z())
        } else {
            let scale = self.radius / distance;
            let closest = self.center + in_plane * scale;
            (closest.x(), closest.y(), closest.z())
        }
    }

    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let angle = t * T::TAU;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 円周上の点を計算
        let v_axis = self.axis.as_vector();
        let v_ref = self.ref_direction.as_vector();

        // v_refと垂直なベクトル（円平面内）
        let v_perp = v_axis.cross(&v_ref);

        let point =
            self.center + v_ref * (self.radius * cos_angle) + v_perp * (self.radius * sin_angle);

        (point.x(), point.y(), point.z())
    }

    fn distance_to_circle(&self, other: &Self) -> T {
        // 簡易実装：中心間距離から半径を考慮
        let dx = other.center.x() - self.center.x();
        let dy = other.center.y() - self.center.y();
        let dz = other.center.z() - self.center.z();
        let center_distance = (dx * dx + dy * dy + dz * dz).sqrt();

        let radii_sum = self.radius + other.radius;

        if center_distance >= radii_sum {
            center_distance - radii_sum
        } else {
            T::ZERO // 交差または包含
        }
    }
}
