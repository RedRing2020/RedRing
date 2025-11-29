// torus_solid_3d.rs
// STEP AP214 準拠のトーラス固体実装
//
// トーラス固体は主半径（major_radius）と副半径（minor_radius）を持つ回転固体です。
//
// STEP エンティティ: TORUS + AXIS2_PLACEMENT_3D
// 固体としての体積と表面を持ちます。

use crate::{Direction3D, Point3D, TorusSurface3D, Vector3D};
use geo_foundation::Scalar;
use std::f64::consts::PI;

/// STEP AP214 準拠のトーラス固体
///
/// 3D CAM 固体加工計算における基本幾何要素として実装。
/// 主半径（ドーナツの中心軸から管の中心までの距離）と
/// 副半径（管の半径）により定義される3次元固体です。
#[derive(Debug, Clone, PartialEq)]
pub struct TorusSolid3D<T: Scalar> {
    /// 原点（トーラスの中心点）
    origin: Point3D<T>,
    /// Z軸方向（トーラスの軸方向）
    z_axis: Direction3D<T>,
    /// X軸方向（主半径の基準方向）
    x_axis: Direction3D<T>,
    /// 主半径（ドーナツの中心軸から管の中心までの距離）
    major_radius: T,
    /// 副半径（管の半径）
    minor_radius: T,
}

impl<T: Scalar> TorusSolid3D<T> {
    /// 新しいトーラス固体を作成
    ///
    /// # Arguments
    /// * `origin` - トーラスの中心点
    /// * `z_axis` - トーラスの軸方向
    /// * `x_axis` - 主半径の基準方向（z_axisと直交である必要があります）
    /// * `major_radius` - 主半径（正の値である必要があります）
    /// * `minor_radius` - 副半径（正の値である必要があります）
    ///
    /// # Returns
    /// * `Some(TorusSolid3D)` - 有効なパラメータの場合
    /// * `None` - 無効なパラメータの場合（半径が非正、軸が非直交など）
    pub fn new(
        origin: Point3D<T>,
        z_axis: Direction3D<T>,
        x_axis: Direction3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        // 半径の妥当性チェック
        if major_radius <= T::ZERO || minor_radius <= T::ZERO {
            return None;
        }

        // トーラス固体の幾何学的制約: major_radius > minor_radius
        // （これにより、固体が適切にドーナツ形状になる）
        if major_radius <= minor_radius {
            return None;
        }

        // 軸の直交性チェック
        let dot_product =
            x_axis.x() * z_axis.x() + x_axis.y() * z_axis.y() + x_axis.z() * z_axis.z();
        let tolerance = T::EPSILON;
        if dot_product.abs() > tolerance {
            return None;
        }

        Some(TorusSolid3D {
            origin,
            z_axis,
            x_axis,
            major_radius,
            minor_radius,
        })
    }

    /// 標準的なトーラス固体を作成（Z軸中心、原点配置）
    ///
    /// # Arguments
    /// * `major_radius` - 主半径
    /// * `minor_radius` - 副半径
    ///
    /// # Returns
    /// * `Some(TorusSolid3D)` - 有効なパラメータの場合
    /// * `None` - 無効なパラメータの場合
    pub fn standard(major_radius: T, minor_radius: T) -> Option<Self> {
        let origin = Point3D::origin();
        let z_axis = Direction3D::from_vector(Vector3D::new(T::ZERO, T::ZERO, T::ONE))?;
        let x_axis = Direction3D::from_vector(Vector3D::new(T::ONE, T::ZERO, T::ZERO))?;

        Self::new(origin, z_axis, x_axis, major_radius, minor_radius)
    }

    /// 最小有効トーラス固体を作成（テスト用）
    pub fn minimal() -> Option<Self> {
        let major_radius = T::from_f64(2.0);
        let minor_radius = T::ONE;
        Self::standard(major_radius, minor_radius)
    }

    // アクセサメソッド
    pub(crate) fn origin_internal(&self) -> &Point3D<T> {
        &self.origin
    }
    pub(crate) fn z_axis_internal(&self) -> &Direction3D<T> {
        &self.z_axis
    }
    pub(crate) fn x_axis_internal(&self) -> &Direction3D<T> {
        &self.x_axis
    }
    pub(crate) fn major_radius_internal(&self) -> T {
        self.major_radius
    }
    pub(crate) fn minor_radius_internal(&self) -> T {
        self.minor_radius
    }

    /// Y軸方向を計算（右手座標系）
    pub(crate) fn y_axis_internal(&self) -> Direction3D<T> {
        let y_vec = Vector3D::new(
            self.z_axis.y() * self.x_axis.z() - self.z_axis.z() * self.x_axis.y(),
            self.z_axis.z() * self.x_axis.x() - self.z_axis.x() * self.x_axis.z(),
            self.z_axis.x() * self.x_axis.y() - self.z_axis.y() * self.x_axis.x(),
        );
        Direction3D::from_vector(y_vec).unwrap_or_else(|| {
            Direction3D::from_vector(Vector3D::new(T::ZERO, T::ONE, T::ZERO)).unwrap()
        })
    }

