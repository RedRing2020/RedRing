//! 3次元円柱ソリッド（CylindricalSolid3D）のCore実装
//!
//! STEP準拠のSOLID_CYLINDER + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//! 拡張機能は cylindrical_solid_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! SOLID_CYLINDER('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS, HEIGHT);
//! ```
//! - location: 円柱底面中心（center）
//! - axis: Z軸方向（円柱軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 円柱半径
//! - height: 円柱高さ

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元円柱ソリッド（STEP準拠のCore実装）
///
/// STEP AP214の SOLID_CYLINDER + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：立体として体積・内部判定を持つ
///
/// ## 座標系定義（STEP準拠）
/// - center: 底面中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 円柱軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 円柱半径
/// - height: 円柱高さ
///
/// ## ソリッド特性
/// - 体積計算：V = π × r² × h
/// - 内部判定：点の包含テスト
/// - 表面積計算：底面 + 側面 + 上面
/// - 境界ボックス：軸方向を考慮した最小直方体
///
/// ## CAD用途
/// - パラメトリック円柱ソリッドの基準座標系
/// - ブーリアン演算（和・差・積）
/// - STEPファイルとの相互変換
/// - 体積・質量特性計算
#[derive(Debug, Clone, PartialEq)]
pub struct CylindricalSolid3D<T: Scalar> {
    /// 円柱の底面中心点（STEP: location）
    center: Point3D<T>,

    /// 円柱の軸方向（STEP: axis）- Z軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    /// axis と直交していなくても自動調整
    ref_direction: Direction3D<T>,

    /// 円柱の半径
    radius: T,

