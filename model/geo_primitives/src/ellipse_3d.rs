//! 3D楕円のCore実装
//!
//! Foundation統一システムに基づくEllipse3Dの必須機能のみ

use crate::{
    ellipse_calculation_analysis, ellipse_calculation_strategy, Angle, Circle3D, Direction3D,
    InfiniteLine3D, Plane3D, Point3D, Vector3D,
};
use geo_contracts::MultipleIntersection;
use geo_contracts::{default_angle_tolerance, default_distance_tolerance};
use geo_contracts::{
    Ellipse3DConstructor, Ellipse3DContainment, Ellipse3DDerived, Ellipse3DDistance,
    Ellipse3DEvaluation, Ellipse3DProperties, Scalar,
};
use geo_contracts::{EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation};

/// 3次元楕円（Core実装）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse3D<T: Scalar> {
    center: Point3D<T>,
    semi_major_axis: T,
    semi_minor_axis: T,
    normal: Direction3D<T>,         // 楕円平面の法線ベクトル（正規化済み）
    major_axis_dir: Direction3D<T>, // 長軸方向ベクトル（正規化済み）
}

impl<T: Scalar> Ellipse3D<T> {
    /// 新しい3D楕円を作成
    ///
    /// 基本的な検証のみ実行
    pub fn new(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        normal: Vector3D<T>,
        major_axis_dir: Vector3D<T>,
    ) -> Option<Self> {
        // 半軸の長さの検証
        if semi_major_axis < semi_minor_axis || semi_minor_axis <= T::ZERO {
            return None;
        }

        // Direction3Dに変換（自動的に正規化される）
        let normal_dir = Direction3D::from_vector(normal)?;
        let major_axis_dir = Direction3D::from_vector(major_axis_dir)?;

        // 基本的な直交性チェック
        let dot_product = normal_dir.dot(&major_axis_dir);
        if dot_product.abs() > default_angle_tolerance::<T>() {
            return None;
        }

        Some(Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            normal: normal_dir,
            major_axis_dir,
        })
    }

    /// XY平面上の軸に平行な楕円を作成
    pub fn xy_aligned(center: Point3D<T>, semi_major_axis: T, semi_minor_axis: T) -> Option<Self> {
        Self::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_z(),
            Vector3D::unit_x(),
        )
    }

    /// 3D円から楕円を作成
    pub fn from_circle(circle: &Circle3D<T>) -> Option<Self> {
        let normal_dir = circle.normal_internal();
        let u_axis_dir = circle.ref_direction_internal();

        Some(Self {
            center: circle.center_internal(),
            semi_major_axis: circle.radius_internal(),
            semi_minor_axis: circle.radius_internal(),
            normal: normal_dir,
            major_axis_dir: u_axis_dir,
        })
    }
    /// 楕円の中心点を取得（内部使用）
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// 長半軸の長さを取得（内部使用）
    pub(crate) fn semi_major_internal(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得（内部使用）
    pub(crate) fn semi_minor_internal(&self) -> T {
        self.semi_minor_axis
    }

    /// 楕円の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center_internal()
    }

    /// 長半軸の長さを取得
    pub fn semi_major_axis(&self) -> T {
        self.semi_major_internal()
    }

    /// 短半軸の長さを取得
    pub fn semi_minor_axis(&self) -> T {
        self.semi_minor_internal()
    }

    /// 楕円平面の法線ベクトルを取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// 長軸方向ベクトルを取得
    pub fn major_axis_direction(&self) -> Direction3D<T> {
        self.major_axis_dir
    }

    /// 短軸方向ベクトルを取得
    pub fn minor_axis_direction(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.normal.cross(&self.major_axis_dir))
            .expect("Cross product of normalized vectors should be valid")
    }
    /// 離心率を計算
    pub fn eccentricity(&self) -> T {
        if self.semi_major_axis == T::ZERO {
            return T::ZERO;
        }
        let ratio = self.semi_minor_axis / self.semi_major_axis;
        (T::ONE - ratio * ratio).sqrt()
    }

    /// 楕円の面積を計算
    pub fn area(&self) -> T {
        T::PI * self.semi_major_axis * self.semi_minor_axis
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        let tolerance = default_distance_tolerance::<T>();
        (self.semi_major_axis - self.semi_minor_axis).abs() <= tolerance
    }

    /// 楕円が退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        let tolerance = default_distance_tolerance::<T>();
        self.semi_minor_axis <= tolerance
    }

    /// 円への変換（円の場合のみ）
    pub fn to_circle(&self) -> Option<Circle3D<T>> {
        if self.is_circle() {
            Circle3D::new(self.center, self.normal, self.semi_major_axis)
        } else {
            None
        }
    }
    /// パラメータ t での楕円上の点を計算
    /// t ∈ [0, 2π]
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir.as_vector() * (self.semi_major_axis * cos_t);
        let minor_component =
            self.minor_axis_direction().as_vector() * (self.semi_minor_axis * sin_t);

        Point3D::new(
            self.center.x() + major_component.x() + minor_component.x(),
            self.center.y() + major_component.y() + minor_component.y(),
            self.center.z() + major_component.z() + minor_component.z(),
        )
    }

    /// パラメータ t での楕円の接線ベクトルを計算
    /// t ∈ [0, 2π]
    pub fn tangent_at_parameter(&self, t: T) -> Vector3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir.as_vector() * (-self.semi_major_axis * sin_t);
        let minor_component =
            self.minor_axis_direction().as_vector() * (self.semi_minor_axis * cos_t);

        major_component + minor_component
    }

    /// 角度 θ での楕円上の点を計算
    pub fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        self.point_at_parameter(angle.to_radians())
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU)
    }

    /// 3D空間での点から楕円への最短距離を計算（内部実装）
    fn distance_to_point_3d_internal(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);

        // 点を楕円の座標系に変換
        let translated = Vector3D::new(
            p.x() - self.center.x(),
            p.y() - self.center.y(),
            p.z() - self.center.z(),
        );

        // 楕円平面への射影
        let u = self.major_axis_dir.as_vector();
        let v = self.minor_axis_direction().as_vector();

        let x_local = translated.dot(&u);
        let y_local = translated.dot(&v);

        // 平面外成分（法線方向）
        let n = self.normal.as_vector();
        let z_local = translated.dot(&n);

        // geo_commonsの共通実装を使用
        geo_commons::ellipse_3d_distance_to_point(
            x_local,
            y_local,
            z_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }
}