    /// 対応するトーラス面を取得
    ///
    /// トーラス固体の外表面に対応するトーラス面を返します。
    pub fn outer_surface(&self) -> Option<TorusSurface3D<T>> {
        TorusSurface3D::new(
            self.origin,
            self.z_axis,
            self.x_axis,
            self.major_radius,
            self.minor_radius,
        )
    }

    /// 体積を計算
    ///
    /// トーラス固体の体積 = 2π²R²r (R=主半径, r=副半径)
    #[allow(dead_code)]
    pub(crate) fn volume_internal(&self) -> T {
        let two = T::from_f64(2.0);
        let pi_squared = T::from_f64(PI * PI);

        two * pi_squared * self.major_radius * self.major_radius * self.minor_radius
    }

    /// 表面積を計算
    ///
    /// トーラス固体の表面積 = 4π²Rr (R=主半径, r=副半径)
    #[allow(dead_code)]
    pub(crate) fn surface_area_internal(&self) -> T {
        let four = T::from_f64(4.0);
        let pi_squared = T::from_f64(PI * PI);

        four * pi_squared * self.major_radius * self.minor_radius
    }

    /// 点が固体内部にあるかを判定
    ///
    /// # Arguments
    /// * `point` - 判定する点
    ///
    /// # Returns
    /// * `true` - 点が固体内部にある場合
    /// * `false` - 点が固体外部にある場合
    pub fn contains_point(&self, point: &Point3D<T>) -> bool {
        // 原点からの相対位置ベクトル
        let relative = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );

        // Z軸方向の成分
        let z_component = relative.x() * self.z_axis.x()
            + relative.y() * self.z_axis.y()
            + relative.z() * self.z_axis.z();

        // XY平面への投影
        let x_component = relative.x() * self.x_axis.x()
            + relative.y() * self.x_axis.y()
            + relative.z() * self.x_axis.z();

        let y_axis = self.y_axis_internal();
        let y_component =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();

        // 中心軸からの距離
        let radial_distance = (x_component * x_component + y_component * y_component).sqrt();

        // トーラス中心線からの距離
        let torus_center_distance = (radial_distance - self.major_radius).abs();

        // 副半径内にあるかチェック
        let cross_section_distance =
            (z_component * z_component + torus_center_distance * torus_center_distance).sqrt();

        cross_section_distance <= self.minor_radius
    }
}

use geo_foundation::{
    TorusSolid3DConstructor, TorusSolid3DCore, TorusSolid3DMeasure, TorusSolid3DProperties,
};

impl<T: Scalar> TorusSolid3DConstructor<T> for TorusSolid3D<T> {
    fn new(
        origin: (T, T, T),
        axis_vector: (T, T, T),
        ref_vector: (T, T, T),
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        let origin = Point3D::new(origin.0, origin.1, origin.2);
        let axis =
            Direction3D::from_vector(Vector3D::new(axis_vector.0, axis_vector.1, axis_vector.2))?;
        let x_axis =
            Direction3D::from_vector(Vector3D::new(ref_vector.0, ref_vector.1, ref_vector.2))?;
        Self::new(origin, axis, x_axis, major_radius, minor_radius)
    }

    fn new_standard(center: (T, T, T), major_radius: T, minor_radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::standard(major_radius, minor_radius).map(|mut t| {
            t.origin = center_point;
            t
        })
    }

    fn unit_torus() -> Self {
        Self::minimal().unwrap()
    }

    fn from_diameters(
        center: (T, T, T),
        axis: (T, T, T),
        major_diameter: T,
        minor_diameter: T,
    ) -> Option<Self> {
        let major_radius = major_diameter / T::from_f64(2.0);
        let minor_radius = minor_diameter / T::from_f64(2.0);
        let axis_vec = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_direction = if axis_vec.z().abs() < T::from_f64(0.99) {
            Vector3D::new(T::ZERO, T::ZERO, T::ONE).cross(&axis_vec)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO).cross(&axis_vec)
        };
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_dir = Direction3D::from_vector(axis_vec)?;
        let ref_dir = Direction3D::from_vector(ref_direction)?;
        Self::new(
            center_point,
            axis_dir,
            ref_dir,
            major_radius,
            minor_radius,
        )
    }

    fn from_radii_and_axis(
        center: (T, T, T),
        axis: (T, T, T),
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        let axis_vec = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_direction = if axis_vec.z().abs() < T::from_f64(0.99) {
            Vector3D::new(T::ZERO, T::ZERO, T::ONE).cross(&axis_vec)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO).cross(&axis_vec)
        };
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_dir = Direction3D::from_vector(axis_vec)?;
        let ref_dir = Direction3D::from_vector(ref_direction)?;
        Self::new(
            center_point,
            axis_dir,
            ref_dir,
            major_radius,
            minor_radius,
        )
    }

    fn ring_torus(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self> {
        Self::from_radii_and_axis(center, axis, radius, radius)
    }
}

