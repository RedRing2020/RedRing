//! 3次元円錐ソリッド（ConicalSolid3D）のCore実装
//!
//! STEP準拠のSOLID_CONE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//! 拡張機能は conical_solid_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! SOLID_CONE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS, HEIGHT);
//! ```
//! - location: 円錐の底面中心点（center）
//! - axis: Z軸方向（底面から頂点への軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 底面の半径
//! - height: 円錐の高さ（軸方向の長さ）
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::{Direction3D, Point3D, Vector3D};

use geo_foundation::Scalar;

/// 3次元円錐ソリッド（STEP準拠のCore実装）
///
/// STEP AP214の SOLID_CONE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：立体として体積・内部判定を持つ
///
/// ## 座標系定義（STEP準拠）
/// - center: 円錐の底面中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 底面から頂点への軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 底面の半径
/// - height: 円錐の高さ（軸方向の長さ）
///
/// ## ソリッド特性
/// - 体積計算：V = (1/3)π × r² × h
/// - 内部判定：点の包含テスト（円錐内部の判定）
/// - 表面積計算：S = π × r × (r + √(r² + h²))
/// - 境界ボックス：底面と頂点を包含する直方体
///
/// ## CAD用途
/// - パラメトリック円錐ソリッドの基準座標系
/// - ブーリアン演算（和・差・積）
/// - STEPファイルとの相互変換
/// - 体積・質量特性計算
/// - 工業製品の円錐部品モデリング
#[derive(Debug, Clone, PartialEq)]
pub struct ConicalSolid3D<T: Scalar> {
    /// 円錐の底面中心点（STEP: location）
    center: Point3D<T>,

    /// 軸方向 - 底面から頂点への方向（STEP: axis）
    /// Direction3D<T>により正規化が保証される
    axis: Direction3D<T>,

    /// 参照方向 - X軸方向（STEP: ref_direction）
    /// Direction3D<T>により正規化が保証される
    ref_direction: Direction3D<T>,

    /// 底面の半径（正の値）
    radius: T,

    /// 円錐の高さ（軸方向の長さ、正の値）
    height: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> ConicalSolid3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で円錐ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 円錐の底面中心点
    /// * `axis` - 軸ベクトル（底面から頂点への方向）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `radius` - 底面の半径（正の値）
    /// * `height` - 円錐の高さ（正の値）
    ///
    /// # Returns
    /// 有効な円錐ソリッドが作成できた場合は `Some(ConicalSolid3D)`、
    /// 軸がゼロベクトル、参照方向がゼロベクトル、半径または高さが負・ゼロの場合は `None`
    ///
    /// # Algorithm
    /// 1. 軸と参照方向を正規化
    /// 2. グラム・シュミット正規直交化で軸に直交する参照方向を計算
    /// 3. 半径と高さの有効性をチェック
    /// 4. STEP準拠の円錐ソリッドを構築
    pub fn new(
        center: Point3D<T>,
        axis: Vector3D<T>,
        ref_direction: Vector3D<T>,
        radius: T,
        height: T,
    ) -> Option<Self> {
        // 半径と高さの有効性チェック
        if radius <= T::ZERO || height <= T::ZERO {
            return None;
        }

        // 軸の正規化
        let normalized_axis = Direction3D::from_vector(axis)?;

        // 参照方向の軸成分を除去して正規化（グラム・シュミット正規直交化）
        let axis_component = ref_direction.dot(&normalized_axis.as_vector());
        let orthogonal_ref = ref_direction - normalized_axis.as_vector() * axis_component;
        let normalized_ref_direction = Direction3D::from_vector(orthogonal_ref)?;

        Some(Self {
            center,
            axis: normalized_axis,
            ref_direction: normalized_ref_direction,
            radius,
            height,
        })
    }