    /// 円柱の高さ
    height: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> CylindricalSolid3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で円柱ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 底面の中心点
    /// * `axis` - 軸方向ベクトル（円柱軸、Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `radius` - 半径（正の値）
    /// * `height` - 高さ（正の値）
    ///
    /// # Returns
    /// 有効な円柱ソリッドが作成できた場合は `Some(CylindricalSolid3D)`、
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
        radius: T,
        height: T,
    ) -> Option<Self> {
        // 半径と高さの検証
        if radius <= T::ZERO || height <= T::ZERO {
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
            radius,
            height,
        })
    }

    /// Z軸に平行な円柱ソリッドを作成（簡易コンストラクタ）
    pub fn new_z_axis(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
            height,
        )
    }

    /// Y軸に平行な円柱ソリッドを作成（簡易コンストラクタ）
    pub fn new_y_axis(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ONE, T::ZERO),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
            height,
        )
    }

    /// X軸に平行な円柱ソリッドを作成（簡易コンストラクタ）
    pub fn new_x_axis(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            Vector3D::new(T::ZERO, T::ONE, T::ZERO),
            radius,
            height,
        )
    }

    // ========================================================================
    // Core Accessor Methods (Internal use only)
    // ========================================================================

    /// 底面の中心点を取得（内部使用）
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// 円柱の軸方向を取得（正規化済み）
    pub fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    /// 参照方向を取得（正規化済み、X軸相当）
    pub fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
    }

    /// Y軸方向を計算（axis × ref_direction）
    pub fn y_axis(&self) -> Direction3D<T> {
        let y_vector = self.axis.as_vector().cross(&self.ref_direction.as_vector());
        Direction3D::from_vector(y_vector)
            .expect("Y-axis calculation should always succeed with orthogonal axes")
    }

    /// 円柱の半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 円柱の高さを取得
    pub fn height(&self) -> T {
        self.height
    }

    // ========================================================================
    // Core Geometric Properties (Internal use only)
    // ========================================================================

    /// 円柱ソリッドの体積を計算（内部使用）
    ///
    /// 体積 = π × r² × h
    pub(crate) fn volume_internal(&self) -> T {
        T::PI * self.radius * self.radius * self.height
    }

    /// 円柱ソリッドの表面積を計算（内部使用）
    ///
    /// 表面積 = 2π × r² + 2π × r × h (底面積 + 側面積)
    #[allow(dead_code)]
    pub(crate) fn surface_area_internal(&self) -> T {
        let base_area = T::PI * self.radius * self.radius;
        let side_area = T::from_f64(2.0) * T::PI * self.radius * self.height;
        T::from_f64(2.0) * base_area + side_area
    }

    /// 円柱ソリッドの境界ボックスを計算
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        // 各軸成分の最大伸び
        let axis_x = self.axis.x();
        let axis_y = self.axis.y();
        let axis_z = self.axis.z();

        let height_x = axis_x * self.height;
        let height_y = axis_y * self.height;
        let height_z = axis_z * self.height;

        // 円形断面の各軸方向への最大伸び
        let radius_x = self.radius * (T::ONE - axis_x * axis_x).sqrt();
        let radius_y = self.radius * (T::ONE - axis_y * axis_y).sqrt();
        let radius_z = self.radius * (T::ONE - axis_z * axis_z).sqrt();

        let min_x = (self.center.x() - radius_x).min(self.center.x() + height_x - radius_x);
        let max_x = (self.center.x() + radius_x).max(self.center.x() + height_x + radius_x);

        let min_y = (self.center.y() - radius_y).min(self.center.y() + height_y - radius_y);
        let max_y = (self.center.y() + radius_y).max(self.center.y() + height_y + radius_y);

        let min_z = (self.center.z() - radius_z).min(self.center.z() + height_z - radius_z);
        let max_z = (self.center.z() + radius_z).max(self.center.z() + height_z + radius_z);

        geo_core::Aabb3D::new(
            geo_core::Point3D::new(min_x, min_y, min_z),
            geo_core::Point3D::new(max_x, max_y, max_z),
        )
    }

    // ========================================================================
    // Core Containment and Distance Methods (Internal use only)
    // ========================================================================

    /// 点が円柱ソリッド内部に含まれるかを判定（内部使用）
    #[allow(dead_code)]
    pub(crate) fn contains_point_internal(&self, point: Point3D<T>) -> bool {
        // 点から底面への投影を計算
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let axis_projection = to_point.dot(&self.axis.as_vector());

        // 高さ範囲の確認
        if axis_projection < T::ZERO || axis_projection > self.height {
            return false;
        }

        // 半径範囲の確認
        let axis_component = Vector3D::new(
            self.axis.x() * axis_projection,
            self.axis.y() * axis_projection,
            self.axis.z() * axis_projection,
        );
        let radial_component = Vector3D::new(
            to_point.x() - axis_component.x(),
            to_point.y() - axis_component.y(),
            to_point.z() - axis_component.z(),
        );
        let radial_distance = radial_component.magnitude();

        radial_distance <= self.radius
    }

    /// 点から円柱ソリッド表面までの距離を計算（内部使用）
    #[allow(dead_code)]
    pub(crate) fn distance_to_surface_internal(&self, point: Point3D<T>) -> T {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let axis_projection = to_point.dot(&self.axis.as_vector());

        // 軸方向の距離
        let axis_distance = if axis_projection < T::ZERO {
            -axis_projection
        } else if axis_projection > self.height {
            axis_projection - self.height
        } else {
            T::ZERO
        };

        // 半径方向の距離
        let axis_component = Vector3D::new(
            self.axis.x() * axis_projection.max(T::ZERO).min(self.height),
            self.axis.y() * axis_projection.max(T::ZERO).min(self.height),
            self.axis.z() * axis_projection.max(T::ZERO).min(self.height),
        );
        let radial_component = Vector3D::new(
            to_point.x() - axis_component.x(),
            to_point.y() - axis_component.y(),
            to_point.z() - axis_component.z(),
        );
        let radial_distance = radial_component.magnitude();
        let radial_excess = (radial_distance - self.radius).max(T::ZERO);

        // 軸方向と半径方向の距離を合成
        (axis_distance * axis_distance + radial_excess * radial_excess).sqrt()
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for CylindricalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CylindricalSolid3D(center: {:?}, axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {}, height: {})",
            self.center,
            self.axis.x(),
            self.axis.y(),
            self.axis.z(),
            self.ref_direction.x(),
            self.ref_direction.y(),
            self.ref_direction.z(),
            self.radius,
            self.height
        )
    }
}

// ============================================================================
// ============================================================================
// Core Traits Implementation (Foundation Pattern)
// ============================================================================

use geo_foundation::{
    CylindricalSolid3DConstructor, CylindricalSolid3DCore, CylindricalSolid3DMeasure,
    CylindricalSolid3DProperties,
};

