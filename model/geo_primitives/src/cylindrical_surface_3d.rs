//! 3次元円柱サーフェス（CylindricalSurface3D）のCore実装
//!
//! STEP準拠のCYLINDRICAL_SURFACE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//! 拡張機能は cylindrical_surface_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! CYLINDRICAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS);
//! ```
//! - location: 円柱軸上の基準点（center）
//! - axis: Z軸方向（円柱軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 円柱半径
//! - 高さは境界によって定義（無限円柱面）

use crate::{BBox3D, Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元円柱サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の CYLINDRICAL_SURFACE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 円柱軸上の基準点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 円柱軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 円柱半径
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 円周方向パラメータ（角度）
/// - v ∈ ℝ: 軸方向パラメータ（無限範囲、境界で制限）
/// - Point(u, v) = center + radius * (cos(u) * X + sin(u) * Y) + v * Z
///
/// ## サーフェス特性
/// - 厚みなし：純粋な2次元多様体
/// - パラメータ化：連続的なUV座標系
/// - 法線計算：各点での垂直ベクトル
/// - 曲率解析：主曲率・平均曲率・ガウス曲率
/// - 境界操作：トリム・分割・結合
///
/// ## CAD用途
/// - 表面解析・品質評価
/// - CAM工具経路生成
/// - レンダリング・テクスチャマッピング
/// - NURBS変換・高精度表現
#[derive(Debug, Clone, PartialEq)]
pub struct CylindricalSurface3D<T: Scalar> {
    /// 円柱軸上の基準点（STEP: location）
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
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> CylindricalSurface3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で円柱サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 軸上の基準点
    /// * `axis` - 軸方向ベクトル（円柱軸、Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `radius` - 半径（正の値）
    ///
    /// # Returns
    /// 有効な円柱サーフェスが作成できた場合は `Some(CylindricalSurface3D)`、
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
    ) -> Option<Self> {
        // 半径の検証
        if radius <= T::ZERO {
            return None;
        }

        // 軸ベクトルの正規化
        let z_axis = Direction3D::from_vector(axis)?;

        // 参照方向の直交化・正規化（グラム・シュミット正規直交化）
        let axis_component = ref_direction.dot(&z_axis.as_vector());
        let orthogonal_ref = ref_direction - z_axis.as_vector() * axis_component;
        let x_axis = Direction3D::from_vector(orthogonal_ref)?;

        Some(Self {
            center,
            axis: z_axis,
            ref_direction: x_axis,
            radius,
        })
    }

    /// Z軸に平行な円柱サーフェスを作成（簡易コンストラクタ）
    pub fn new_z_axis(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE), // Z軸
            Vector3D::new(T::ONE, T::ZERO, T::ZERO), // X軸を参照方向
            radius,
        )
    }

    /// Y軸に平行な円柱サーフェスを作成（簡易コンストラクタ）
    pub fn new_y_axis(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ONE, T::ZERO), // Y軸
            Vector3D::new(T::ONE, T::ZERO, T::ZERO), // X軸を参照方向
            radius,
        )
    }

    /// X軸に平行な円柱サーフェスを作成（簡易コンストラクタ）
    pub fn new_x_axis(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ONE, T::ZERO, T::ZERO), // X軸
            Vector3D::new(T::ZERO, T::ONE, T::ZERO), // Y軸を参照方向
            radius,
        )
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 軸上の基準点を取得
    pub fn center(&self) -> Point3D<T> {
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

    // ========================================================================
    // Core Parametric Surface Methods (サーフェス特性)
    // ========================================================================

    /// パラメータ (u, v) から3D点を計算
    ///
    /// # Parameters
    /// * `u` - 円周方向パラメータ [0, 2π]
    /// * `v` - 軸方向パラメータ [-∞, +∞]
    ///
    /// # Returns
    /// サーフェス上の3D点
    ///
    /// # Formula
    /// Point(u, v) = center + radius * (cos(u) * X + sin(u) * Y) + v * Z
    pub fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 円周方向の点
        let radial_point = x_axis * (cos_u * self.radius) + y_axis * (sin_u * self.radius);

        // 軸方向のオフセット
        let axial_offset = z_axis * v;

        Point3D::new(
            self.center.x() + radial_point.x() + axial_offset.x(),
            self.center.y() + radial_point.y() + axial_offset.y(),
            self.center.z() + radial_point.z() + axial_offset.z(),
        )
    }

    /// パラメータ (u, v) での法線ベクトルを計算
    ///
    /// 円柱サーフェスの法線は径方向を向く
    ///
    /// # Parameters
    /// * `u` - 円周方向パラメータ
    /// * `v` - 軸方向パラメータ（法線には影響しない）
    ///
    /// # Returns
    /// 正規化された法線ベクトル
    pub fn normal_at_uv(&self, u: T, _v: T) -> Vector3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis().as_vector();

        // 径方向の単位ベクトル
        x_axis * cos_u + y_axis * sin_u
    }

    /// パラメータ (u, v) でのU方向接線ベクトルを計算
    ///
    /// ∂P/∂u = radius * (-sin(u) * X + cos(u) * Y)
    pub fn tangent_u_at_uv(&self, u: T, _v: T) -> Vector3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis().as_vector();

        (x_axis * (-sin_u) + y_axis * cos_u) * self.radius
    }

    /// パラメータ (u, v) でのV方向接線ベクトルを計算
    ///
    /// ∂P/∂v = Z (軸方向)
    pub fn tangent_v_at_uv(&self, _u: T, _v: T) -> Vector3D<T> {
        self.axis.as_vector()
    }

    // ========================================================================
    // Core Surface Analysis Methods
    // ========================================================================

    /// パラメータ (u, v) での主曲率を計算
    ///
    /// 円柱サーフェスの場合：
    /// - 円周方向曲率 κ₁ = 1/radius
    /// - 軸方向曲率 κ₂ = 0
    ///
    /// # Returns
    /// (κ₁, κ₂) - 主曲率のペア
    pub fn curvature_at_uv(&self, _u: T, _v: T) -> (T, T) {
        let circumferential_curvature = T::ONE / self.radius;
        let axial_curvature = T::ZERO;
        (circumferential_curvature, axial_curvature)
    }

    /// 平均曲率を計算（H = (κ₁ + κ₂) / 2）
    pub fn mean_curvature_at_uv(&self, u: T, v: T) -> T {
        let (k1, k2) = self.curvature_at_uv(u, v);
        (k1 + k2) / T::from_f64(2.0)
    }

    /// ガウス曲率を計算（K = κ₁ * κ₂）
    pub fn gaussian_curvature_at_uv(&self, u: T, v: T) -> T {
        let (k1, k2) = self.curvature_at_uv(u, v);
        k1 * k2
    }

    // ========================================================================
    // Core Geometric Properties (サーフェス特性)
    // ========================================================================

    /// 円柱サーフェスの境界ボックスを計算（無限軸方向のため制限が必要）
    ///
    /// 注意: 無限サーフェスのため、v方向の境界は外部で指定する必要がある
    /// ここでは半径による径方向の境界のみ計算
    pub fn bounding_box_radial(&self) -> BBox3D<T> {
        // 各軸成分の最大伸び（径方向のみ）
        let axis_x = self.axis.x();
        let axis_y = self.axis.y();
        let axis_z = self.axis.z();

        // 円形断面の各軸方向への最大伸び
        let radius_x = self.radius * (T::ONE - axis_x * axis_x).sqrt();
        let radius_y = self.radius * (T::ONE - axis_y * axis_y).sqrt();
        let radius_z = self.radius * (T::ONE - axis_z * axis_z).sqrt();

        let min_x = self.center.x() - radius_x;
        let max_x = self.center.x() + radius_x;

        let min_y = self.center.y() - radius_y;
        let max_y = self.center.y() + radius_y;

        let min_z = self.center.z() - radius_z;
        let max_z = self.center.z() + radius_z;

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    // ========================================================================
    // Core Distance and Projection Methods (サーフェス特性)
    // ========================================================================

    /// 点からサーフェスへの最短距離を計算
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        // 点から軸への距離を計算
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 軸方向成分を除去して径方向距離を計算
        let axis_projection = to_point.dot(&self.axis.as_vector());
        let axis_component = self.axis.as_vector() * axis_projection;
        let radial_vector = Vector3D::new(
            to_point.x() - axis_component.x(),
            to_point.y() - axis_component.y(),
            to_point.z() - axis_component.z(),
        );
        let radial_distance = radial_vector.length();

        // サーフェスまでの距離
        (radial_distance - self.radius).abs()
    }

    /// 点をサーフェス上に投影し、最近点とそのUVパラメータを返す
    pub fn closest_point_on_surface(&self, point: Point3D<T>) -> (Point3D<T>, T, T) {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // V パラメータ（軸方向）
        let v = to_point.dot(&self.axis.as_vector());

        // 径方向ベクトル
        let axis_component = self.axis.as_vector() * v;
        let radial_vector = Vector3D::new(
            to_point.x() - axis_component.x(),
            to_point.y() - axis_component.y(),
            to_point.z() - axis_component.z(),
        );
        let radial_distance = radial_vector.length();

        // U パラメータ（角度）
        let u = if radial_distance > T::EPSILON {
            let normalized_radial = radial_vector / radial_distance;
            let x_component = normalized_radial.dot(&self.ref_direction.as_vector());
            let y_component = normalized_radial.dot(&self.y_axis().as_vector());
            y_component.atan2(x_component)
        } else {
            T::ZERO // 軸上の点の場合は任意の角度
        };

        // サーフェス上の最近点
        let closest_point = self.point_at_uv(u, v);

        (closest_point, u, v)
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for CylindricalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CylindricalSurface3D(center: {:?}, axis: {:?}, ref_direction: {:?}, radius: {:?})",
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius()
        )
    }
}