    /// Z軸標準の円錐ソリッドを作成（簡易コンストラクタ）
    ///
    /// # Arguments
    /// * `center` - 円錐の底面中心点
    /// * `radius` - 底面の半径
    /// * `height` - 円錐の高さ
    ///
    /// # Returns
    /// 標準的な円錐ソリッド（Z軸方向、X軸参照方向）
    pub fn new_standard(center: Point3D<T>, radius: T, height: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
            height,
        )
    }

    /// 原点中心の円錐ソリッドを作成（簡易コンストラクタ）
    ///
    /// # Arguments
    /// * `radius` - 底面の半径
    /// * `height` - 円錐の高さ
    ///
    /// # Returns
    /// 原点中心の標準的な円錐ソリッド
    pub fn new_at_origin(radius: T, height: T) -> Option<Self> {
        Self::new_standard(Point3D::new(T::ZERO, T::ZERO, T::ZERO), radius, height)
    }

    /// 頂点と底面から円錐ソリッドを作成
    ///
    /// # Arguments
    /// * `apex` - 円錐の頂点
    /// * `base_center` - 底面の中心点
    /// * `ref_direction` - 参照方向ベクトル
    /// * `radius` - 底面の半径
    ///
    /// # Returns
    /// 頂点と底面から定義された円錐ソリッド
    pub fn from_apex_and_base(
        apex: Point3D<T>,
        base_center: Point3D<T>,
        ref_direction: Vector3D<T>,
        radius: T,
    ) -> Option<Self> {
        // 底面から頂点への軸ベクトルを計算
        let axis = Vector3D::new(
            apex.x() - base_center.x(),
            apex.y() - base_center.y(),
            apex.z() - base_center.z(),
        );
        let height = axis.length();

        Self::new(base_center, axis, ref_direction, radius, height)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 円錐の底面中心点を取得
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// 軸方向（底面から頂点への正規化ベクトル）を取得
    pub(crate) fn axis_internal(&self) -> Direction3D<T> {
        self.axis
    }

    /// 参照方向（X軸の正規化ベクトル）を取得
    pub(crate) fn ref_direction_internal(&self) -> Direction3D<T> {
        self.ref_direction
    }

    /// 底面の半径を取得
    pub(crate) fn radius_internal(&self) -> T {
        self.radius
    }

    /// 円錐の高さを取得
    pub(crate) fn height_internal(&self) -> T {
        self.height
    }

    /// 円錐の頂点座標を取得
    ///
    /// # Returns
    /// 底面中心から軸方向に高さ分移動した点
    pub(crate) fn apex_internal(&self) -> Point3D<T> {
        Point3D::new(
            self.center.x() + self.axis.x() * self.height,
            self.center.y() + self.axis.y() * self.height,
            self.center.z() + self.axis.z() * self.height,
        )
    }

    /// 導出Y軸方向を取得（axis × ref_direction）
    ///
    /// # Returns
    /// 右手系座標系のY軸方向
    pub(crate) fn derived_y_axis_internal(&self) -> Direction3D<T> {
        let y_vector = self.axis.as_vector().cross(&self.ref_direction.as_vector());
        Direction3D::from_vector(y_vector).unwrap() // 直交ベクトルなので常に成功
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// 円錐の体積を計算
    ///
    /// # Returns
    /// 体積 V = (1/3)π × r² × h
    #[allow(dead_code)]
    pub(crate) fn volume_internal(&self) -> T {
        T::PI * self.radius_internal() * self.radius_internal() * self.height_internal()
            / T::from_f64(3.0)
    }

    /// 円錐の表面積を計算（底面含む）
    ///
    /// # Returns
    /// 表面積 S = π × r × (r + √(r² + h²))
    #[allow(dead_code)]
    pub(crate) fn surface_area_internal(&self) -> T {
        let slant_height = (self.radius_internal() * self.radius_internal()
            + self.height_internal() * self.height_internal())
        .sqrt();
        T::PI * self.radius_internal() * (self.radius_internal() + slant_height)
    }

    /// 円錐の境界ボックスを計算
    ///
    /// # Returns
    /// 円錐を完全に包含する境界ボックス
    ///
    /// # Algorithm
    /// 1. 底面の円の境界を軸に垂直な平面で計算
    /// 2. 頂点座標を考慮
    /// 3. 全体を包含する境界ボックスを構築
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        // 軸に垂直なベクトル（参照方向とY軸）を取得
        let x_dir = self.ref_direction.as_vector();
        let y_dir = self.derived_y_axis_internal().as_vector();

        // 底面の円周上の極値を計算
        let radius_x: Vector3D<T> = x_dir * self.radius;
        let radius_y: Vector3D<T> = y_dir * self.radius;

        // 底面の境界点（座標レベルで計算して型推論問題を回避）
        let center_vec = self.center.to_vector();
        let base_bounds = [
            Point3D::new(
                center_vec.x() + radius_x.x(),
                center_vec.y() + radius_x.y(),
                center_vec.z() + radius_x.z(),
            ),
            Point3D::new(
                center_vec.x() - radius_x.x(),
                center_vec.y() - radius_x.y(),
                center_vec.z() - radius_x.z(),
            ),
            Point3D::new(
                center_vec.x() + radius_y.x(),
                center_vec.y() + radius_y.y(),
                center_vec.z() + radius_y.z(),
            ),
            Point3D::new(
                center_vec.x() - radius_y.x(),
                center_vec.y() - radius_y.y(),
                center_vec.z() - radius_y.z(),
            ),
        ];

        // 頂点
        let apex = self.apex_internal();

        // 全ての点の最小・最大値を計算
        let mut min_x = self.center.x();
        let mut max_x = self.center.x();
        let mut min_y = self.center.y();
        let mut max_y = self.center.y();
        let mut min_z = self.center.z();
        let mut max_z = self.center.z();

        // 底面の境界点を考慮
        for point in &base_bounds {
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
            min_z = min_z.min(point.z());
            max_z = max_z.max(point.z());
        }

        // 頂点を考慮
        min_x = min_x.min(apex.x());
        max_x = max_x.max(apex.x());
        min_y = min_y.min(apex.y());
        max_y = max_y.max(apex.y());
        min_z = min_z.min(apex.z());
        max_z = max_z.max(apex.z());

        geo_core::Aabb3D::new(
            analysis::Point3::new(min_x, min_y, min_z),
            analysis::Point3::new(max_x, max_y, max_z),
        )
    }

    // ========================================================================
    // Core Validation Methods
    // ========================================================================

    /// 円錐が有効かチェック
    ///
    /// # Returns
    /// 軸と参照方向が正規化されており、半径と高さが正の値の場合 true
    pub fn is_valid(&self) -> bool {
        let axis_length = self.axis.as_vector().length();
        let ref_length = self.ref_direction.as_vector().length();

        (axis_length - T::ONE).abs() < T::from_f64(1e-10)
            && (ref_length - T::ONE).abs() < T::from_f64(1e-10)
            && self.radius > T::ZERO
            && self.height > T::ZERO
    }
}