impl<T: Scalar> CylindricalSolid3DConstructor<T> for CylindricalSolid3D<T> {
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_vector = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_vector = Vector3D::new(ref_direction.0, ref_direction.1, ref_direction.2);
        Self::new(center_point, axis_vector, ref_vector, radius, height)
    }

    fn new_standard(center: (T, T, T), radius: T, height: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_z_axis(center_point, radius, height)
    }

    fn unit_cylinder() -> Self {
        Self::new_z_axis(Point3D::origin(), T::ONE, T::from_f64(2.0))
            .expect("Unit cylinder should always be valid")
    }

    // Phase 2: 追加コンストラクタ

    fn from_axis_and_radius(
        start_point: (T, T, T),
        end_point: (T, T, T),
        radius: T,
    ) -> Option<Self> {
        let p1 = Point3D::new(start_point.0, start_point.1, start_point.2);
        let p2 = Point3D::new(end_point.0, end_point.1, end_point.2);
        let axis_vec = Vector3D::from_points(&p1, &p2);
        let height = axis_vec.length();

        if height < T::EPSILON {
            return None;
        }

        let axis_dir = axis_vec / height;
        let ref_dir = if axis_dir.z().abs() < T::from_f64(0.9) {
            Vector3D::new(T::ZERO, T::ZERO, T::ONE)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO)
        };

        let center_point = Point3D::new(start_point.0, start_point.1, start_point.2);

        Self::new(center_point, axis_dir, ref_dir, radius, height)
    }

    fn from_diameter(center: (T, T, T), _axis: (T, T, T), diameter: T, height: T) -> Option<Self> {
        let radius = diameter / (T::ONE + T::ONE);
        Self::new_standard(center, radius, height)
    }

    fn from_two_points_and_radius(p1: (T, T, T), p2: (T, T, T), radius: T) -> Option<Self> {
        Self::from_axis_and_radius(p1, p2, radius)
    }
}

