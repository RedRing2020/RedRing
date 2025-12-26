//! 3次元楕円体ソリッド（EllipsoidalSolid3D）のCore実装
//!
//! STEP準拠の ELLIPSOIDAL_SOLID + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//! 拡張機能は ellipsoidal_solid_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! ELLIPSOIDAL_SOLID('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), A_RADIUS, B_RADIUS, C_RADIUS);
//! ```
//! - location: 楕円体の中心点（center）
//! - axis: Z軸方向（楕円体軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - a_radius: X軸方向の半径
//! - b_radius: Y軸方向の半径
//! - c_radius: Z軸方向の半径

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元楕円体ソリッド（STEP準拠のCore実装）
///
/// STEP AP214の ELLIPSOIDAL_SOLID + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：立体として体積・内部判定を持つ
///
/// ## 座標系定義（STEP準拠）
/// - center: 楕円体の中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 楕円体軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - a_radius: X軸方向の半径
/// - b_radius: Y軸方向の半径
/// - c_radius: Z軸方向の半径
///
/// ## ソリッド特性
/// - 体積計算：V = (4/3)π × a × b × c
/// - 内部判定：点の包含テスト (x/a)² + (y/b)² + (z/c)² ≤ 1
/// - 表面積計算：Knudの近似式を使用
/// - 境界ボックス：中心を基準とした直方体
///
/// ## CAD用途
/// - パラメトリック楕円体ソリッドの基準座標系
/// - ブーリアン演算（和・差・積）
/// - STEPファイルとの相互変換
/// - 体積・質量特性計算
#[derive(Debug, Clone, PartialEq)]
pub struct EllipsoidalSolid3D<T: Scalar> {
    /// 楕円体の中心点（STEP: location）
    center: Point3D<T>,

    /// 楕円体の軸方向（STEP: axis）- Z軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    /// axis と直交していなくても自動調整
    ref_direction: Direction3D<T>,

    /// X軸方向の半径
    a_radius: T,

    /// Y軸方向の半径
    b_radius: T,

    /// Z軸方向の半径
    c_radius: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> EllipsoidalSolid3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で楕円体ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 中心点
    /// * `axis` - 軸方向ベクトル（楕円体軸、Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `a_radius` - X軸方向の半径（正の値）
    /// * `b_radius` - Y軸方向の半径（正の値）
    /// * `c_radius` - Z軸方向の半径（正の値）
    ///
    /// # Returns
    /// 有効な楕円体ソリッドが作成できた場合は `Some(EllipsoidalSolid3D)`、
    /// 無効なパラメータの場合は `None`
    ///
    /// # アルゴリズム
    /// 1. 軸を正規化してZ軸とする
    /// 2. 参照方向から軸成分を除去（グラム・シュミット正規直交化）
    /// 3. 正規化して参照方向とする
    pub fn new(
        center: Point3D<T>,
        axis: Vector3D<T>,
        ref_direction: Vector3D<T>,
        a_radius: T,
        b_radius: T,
        c_radius: T,
    ) -> Option<Self> {
        // 半径の検証
        if a_radius <= T::ZERO || b_radius <= T::ZERO || c_radius <= T::ZERO {
            return None;
        }

        // 軸の正規化
        let z_axis = Direction3D::from_vector(axis)?;

        // 参照方向の軸成分を除去して正規化（グラム・シュミット正規直交化）
        let axis_component = ref_direction.dot(&z_axis.as_vector());
        let orthogonal_ref = ref_direction - z_axis.as_vector() * axis_component;

        // 参照方向の正規化
        let x_axis = Direction3D::from_vector(orthogonal_ref)?;

        Some(Self {
            center,
            axis: z_axis,
            ref_direction: x_axis,
            a_radius,
            b_radius,
            c_radius,
        })
    }