impl<T: Scalar> Ellipse3DConstructor<T> for Ellipse3D<T> {
    /// 基本コンストラクタ（中心点、平面法線、長軸半径、短軸半径、長軸方向）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        major_axis_direction: (T, T, T),
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let major_dir_vec = Vector3D::new(
            major_axis_direction.0,
            major_axis_direction.1,
            major_axis_direction.2,
        );

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            normal_vec,
            major_dir_vec,
        )
    }

    /// 完全な座標系で作成
    fn new_with_coordinate_system(
        center: (T, T, T),
        normal: (T, T, T),
        major_axis_direction: (T, T, T),
        _minor_axis_direction: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
    ) -> Option<Self> {
        // minor_axis_directionは無視し、major_axis_directionとnormalから自動計算
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let major_dir_vec = Vector3D::new(
            major_axis_direction.0,
            major_axis_direction.1,
            major_axis_direction.2,
        );

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            normal_vec,
            major_dir_vec,
        )
    }

    /// XY平面上の楕円作成
    fn new_xy_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_dir = Vector3D::new(cos_rot, sin_rot, T::ZERO);

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_z(),
            major_dir,
        )
    }
    /// XZ平面上の楕円作成
    fn new_xz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_dir = Vector3D::new(cos_rot, T::ZERO, sin_rot);

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_y(),
            major_dir,
        )
    }

    /// YZ平面上の楕円作成
    fn new_yz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_dir = Vector3D::new(T::ZERO, cos_rot, sin_rot);

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_x(),
            major_dir,
        )
    }

    /// XY平面単位楕円
    fn unit_ellipse_xy() -> Self {
        Self {
            center: Point3D::origin(),
            semi_major_axis: T::ONE,
            semi_minor_axis: T::ONE,
            normal: Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            major_axis_dir: Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
        }
    }
}

impl<T: Scalar> Ellipse3DProperties<T> for Ellipse3D<T> {
    /// 楕円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> (T, T, T) {
        (self.normal.x(), self.normal.y(), self.normal.z())
    }

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> (T, T, T) {
        (
            self.major_axis_dir.x(),
            self.major_axis_dir.y(),
            self.major_axis_dir.z(),
        )
    }

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> (T, T, T) {
        let minor_axis = self.minor_axis_direction();
        (minor_axis.x(), minor_axis.y(), minor_axis.z())
    }

    /// 3D中心座標を取得
    fn center_3d(&self) -> (T, T, T) {
        (self.center.x(), self.center.y(), self.center.z())
    }

    /// 3D中心点をタプルとして取得
    fn center_3d_tuple(&self) -> (T, T, T) {
        (self.center.x(), self.center.y(), self.center.z())
    }
    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }
}

impl<T: Scalar + From<f64>> Ellipse3DDerived<T> for Ellipse3D<T> {
    fn area(&self) -> T {
        Ellipse3D::area(self)
    }

    fn circumference(&self) -> T {
        <Self as geo_contracts::EllipseCalculation<T>>::perimeter_ramanujan_ii(self)
    }

    fn perimeter(&self) -> T {
        <Self as Ellipse3DDerived<T>>::circumference(self)
    }