// ============================================================================
// Core Traits Implementation (Foundation Pattern)
// ============================================================================

use geo_foundation::{
    ConicalSolid3DConstructor, ConicalSolid3DCore, ConicalSolid3DMeasure, ConicalSolid3DProperties,
};

impl<T: Scalar> ConicalSolid3DConstructor<T> for ConicalSolid3D<T> {
    fn new(
        apex: (T, T, T),
        _base_center: (T, T, T), // 未使用
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self> {
        let apex_point = Point3D::new(apex.0, apex.1, apex.2);
        let axis_vector = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_vector = Vector3D::new(ref_direction.0, ref_direction.1, ref_direction.2);
        Self::new(apex_point, axis_vector, ref_vector, radius, height)
    }

    fn new_standard(base_center: (T, T, T), radius: T, height: T) -> Option<Self> {
        let center_point = Point3D::new(base_center.0, base_center.1, base_center.2);
        Self::new_standard(center_point, radius, height)
    }

    fn unit_cone() -> Self {
        Self::new_at_origin(T::ONE, T::from_f64(2.0)).expect("Unit cone should always be valid")
    }

    fn from_apex_and_base_circle(
        apex: (T, T, T),
        base_center: (T, T, T),
        radius: T,
    ) -> Option<Self> {
        let apex_point = Point3D::new(apex.0, apex.1, apex.2);
        let base_point = Point3D::new(base_center.0, base_center.1, base_center.2);
        let axis_vector = Vector3D::new(
            base_point.x() - apex_point.x(),
            base_point.y() - apex_point.y(),
            base_point.z() - apex_point.z(),
        );
        let height = axis_vector.length();
        let ref_direction = if axis_vector.z().abs() < T::from_f64(0.99) {
            Vector3D::new(T::ZERO, T::ZERO, T::ONE).cross(&axis_vector)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO).cross(&axis_vector)
        };
        Self::new(apex_point, axis_vector, ref_direction, radius, height)
    }