impl<T: Scalar> TorusSolid3DProperties<T> for TorusSolid3D<T> {
    fn center(&self) -> (T, T, T) {
        let o = self.origin_internal();
        (o.x(), o.y(), o.z())
    }

    fn axis(&self) -> (T, T, T) {
        let a = self.z_axis_internal();
        (a.x(), a.y(), a.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        let r = self.x_axis_internal();
        (r.x(), r.y(), r.z())
    }

    fn major_radius(&self) -> T {
        self.major_radius_internal()
    }

    fn minor_radius(&self) -> T {
        self.minor_radius_internal()
    }

    fn tube_diameter(&self) -> T {
        self.minor_radius_internal() * T::from_f64(2.0)
    }

    fn aspect_ratio(&self) -> T {
        self.major_radius_internal() / self.minor_radius_internal()
    }

    fn outer_radius(&self) -> T {
        self.major_radius_internal() + self.minor_radius_internal()
    }

    fn inner_radius(&self) -> T {
        (self.major_radius_internal() - self.minor_radius_internal()).max(T::ZERO)
    }
}

impl<T: Scalar> TorusSolid3DMeasure<T> for TorusSolid3D<T> {
    fn volume(&self) -> T {
        self.volume_internal()
    }

    fn surface_area(&self) -> T {
        self.surface_area_internal()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point(&p)
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        // 簡易実装: トーラス表面への最短距離の近似計算
        let local = p - *self.origin_internal();
        let z_axis = self.z_axis_internal();
        let z_component = local.dot(&z_axis.as_vector());
        let radial_vector = local - (z_axis.as_vector() * z_component);
        let radial_distance = (radial_vector.x() * radial_vector.x()
            + radial_vector.y() * radial_vector.y()
            + radial_vector.z() * radial_vector.z())
        .sqrt();
        let torus_center_distance = (radial_distance - self.major_radius_internal()).abs();
        let cross_section_distance =
            (z_component * z_component + torus_center_distance * torus_center_distance).sqrt();
        (cross_section_distance - self.minor_radius_internal()).abs()
    }

    fn point_at_toroidal(&self, u: T, v: T) -> (T, T, T) {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();
        
        let o = self.origin_internal();
        let x_axis = self.x_axis_internal().as_vector();
        let y_axis = self.y_axis_internal().as_vector();
        let z_axis = self.z_axis_internal().as_vector();
        
        let r_major = self.major_radius_internal();
        let r_minor = self.minor_radius_internal();
        
        let circle_radius = r_major + r_minor * cos_v;
        
        let x = o.x() + circle_radius * cos_u * x_axis.x() + circle_radius * sin_u * y_axis.x() + r_minor * sin_v * z_axis.x();
        let y = o.y() + circle_radius * cos_u * x_axis.y() + circle_radius * sin_u * y_axis.y() + r_minor * sin_v * z_axis.y();
        let z = o.z() + circle_radius * cos_u * x_axis.z() + circle_radius * sin_u * y_axis.z() + r_minor * sin_v * z_axis.z();
        
        (x, y, z)
    }

    fn bounding_box(&self) -> ((T, T, T), (T, T, T)) {
        let o = self.origin_internal();
        let r_outer = self.major_radius_internal() + self.minor_radius_internal();
        
        let min_x = o.x() - r_outer;
        let max_x = o.x() + r_outer;
        let min_y = o.y() - r_outer;
        let max_y = o.y() + r_outer;
        let min_z = o.z() - self.minor_radius_internal();
        let max_z = o.z() + self.minor_radius_internal();
        
        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }

    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let local = p - *self.origin_internal();
        let z_axis = self.z_axis_internal();
        let z_component = local.dot(&z_axis.as_vector());
        let radial_vector = local - (z_axis.as_vector() * z_component);
        let radial_distance = radial_vector.length();
        
        let radial_dir = if radial_distance > T::EPSILON {
            radial_vector / radial_distance
        } else {
            self.x_axis_internal().as_vector()
        };
        
        let torus_center = radial_dir * self.major_radius_internal();
        let to_surface = Vector3D::new(
            local.x() - torus_center.x(),
            local.y() - torus_center.y(),
            local.z() - torus_center.z(),
        );
        let distance_to_tube = to_surface.length();
        
        let surface_dir = if distance_to_tube > T::EPSILON {
            to_surface / distance_to_tube
        } else {
            z_axis.as_vector()
        };
        
        let surface_point = torus_center + surface_dir * self.minor_radius_internal();
        let o = self.origin_internal();
        
        (
            o.x() + surface_point.x(),
            o.y() + surface_point.y(),
            o.z() + surface_point.z(),
        )
    }

    fn is_self_intersecting(&self) -> bool {
        self.major_radius_internal() < self.minor_radius_internal()
    }
}

impl<T: Scalar> TorusSolid3DCore<T> for TorusSolid3D<T> {}