    fn measure(&self) -> T {
        <Self as Ellipse3DDerived<T>>::area(self)
    }

    /// 離心率を取得
    fn eccentricity(&self) -> T {
        Ellipse3D::eccentricity(self)
    }

    fn focal_distance(&self) -> T {
        geo_contracts::EllipseCalculation::focal_distance(self)
    }

    /// 楕円が円かどうか判定
    fn is_circle(&self) -> bool {
        let tolerance = default_distance_tolerance::<T>();
        (self.semi_major_axis - self.semi_minor_axis).abs() <= tolerance
    }
}

impl<T: Scalar + From<f64>> Ellipse3DEvaluation<T> for Ellipse3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let p = Ellipse3D::point_at_parameter(self, t);
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar + From<f64>> Ellipse3DContainment<T> for Ellipse3D<T> {
    fn contains_point_3d(&self, point: (T, T, T)) -> bool {
        self.distance_to_point_3d_internal(point) <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar + From<f64>> Ellipse3DDistance<T> for Ellipse3D<T> {
    fn distance_to_point_3d(&self, point: (T, T, T)) -> T {
        self.distance_to_point_3d_internal(point)
    }
}

impl<T: Scalar + From<f64>> MultipleIntersection<T, InfiniteLine3D<T>> for Ellipse3D<T> {
    type Point = (T, T, T);

    fn intersections_with(&self, _other: &InfiniteLine3D<T>, _tolerance: T) -> Vec<Self::Point> {
        Vec::new()
    }
}

impl<T: Scalar + From<f64>> MultipleIntersection<T, Plane3D<T>> for Ellipse3D<T> {
    type Point = (T, T, T);

    fn intersections_with(&self, _other: &Plane3D<T>, _tolerance: T) -> Vec<Self::Point> {
        Vec::new()
    }
}

impl<T: Scalar> EllipseCalculation<T> for Ellipse3D<T> {
    type Point = Point3D<T>;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }

    fn perimeter_ramanujan_i(&self) -> T {
        geo_commons::ellipse_perimeter_ramanujan_i(self.semi_major_axis, self.semi_minor_axis)
    }

    fn perimeter_ramanujan_ii(&self) -> T {
        geo_commons::ellipse_perimeter_ramanujan_ii(self.semi_major_axis, self.semi_minor_axis)
    }

    fn perimeter_pade(&self) -> T {
        geo_commons::ellipse_perimeter_padé(self.semi_major_axis, self.semi_minor_axis)
    }

    fn perimeter_cantrell(&self) -> T {
        geo_commons::ellipse_perimeter_cantrell(self.semi_major_axis, self.semi_minor_axis)
    }

    fn perimeter_series(&self, terms: usize) -> T {
        geo_commons::ellipse_circumference_series(self.semi_major_axis, self.semi_minor_axis, terms)
    }

    fn perimeter_numerical(&self, n_points: usize) -> T {
        geo_commons::ellipse_circumference_numerical(
            self.semi_major_axis,
            self.semi_minor_axis,
            n_points,
        )
    }

    fn eccentricity(&self) -> T {
        geo_commons::ellipse_eccentricity(self.semi_major_axis, self.semi_minor_axis)
    }

    fn focal_distance(&self) -> T {
        geo_commons::ellipse_focal_distance(self.semi_major_axis, self.semi_minor_axis)
    }

    fn area(&self) -> T {
        T::PI * self.semi_major_axis * self.semi_minor_axis
    }

    /// 楕円の焦点座標を計算（3D空間）
    fn foci(&self) -> (Point3D<T>, Point3D<T>) {
        let foci_tuple = geo_commons::ellipse_foci(self.semi_major_axis, self.semi_minor_axis);
        let (f1_local, f2_local) = (foci_tuple.0, foci_tuple.1);

        // 楕円平面内での焦点（長軸方向に配置）
        let f1_vec = self.major_axis_dir.as_vector() * f1_local.0;
        let f2_vec = self.major_axis_dir.as_vector() * f2_local.0;

        // 3D空間での焦点座標
        let f1_final = self.center + f1_vec;
        let f2_final = self.center + f2_vec;

        (f1_final, f2_final)
    }
}

impl<T: Scalar> EllipseAdaptiveCalculation<T> for Ellipse3D<T> {
    fn perimeter_adaptive(&self, target_accuracy: T, max_computation_cost: T) -> T {
        ellipse_calculation_strategy::perimeter_adaptive(
            self,
            target_accuracy,
            max_computation_cost,
        )
    }
}

impl<T: Scalar> EllipseAccuracyAnalysis<T> for Ellipse3D<T> {
    fn compare_approximation_methods(&self) -> Vec<(&'static str, T, T)> {
        ellipse_calculation_analysis::compare_approximation_methods(self)
    }
}