    fn from_apex_angle(apex: (T, T, T), axis: (T, T, T), half_angle: T, height: T) -> Option<Self> {
        let radius = height * half_angle.tan();
        let apex_point = Point3D::new(apex.0, apex.1, apex.2);
        let axis_vector = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_direction = if axis_vector.z().abs() < T::from_f64(0.99) {
            Vector3D::new(T::ZERO, T::ZERO, T::ONE).cross(&axis_vector)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO).cross(&axis_vector)
        };
        Self::new(apex_point, axis_vector, ref_direction, radius, height)
    }

    fn frustum(base_center: (T, T, T), axis: (T, T, T), base_radius: T, height: T) -> Option<Self> {
        let center_point = Point3D::new(base_center.0, base_center.1, base_center.2);
        let axis_vector = Vector3D::new(axis.0, axis.1, axis.2);
        let ref_direction = if axis_vector.z().abs() < T::from_f64(0.99) {
            Vector3D::new(T::ZERO, T::ZERO, T::ONE).cross(&axis_vector)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO).cross(&axis_vector)
        };
        Self::new(
            center_point,
            axis_vector,
            ref_direction,
            base_radius,
            height,
        )
    }
}

impl<T: Scalar> ConicalSolid3DProperties<T> for ConicalSolid3D<T> {
    fn apex(&self) -> (T, T, T) {
        let a = self.apex_internal();
        (a.x(), a.y(), a.z())
    }

    fn base_center(&self) -> (T, T, T) {
        let b = self.center_internal();
        (b.x(), b.y(), b.z())
    }

    fn radius(&self) -> T {
        self.radius_internal()
    }

    fn height(&self) -> T {
        self.height_internal()
    }

    fn axis(&self) -> (T, T, T) {
        let a = self.axis_internal();
        (a.x(), a.y(), a.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        let r = self.ref_direction_internal();
        (r.x(), r.y(), r.z())
    }

    fn slant_height(&self) -> T {
        let r = self.radius_internal();
        let h = self.height_internal();
        (r * r + h * h).sqrt()
    }

    fn half_angle(&self) -> T {
        (self.radius_internal() / self.height_internal()).atan()
    }

    fn lateral_surface_area(&self) -> T {
        T::PI * self.radius_internal() * self.slant_height()
    }

    fn base_area(&self) -> T {
        T::PI * self.radius_internal() * self.radius_internal()
    }
}

impl<T: Scalar> ConicalSolid3DMeasure<T> for ConicalSolid3D<T> {
    fn volume(&self) -> T {
        self.volume_internal()
    }

    fn surface_area(&self) -> T {
        self.surface_area_internal()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let point_3d = Point3D::new(point.0, point.1, point.2);
        self.contains_point(point_3d)
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let point_3d = Point3D::new(point.0, point.1, point.2);
        self.distance_to_surface(point_3d).abs()
    }