    /// Z軸標準の楕円体ソリッドを作成（簡易コンストラクタ）
    pub fn new_standard(center: Point3D<T>, a_radius: T, b_radius: T, c_radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            a_radius,
            b_radius,
            c_radius,
        )
    }

    /// 原点中心の楕円体ソリッドを作成（簡易コンストラクタ）
    pub fn new_at_origin(a_radius: T, b_radius: T, c_radius: T) -> Option<Self> {
        Self::new_standard(
            Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            a_radius,
            b_radius,
            c_radius,
        )
    }

    /// 球ソリッドとして作成（a = b = c の特殊ケース）
    pub fn new_sphere(
        center: Point3D<T>,
        axis: Vector3D<T>,
        ref_direction: Vector3D<T>,
        radius: T,
    ) -> Option<Self> {
        Self::new(center, axis, ref_direction, radius, radius, radius)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 楕円体の中心点を取得
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// 楕円体の軸方向を取得（正規化済み）
    pub(crate) fn axis_internal(&self) -> Direction3D<T> {
        self.axis
    }

    /// 参照方向を取得（正規化済み、X軸相当）
    pub(crate) fn ref_direction_internal(&self) -> Direction3D<T> {
        self.ref_direction
    }

    /// Y軸方向を計算（axis × ref_direction）
    pub(crate) fn y_axis_internal(&self) -> Direction3D<T> {
        let y_vector = self.axis.as_vector().cross(&self.ref_direction.as_vector());
        Direction3D::from_vector(y_vector)
            .expect("Y-axis calculation should always succeed with orthogonal axes")
    }

    /// X軸方向の半径を取得
    pub(crate) fn a_radius_internal(&self) -> T {
        self.a_radius
    }

    /// Y軸方向の半径を取得
    pub(crate) fn b_radius_internal(&self) -> T {
        self.b_radius
    }

    /// Z軸方向の半径を取得
    pub(crate) fn c_radius_internal(&self) -> T {
        self.c_radius
    }

    // ========================================================================
    // Core Geometric Properties (ソリッド特性)
    // ========================================================================

    /// 楕円体ソリッドの体積を計算
    ///
    /// # Formula
    /// V = (4/3)π × a × b × c
    ///
    /// # Returns
    /// 体積
    pub fn volume(&self) -> T {
        let pi = T::PI;
        let four_thirds = T::from_f64(4.0) / T::from_f64(3.0);
        four_thirds * pi * self.a_radius * self.b_radius * self.c_radius
    }

    /// 楕円体ソリッドの表面積を近似計算（Knudの近似式）
    ///
    /// # Formula
    /// S ≈ 4π × [(a^p × b^p + b^p × c^p + c^p × a^p) / 3]^(1/p)
    /// where p ≈ 1.6075
    ///
    /// # Returns
    /// 表面積の近似値
    pub fn surface_area(&self) -> T {
        let pi = T::PI;
        let p = T::from_f64(1.6075);
        let a = self.a_radius;
        let b = self.b_radius;
        let c = self.c_radius;

        // a^p, b^p, c^p を計算
        let a_p = a.powf(p);
        let b_p = b.powf(p);
        let c_p = c.powf(p);

        // (a^p × b^p + b^p × c^p + c^p × a^p) / 3
        let sum = (a_p * b_p + b_p * c_p + c_p * a_p) / T::from_f64(3.0);

        // 4π × sum^(1/p)
        T::from_f64(4.0) * pi * sum.powf(T::ONE / p)
    }

    /// 点が楕円体ソリッドの内部にあるかを判定
    ///
    /// # Arguments
    /// * `point` - 判定する点
    ///
    /// # Returns
    /// 内部にある場合は `true`、外部または表面の場合は `false`
    ///
    /// # Algorithm
    /// ローカル座標系に変換して (x/a)² + (y/b)² + (z/c)² ≤ 1 で判定
    pub fn contains_point(&self, point: &Point3D<T>) -> bool {
        // 中心からのオフセット
        let offset = *point - self.center;

        // ローカル座標軸
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis_internal().as_vector();
        let z_axis = self.axis.as_vector();

        // ローカル座標に投影
        let local_x = offset.dot(&x_axis);
        let local_y = offset.dot(&y_axis);
        let local_z = offset.dot(&z_axis);

        // 正規化された楕円体方程式
        let x_norm = local_x / self.a_radius;
        let y_norm = local_y / self.b_radius;
        let z_norm = local_z / self.c_radius;

        // (x/a)² + (y/b)² + (z/c)² ≤ 1
        let sum = x_norm * x_norm + y_norm * y_norm + z_norm * z_norm;
        sum <= T::ONE
    }

    /// 点が楕円体ソリッドの表面上にあるかを判定（許容誤差付き）
    ///
    /// # Arguments
    /// * `point` - 判定する点
    ///
    /// # Returns
    /// 表面上にある場合は `true`
    pub fn is_on_surface(&self, point: &Point3D<T>) -> bool {
        // 中心からのオフセット
        let offset = *point - self.center;

        // ローカル座標軸
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis_internal().as_vector();
        let z_axis = self.axis.as_vector();

        // ローカル座標に投影
        let local_x = offset.dot(&x_axis);
        let local_y = offset.dot(&y_axis);
        let local_z = offset.dot(&z_axis);

        let x_norm = local_x / self.a_radius;
        let y_norm = local_y / self.b_radius;
        let z_norm = local_z / self.c_radius;

        let sum = x_norm * x_norm + y_norm * y_norm + z_norm * z_norm;

        // 許容誤差内で 1 に等しいかチェック
        (sum - T::ONE).abs() < T::EPSILON * T::from_f64(10.0)
    }

    /// 点と楕円体ソリッド表面との距離を計算（近似）
    ///
    /// # Arguments
    /// * `point` - 判定する点
    ///
    /// # Returns
    /// 表面までの距離（近似値）
    pub fn distance_to_surface(&self, point: &Point3D<T>) -> T {
        let closest = self.closest_point_on_surface(point);
        let diff = *point - closest;
        (diff.x() * diff.x() + diff.y() * diff.y() + diff.z() * diff.z()).sqrt()
    }

    /// 指定点に最も近い表面上の点を取得（近似）
    ///
    /// # Arguments
    /// * `point` - 基準点
    ///
    /// # Returns
    /// 表面上の最近接点（近似）
    pub fn closest_point_on_surface(&self, point: &Point3D<T>) -> Point3D<T> {
        // 中心からのオフセット
        let offset = *point - self.center;

        // ローカル座標軸
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis_internal().as_vector();
        let z_axis = self.axis.as_vector();

        // ローカル座標に投影
        let local_x = offset.dot(&x_axis);
        let local_y = offset.dot(&y_axis);
        let local_z = offset.dot(&z_axis);

        // 正規化座標
        let x_norm = local_x / self.a_radius;
        let y_norm = local_y / self.b_radius;
        let z_norm = local_z / self.c_radius;

        // 原点からの距離
        let dist = (x_norm * x_norm + y_norm * y_norm + z_norm * z_norm).sqrt();

        if dist < T::EPSILON {
            // 中心点の場合はX軸上の点を返す
            return self.center + x_axis * self.a_radius;
        }

        // 表面上の点を計算（正規化ベクトルをスケール）
        let surface_x = local_x / dist;
        let surface_y = local_y / dist;
        let surface_z = local_z / dist;

        // ワールド座標系に変換
        self.center + x_axis * surface_x + y_axis * surface_y + z_axis * surface_z
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ellipsoidal_solid() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_dir = Vector3D::new(1.0, 0.0, 0.0);

        let ellipsoid = EllipsoidalSolid3D::new(center, axis, ref_dir, 2.0, 3.0, 4.0);
        assert!(ellipsoid.is_some());

        let e = ellipsoid.unwrap();
        assert_eq!(e.a_radius_internal(), 2.0);
        assert_eq!(e.b_radius_internal(), 3.0);
        assert_eq!(e.c_radius_internal(), 4.0);
    }

    #[test]
    fn test_new_standard_ellipsoidal_solid() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let ellipsoid = EllipsoidalSolid3D::new_standard(center, 2.0, 3.0, 4.0);

        assert!(ellipsoid.is_some());
        let e = ellipsoid.unwrap();
        assert_eq!(e.center_internal(), center);
    }

    #[test]
    fn test_volume() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // V = (4/3)π × 2 × 3 × 4 = 32π
        let expected = (4.0 / 3.0) * std::f64::consts::PI * 2.0 * 3.0 * 4.0;
        let volume = ellipsoid.volume();

        assert!((volume - expected).abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let ellipsoid = EllipsoidalSolid3D::new_at_origin(2.0, 3.0, 4.0).unwrap();

        // 中心点は内部
        assert!(ellipsoid.contains_point(&Point3D::new(0.0, 0.0, 0.0)));

        // X軸上の点（境界内）
        assert!(ellipsoid.contains_point(&Point3D::new(1.0, 0.0, 0.0)));

        // Y軸上の点（境界内）
        assert!(ellipsoid.contains_point(&Point3D::new(0.0, 2.0, 0.0)));

        // Z軸上の点（境界内）
        assert!(ellipsoid.contains_point(&Point3D::new(0.0, 0.0, 3.0)));

        // 外部の点
        assert!(!ellipsoid.contains_point(&Point3D::new(3.0, 0.0, 0.0)));
        assert!(!ellipsoid.contains_point(&Point3D::new(0.0, 4.0, 0.0)));
        assert!(!ellipsoid.contains_point(&Point3D::new(0.0, 0.0, 5.0)));
    }

    #[test]
    fn test_sphere_special_case() {
        // a = b = c = 5.0 の場合、球になる
        let sphere = EllipsoidalSolid3D::new_sphere(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            5.0,
        )
        .unwrap();

        // 体積は球の公式と一致するはず: V = (4/3)π × r³
        let expected_volume = (4.0 / 3.0) * std::f64::consts::PI * 5.0_f64.powi(3);
        assert!((sphere.volume() - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_invalid_radii() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_dir = Vector3D::new(1.0, 0.0, 0.0);

        // 負の半径
        assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, -1.0, 2.0, 3.0).is_none());
        assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, 1.0, -2.0, 3.0).is_none());
        assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, 1.0, 2.0, -3.0).is_none());

        // ゼロ半径
        assert!(EllipsoidalSolid3D::new(center, axis, ref_dir, 0.0, 2.0, 3.0).is_none());
    }
}

