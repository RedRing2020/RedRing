//! 3D楕円のCore実装
//!
//! Foundation統一システムに基づくEllipse3Dの必須機能のみ

use crate::{Angle, Circle3D, Direction3D, Point3D, Vector3D};
use analysis::linalg::vector::{Vector2, Vector3};
use geo_foundation::prelude::{
    EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
};
use geo_foundation::{
    tolerance_migration::DefaultTolerances, Ellipse2DMeasure, Ellipse2DProperties,
    Ellipse3DConstructor, Ellipse3DMeasure, Ellipse3DProperties, Scalar,
};

/// 3次元楕円（Core実装）
///
/// Core機能のみ：
/// - 基本構築・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ
/// - 基本パラメトリック操作
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse3D<T: Scalar> {
    center: Point3D<T>,
    semi_major_axis: T,
    semi_minor_axis: T,
    normal: Direction3D<T>,         // 楕円平面の法線ベクトル（正規化済み）
    major_axis_dir: Direction3D<T>, // 長軸方向ベクトル（正規化済み）
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================
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
        if dot_product.abs() > DefaultTolerances::distance::<T>() {
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
        let normal_dir = circle.normal();
        let u_axis_dir = circle.u_axis();

        Some(Self {
            center: circle.center(),
            semi_major_axis: circle.radius(),
            semi_minor_axis: circle.radius(),
            normal: normal_dir,
            major_axis_dir: u_axis_dir,
        })
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 楕円の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 長半軸の長さを取得
    pub fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    pub fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
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

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

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
        let tolerance = DefaultTolerances::distance::<T>();
        (self.semi_major_axis - self.semi_minor_axis).abs() <= tolerance
    }

    /// 楕円が退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
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

    // ========================================================================
    // Core Parametric Methods
    // ========================================================================

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
}

// ============================================================================
// Foundation Pattern: Core Traits Implementation
// ============================================================================

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

// Ellipse2DProperties の実装（継承のため必要）
impl<T: Scalar> Ellipse2DProperties<T> for Ellipse3D<T> {
    /// 楕円の中心座標を取得
    fn center(&self) -> (T, T) {
        (self.center.x(), self.center.y())
    }

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T {
        // 3D楕円の場合、2D回転角の概念は複雑
        // 簡易実装: major_axis_dir のXY平面での角度
        let x_component = self.major_axis_dir.x();
        let y_component = self.major_axis_dir.y();
        y_component.atan2(x_component)
    }