    fn point_at_conical(&self, r: T, theta: T, z: T) -> (T, T, T) {
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let x_axis = self.ref_direction_internal().as_vector();
        let z_axis = self.axis_internal().as_vector();
        let y_axis = z_axis.cross(&x_axis);

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
        let r = self.radius_internal();
        let h = self.height_internal();

        let min_x = self.center.x() - r;
        let max_x = self.center.x() + r;
        let min_y = self.center.y() - r;
        let max_y = self.center.y() + r;

        let base_z = self.center.z();
        let apex_z = base_z + h * self.axis_internal().z();
        let min_z = base_z.min(apex_z);
        let max_z = base_z.max(apex_z);

        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }

    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T) {
        let apex = self.apex_internal();
        let to_point_x = point.0 - apex.x();
        let to_point_y = point.1 - apex.y();
        let to_point_z = point.2 - apex.z();

        let axis_projection = to_point_x * self.axis_internal().x()
            + to_point_y * self.axis_internal().y()
            + to_point_z * self.axis_internal().z();

        if axis_projection <= T::ZERO {
            return (apex.x(), apex.y(), apex.z());
        }

        if axis_projection >= self.height_internal() {
            let clamped_h = self.height_internal();
            let axis_comp_x = self.axis_internal().x() * clamped_h;
            let axis_comp_y = self.axis_internal().y() * clamped_h;
            let axis_comp_z = self.axis_internal().z() * clamped_h;

            let radial_x = to_point_x - axis_comp_x;
            let radial_y = to_point_y - axis_comp_y;
            let radial_z = to_point_z - axis_comp_z;

            let radial_len =
                (radial_x * radial_x + radial_y * radial_y + radial_z * radial_z).sqrt();

            if radial_len > T::EPSILON {
                let scale = self.radius_internal() / radial_len;
                (
                    apex.x() + axis_comp_x + radial_x * scale,
                    apex.y() + axis_comp_y + radial_y * scale,
                    apex.z() + axis_comp_z + radial_z * scale,
                )
            } else {
                (
                    apex.x() + axis_comp_x + self.radius_internal(),
                    apex.y() + axis_comp_y,
                    apex.z() + axis_comp_z,
                )
            }
        } else {
            let current_radius =
                self.radius_internal() * (T::ONE - axis_projection / self.height_internal());
            let axis_comp_x = self.axis_internal().x() * axis_projection;
            let axis_comp_y = self.axis_internal().y() * axis_projection;
            let axis_comp_z = self.axis_internal().z() * axis_projection;

            let radial_x = to_point_x - axis_comp_x;
            let radial_y = to_point_y - axis_comp_y;
            let radial_z = to_point_z - axis_comp_z;

            let radial_len =
                (radial_x * radial_x + radial_y * radial_y + radial_z * radial_z).sqrt();

            if radial_len > T::EPSILON {
                let scale = current_radius / radial_len;
                (
                    apex.x() + axis_comp_x + radial_x * scale,
                    apex.y() + axis_comp_y + radial_y * scale,
                    apex.z() + axis_comp_z + radial_z * scale,
                )
            } else {
                (
                    apex.x() + axis_comp_x + current_radius,
                    apex.y() + axis_comp_y,
                    apex.z() + axis_comp_z,
                )
            }
        }
    }

    fn contains_point_tolerance(&self, point: (T, T, T), tolerance: T) -> bool {
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }
}

impl<T: Scalar> ConicalSolid3DCore<T> for ConicalSolid3D<T> {}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for ConicalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ConicalSolid3D(center: ({}, {}, {}), axis: ({}, {}, {}), radius: {}, height: {})",
            self.center.x(),
            self.center.y(),
            self.center.z(),
            self.axis.x(),
            self.axis.y(),
            self.axis.z(),
            self.radius,
            self.height
        )
    }
}

// ============================================================================
// Backward Compatibility (移行期間中のみ)
// ============================================================================

/// 旧名前との互換性のためのエイリアス
/// 将来的には削除予定
pub type Cone3D<T> = ConicalSolid3D<T>;