// ============================================================================
// Core Traits Implementation
// ============================================================================

use geo_foundation::{
    EllipsoidalSolid3DConstructor, EllipsoidalSolid3DCore, EllipsoidalSolid3DMeasure,
    EllipsoidalSolid3DProperties,
};

impl<T: Scalar> EllipsoidalSolid3DConstructor<T> for EllipsoidalSolid3D<T> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        a_radius: T,
        b_radius: T,
        c_radius: T,
    ) -> Option<Self> {
        Self::new(
            Point3D::new(center.0, center.1, center.2),
            Vector3D::new(axis.0, axis.1, axis.2),
            Vector3D::new(ref_direction.0, ref_direction.1, ref_direction.2),
            a_radius,
            b_radius,
            c_radius,
        )
    }

    fn new_standard(center: (T, T, T), a_radius: T, b_radius: T, c_radius: T) -> Option<Self> {
        Self::new_standard(
            Point3D::new(center.0, center.1, center.2),
            a_radius,
            b_radius,
            c_radius,
        )
    }

    fn unit_ellipsoid() -> Self {
        Self::new_standard(
            Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            T::ONE,
            T::ONE,
            T::ONE,
        )
        .unwrap()
    }

    fn from_radii(center: (T, T, T), a: T, b: T, c: T) -> Option<Self> {
        Self::new_standard(Point3D::new(center.0, center.1, center.2), a, b, c)
    }

    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self> {
        let center_x = (min.0 + max.0) / T::from_f64(2.0);
        let center_y = (min.1 + max.1) / T::from_f64(2.0);
        let center_z = (min.2 + max.2) / T::from_f64(2.0);

        let a = (max.0 - min.0) / T::from_f64(2.0);
        let b = (max.1 - min.1) / T::from_f64(2.0);
        let c = (max.2 - min.2) / T::from_f64(2.0);

        Self::new_standard(Point3D::new(center_x, center_y, center_z), a, b, c)
    }

    fn new_sphere(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
    ) -> Option<Self> {
        Self::new_sphere(
            Point3D::new(center.0, center.1, center.2),
            Vector3D::new(axis.0, axis.1, axis.2),
            Vector3D::new(ref_direction.0, ref_direction.1, ref_direction.2),
            radius,
        )
    }
}