impl<T: Scalar> CylindricalSolid3DProperties<T> for CylindricalSolid3D<T> {
    fn center(&self) -> (T, T, T) {
        (self.center.x(), self.center.y(), self.center.z())
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn height(&self) -> T {
        self.height
    }

    fn axis(&self) -> (T, T, T) {
        (self.axis.x(), self.axis.y(), self.axis.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        (
            self.ref_direction.x(),
            self.ref_direction.y(),
            self.ref_direction.z(),
        )
    }

    fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    // Phase 2: 追加プロパティ

    fn top_center(&self) -> (T, T, T) {
        let top_x = self.center.x() + self.axis.x() * self.height;
        let top_y = self.center.y() + self.axis.y() * self.height;
        let top_z = self.center.z() + self.axis.z() * self.height;
        (top_x, top_y, top_z)
    }

    fn lateral_surface_area(&self) -> T {
        T::TAU * self.radius * self.height
    }

    fn base_area(&self) -> T {
        T::PI * self.radius * self.radius
    }
}

impl<T: Scalar> CylindricalSolid3DMeasure<T> for CylindricalSolid3D<T> {
    fn volume(&self) -> T {
        T::PI * self.radius * self.radius * self.height
    }

    fn surface_area(&self) -> T {
        let base_area = T::PI * self.radius * self.radius;
        let side_area = T::from_f64(2.0) * T::PI * self.radius * self.height;
        T::from_f64(2.0) * base_area + side_area
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        // 点から底面への投影を計算
        let to_point_x = point.0 - self.center.x();
        let to_point_y = point.1 - self.center.y();
        let to_point_z = point.2 - self.center.z();

        let axis_projection =
            to_point_x * self.axis.x() + to_point_y * self.axis.y() + to_point_z * self.axis.z();

        // 高さ範囲の確認
        if axis_projection < T::ZERO || axis_projection > self.height {
            return false;
        }

        // 半径範囲の確認
        let axis_comp_x = self.axis.x() * axis_projection;
        let axis_comp_y = self.axis.y() * axis_projection;
        let axis_comp_z = self.axis.z() * axis_projection;

        let radial_x = to_point_x - axis_comp_x;
        let radial_y = to_point_y - axis_comp_y;
        let radial_z = to_point_z - axis_comp_z;

        let radial_distance_sq = radial_x * radial_x + radial_y * radial_y + radial_z * radial_z;

        radial_distance_sq <= self.radius * self.radius
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let to_point_x = point.0 - self.center.x();
        let to_point_y = point.1 - self.center.y();
        let to_point_z = point.2 - self.center.z();

        let axis_projection =
            to_point_x * self.axis.x() + to_point_y * self.axis.y() + to_point_z * self.axis.z();

        // 軸方向の距離
        let axis_distance = if axis_projection < T::ZERO {
            -axis_projection
        } else if axis_projection > self.height {
            axis_projection - self.height
        } else {
            T::ZERO
        };

        // 半径方向の距離
        let clamped_projection = axis_projection.max(T::ZERO).min(self.height);
        let axis_comp_x = self.axis.x() * clamped_projection;
        let axis_comp_y = self.axis.y() * clamped_projection;
        let axis_comp_z = self.axis.z() * clamped_projection;

        let radial_x = to_point_x - axis_comp_x;
        let radial_y = to_point_y - axis_comp_y;
        let radial_z = to_point_z - axis_comp_z;

        let radial_distance =
            (radial_x * radial_x + radial_y * radial_y + radial_z * radial_z).sqrt();
        let radial_excess = (radial_distance - self.radius).max(T::ZERO);

        // 軸方向と半径方向の距離を合成
        (axis_distance * axis_distance + radial_excess * radial_excess).sqrt()
    }

    // Phase 2: 追加測定

    fn point_at_cylindrical(&self, r: T, theta: T, z: T) -> (T, T, T) {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        let x = self.center.x()
            + r * cos_theta * x_axis.x()
            + r * sin_theta * y_axis.x()
            + z * z_axis.x();
        let y = self.center.y()
            + r * cos_theta * x_axis.y()
            + r * sin_theta * y_axis.y()
            + z * z_axis.y();
        let z_coord = self.center.z()
            + r * cos_theta * x_axis.z()
            + r * sin_theta * y_axis.z()
            + z * z_axis.z();

        (x, y, z_coord)
    }

    fn bounding_box(&self) -> ((T, T, T), (T, T, T)) {
        let r = self.radius;
        let h = self.height;

        let min_x = self.center.x() - r;
        let max_x = self.center.x() + r;
        let min_y = self.center.y() - r;
        let max_y = self.center.y() + r;

        let base_z = self.center.z();
        let top_z = base_z + h * self.axis.z();
        let min_z = base_z.min(top_z);
        let max_z = base_z.max(top_z);

        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }

    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T) {
        let to_point_x = point.0 - self.center.x();
        let to_point_y = point.1 - self.center.y();
        let to_point_z = point.2 - self.center.z();

        let axis_projection =
            to_point_x * self.axis.x() + to_point_y * self.axis.y() + to_point_z * self.axis.z();

        let clamped_h = axis_projection.max(T::ZERO).min(self.height);

        let axis_comp_x = self.axis.x() * clamped_h;
        let axis_comp_y = self.axis.y() * clamped_h;
        let axis_comp_z = self.axis.z() * clamped_h;

        let radial_x = to_point_x - axis_comp_x;
        let radial_y = to_point_y - axis_comp_y;
        let radial_z = to_point_z - axis_comp_z;

        let radial_len = (radial_x * radial_x + radial_y * radial_y + radial_z * radial_z).sqrt();

        if radial_len > T::EPSILON {
            let scale = self.radius / radial_len;
            let surface_x = self.center.x() + axis_comp_x + radial_x * scale;
            let surface_y = self.center.y() + axis_comp_y + radial_y * scale;
            let surface_z = self.center.z() + axis_comp_z + radial_z * scale;
            (surface_x, surface_y, surface_z)
        } else {
            (
                self.center.x() + axis_comp_x + self.radius,
                self.center.y() + axis_comp_y,
                self.center.z() + axis_comp_z,
            )
        }
    }

    fn intersects_line(&self, _line_point: (T, T, T), _line_dir: (T, T, T)) -> Option<(T, T, T)> {
        None
    }
}

impl<T: Scalar> CylindricalSolid3DCore<T> for CylindricalSolid3D<T> {}

// ============================================================================
// Backward Compatibility (移行期間中のみ)
// ============================================================================

/// 旧名前との互換性のためのエイリアス
/// 将来のバージョンで削除予定
#[deprecated(since = "0.1.0", note = "Use CylindricalSolid3D instead")]
pub type Cylinder3D<T> = CylindricalSolid3D<T>;