    /// 楕円の焦点間距離を取得
    fn focal_distance(&self) -> T {
        geo_foundation::prelude::commons::ellipse_focal_distance(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// 第1焦点の座標を取得
    fn focus1(&self) -> (T, T) {
        let (f1, _f2) = self.foci();
        (f1.x(), f1.y())
    }

    /// 第2焦点の座標を取得
    fn focus2(&self) -> (T, T) {
        let (_f1, f2) = self.foci();
        (f2.x(), f2.y())
    }

    /// Analysis層互換の座標変換
    fn to_analysis_vector(&self) -> Vector2<T> {
        Vector2::new(self.center.x(), self.center.y())
    }

    /// 中心点をタプルとして取得
    fn center_tuple(&self) -> (T, T) {
        (self.center.x(), self.center.y())
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

    /// Analysis層互換の3D座標変換
    fn to_analysis_vector_3d(&self) -> Vector3<T> {
        Vector3::new(self.center.x(), self.center.y(), self.center.z())
    }

    /// 3D中心点をタプルとして取得
    fn center_3d_tuple(&self) -> (T, T, T) {
        (self.center.x(), self.center.y(), self.center.z())
    }
}

// Ellipse2DMeasure の実装（継承のため必要）
impl<T: Scalar + From<f64>> Ellipse2DMeasure<T> for Ellipse3D<T> {
    /// 楕円の面積を計算
    fn area(&self) -> T {
        T::PI * self.semi_major_axis * self.semi_minor_axis
    }

    /// 楕円の周長を計算（近似）
    fn perimeter(&self) -> T {
        let a = self.semi_major_axis;
        let b = self.semi_minor_axis;
        let h = ((a - b) * (a - b)) / ((a + b) * (a + b));

        // ラマヌジャンの第二近似式の簡易版
        let three = T::ONE + T::ONE + T::ONE;
        let ten = three + three + three + T::ONE;
        let four = T::ONE + T::ONE + T::ONE + T::ONE;

        T::PI * (a + b) * (T::ONE + (three * h) / (ten + (four - three * h).sqrt()))
    }

    /// 楕円の正確な周長を計算（数値積分）
    fn perimeter_exact(&self, _tolerance: T) -> T {
        // 数値積分による正確な周長計算は複雑
        // 現在は近似式を使用
        self.perimeter()
    }

    /// 楕円の離心率を計算
    fn eccentricity(&self) -> T {
        if self.semi_major_axis == T::ZERO {
            return T::ZERO;
        }

        let e_squared = T::ONE
            - (self.semi_minor_axis * self.semi_minor_axis)
                / (self.semi_major_axis * self.semi_major_axis);
        if e_squared <= T::ZERO {
            T::ZERO
        } else {
            e_squared.sqrt()
        }
    }

    /// 点が楕円内部にあるかを判定（2D投影）
    fn contains_point(&self, point: (T, T)) -> bool {
        // 簡易実装：2D投影での判定
        let test_point = Point3D::new(point.0, point.1, self.center.z());
        self.distance_to_point_3d_internal((test_point.x(), test_point.y(), test_point.z()))
            <= geo_foundation::GEOMETRIC_DISTANCE_TOLERANCE.into()
    }

    /// 点が楕円境界上にあるかを判定（許容誤差考慮）
    fn on_ellipse(&self, point: (T, T), tolerance: T) -> bool {
        let test_point = Point3D::new(point.0, point.1, self.center.z());
        self.distance_to_point_3d_internal((test_point.x(), test_point.y(), test_point.z()))
            <= tolerance
    }

    /// 点から楕円への最短距離を計算（2D投影）
    fn distance_to_point(&self, point: (T, T)) -> T {
        let test_point = Point3D::new(point.0, point.1, self.center.z());
        self.distance_to_point_3d_internal((test_point.x(), test_point.y(), test_point.z()))
    }

    /// 楕円が円に近いかを判定
    fn is_nearly_circular(&self, tolerance: T) -> bool {
        let ratio = self.semi_minor_axis / self.semi_major_axis;
        (ratio - T::ONE).abs() <= tolerance
    }

    /// 楕円が完全な円かを判定
    fn is_circle(&self) -> bool {
        self.semi_major_axis == self.semi_minor_axis
    }

    /// 他の楕円との交点を計算（未実装）
    fn intersection_with_ellipse(&self, _other: &Self) -> Vec<(T, T)> {
        // 3D楕円同士の交点計算は極めて複雑
        Vec::new()
    }

    /// 直線との交点を計算（未実装）
    fn intersection_with_line(&self, _line_point: (T, T), _line_direction: (T, T)) -> Vec<(T, T)> {
        // 3D楕円と2D直線の交点計算は複雑
        Vec::new()
    }
}

impl<T: Scalar + From<f64>> Ellipse3DMeasure<T> for Ellipse3D<T> {
    /// 3D空間での点が楕円内部にあるかを判定
    fn contains_point_3d(&self, point: (T, T, T)) -> bool {
        self.distance_to_point_3d_internal(point)
            <= geo_foundation::GEOMETRIC_DISTANCE_TOLERANCE.into()
    }

    /// 3D空間での点から楕円への最短距離を計算
    fn distance_to_point_3d(&self, point: (T, T, T)) -> T {
        self.distance_to_point_3d_internal(point)
    }

    /// 3D空間での直線との交点を計算（未実装）
    fn intersection_with_line_3d(
        &self,
        _line_point: (T, T, T),
        _line_direction: (T, T, T),
    ) -> Vec<(T, T, T)> {
        // 3D楕円と3D直線の交点計算は複雑
        Vec::new()
    }

    /// 平面との交点を計算（未実装）
    fn intersection_with_plane(
        &self,
        _plane_point: (T, T, T),
        _plane_normal: (T, T, T),
    ) -> Vec<(T, T, T)> {
        // 3D楕円と平面の交点計算は複雑
        Vec::new()
    }
}

// 内部実装用のヘルパーメソッド
impl<T: Scalar> Ellipse3D<T> {
    /// 3D空間での点から楕円への最短距離の内部実装
    fn distance_to_point_3d_internal(&self, point: (T, T, T)) -> T {
        let test_point = Point3D::new(point.0, point.1, point.2);

        // 楕円平面への投影を計算（簡易実装）
        let to_point = Vector3D::from_points(&self.center, &test_point);
        let projected_distance = to_point.dot(&self.normal);

        // 平面からの距離と楕円境界からの距離を組み合わせ
        let plane_distance = projected_distance.abs();

        // 平面内での楕円境界からの距離（簡易計算）
        let projected_point = test_point - self.normal.as_vector() * projected_distance;
        let to_projected = Vector3D::from_points(&self.center, &projected_point);

        let u_component = to_projected.dot(&self.major_axis_dir);
        let v_component = to_projected.dot(&self.minor_axis_direction());

        let normalized_u = u_component / self.semi_major_axis;
        let normalized_v = v_component / self.semi_minor_axis;
        let normalized_distance =
            (normalized_u * normalized_u + normalized_v * normalized_v).sqrt();

        let boundary_distance = if normalized_distance <= T::ONE {
            T::ZERO
        } else {
            // 簡易境界距離計算
            (normalized_distance - T::ONE) * self.semi_major_axis.min(self.semi_minor_axis)
        };

        (plane_distance * plane_distance + boundary_distance * boundary_distance).sqrt()
    }
}

// ============================================================================
// Advanced Calculation Traits Implementation
// ============================================================================

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

    /// ラマヌジャン近似式I（標準版）による周長計算
    fn perimeter_ramanujan_i(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_ramanujan_i(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// ラマヌジャン近似式II（高精度版）による周長計算
    fn perimeter_ramanujan_ii(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_ramanujan_ii(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// パダン近似による周長計算（中程度精度）
    fn perimeter_pade(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_padé(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// カントレル近似による周長計算（高精度）
    fn perimeter_cantrell(&self) -> T {
        geo_foundation::prelude::commons::ellipse_perimeter_cantrell(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// 無限級数展開による周長計算（高精度版）
    fn perimeter_series(&self, terms: usize) -> T {
        geo_foundation::prelude::commons::ellipse_circumference_series(
            self.semi_major_axis,
            self.semi_minor_axis,
            terms,
        )
    }

    /// 数値積分による周長計算（最高精度版）
    fn perimeter_numerical(&self, n_points: usize) -> T {
        geo_foundation::prelude::commons::ellipse_circumference_numerical(
            self.semi_major_axis,
            self.semi_minor_axis,
            n_points,
        )
    }

    /// 楕円の離心率計算
    fn eccentricity(&self) -> T {
        geo_foundation::prelude::commons::ellipse_eccentricity(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// 楕円の焦点距離計算
    fn focal_distance(&self) -> T {
        geo_foundation::prelude::commons::ellipse_focal_distance(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// 楕円の面積計算
    fn area(&self) -> T {
        geo_foundation::prelude::commons::metrics::area_volume::ellipse_area(
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// 楕円の焦点座標を計算（3D空間）
    fn foci(&self) -> (Point3D<T>, Point3D<T>) {
        let foci_tuple = geo_foundation::prelude::commons::ellipse_foci(
            self.semi_major_axis,
            self.semi_minor_axis,
        );
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

impl<T: Scalar> EllipseAdaptiveCalculation<T> for Ellipse3D<T> {}
impl<T: Scalar> EllipseAccuracyAnalysis<T> for Ellipse3D<T> {}