impl<T: Scalar> EllipsoidalSolid3DProperties<T> for EllipsoidalSolid3D<T> {
    fn center(&self) -> (T, T, T) {
        let c = self.center_internal();
        (c.x(), c.y(), c.z())
    }

    fn a_radius(&self) -> T {
        self.a_radius_internal()
    }

    fn b_radius(&self) -> T {
        self.b_radius_internal()
    }

    fn c_radius(&self) -> T {
        self.c_radius_internal()
    }

    fn axis(&self) -> (T, T, T) {
        let a = self.axis_internal();
        (a.x(), a.y(), a.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        let r = self.ref_direction_internal();
        (r.x(), r.y(), r.z())
    }

    fn radii(&self) -> (T, T, T) {
        (self.a_radius, self.b_radius, self.c_radius)
    }

    fn is_sphere(&self) -> bool {
        let epsilon = T::EPSILON * T::from_f64(10.0);
        (self.a_radius - self.b_radius).abs() < epsilon
            && (self.b_radius - self.c_radius).abs() < epsilon
    }

    fn is_unit_ellipsoid(&self) -> bool {
        let epsilon = T::EPSILON * T::from_f64(10.0);
        (self.a_radius - T::ONE).abs() < epsilon
            && (self.b_radius - T::ONE).abs() < epsilon
            && (self.c_radius - T::ONE).abs() < epsilon
    }

    fn is_centered_at_origin(&self) -> bool {
        let epsilon = T::EPSILON * T::from_f64(10.0);
        let c = self.center_internal();
        c.x().abs() < epsilon && c.y().abs() < epsilon && c.z().abs() < epsilon
    }
}

impl<T: Scalar> EllipsoidalSolid3DMeasure<T> for EllipsoidalSolid3D<T> {
    fn volume(&self) -> T {
        self.volume()
    }

    fn surface_area(&self) -> T {
        self.surface_area()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        self.contains_point(&Point3D::new(point.0, point.1, point.2))
    }

    fn distance_to_surface(&self, point: (T, T, T)) -> T {
        self.distance_to_surface(&Point3D::new(point.0, point.1, point.2))
    }

    fn bounding_box(&self) -> ((T, T, T), (T, T, T)) {
        let bbox = self.bounding_box();
        let min = bbox.min();
        let max = bbox.max();
        ((min.x(), min.y(), min.z()), (max.x(), max.y(), max.z()))
    }

    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T) {
        let p = self.closest_point_on_surface(&Point3D::new(point.0, point.1, point.2));
        (p.x(), p.y(), p.z())
    }

    fn is_on_surface(&self, point: (T, T, T)) -> bool {
        self.is_on_surface(&Point3D::new(point.0, point.1, point.2))
    }

    fn is_degenerate(&self) -> bool {
        self.is_degenerate()
    }
}

impl<T: Scalar> EllipsoidalSolid3DCore<T> for EllipsoidalSolid3D<T> {}
