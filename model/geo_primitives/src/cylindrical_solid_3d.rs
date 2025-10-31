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

use crate::{BBox3D, Direction3D, Point3D, Vector3D};
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
    // Core Accessor Methods
    // ========================================================================

    /// 底面の中心点を取得
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

    /// 円柱の高さを取得
    pub fn height(&self) -> T {
        self.height
    }

    // ========================================================================
    // Core Geometric Properties (ソリッド特性)
    // ========================================================================

    /// 円柱ソリッドの体積を計算
    ///
    /// 体積 = π × r² × h
    pub fn volume(&self) -> T {
        T::PI * self.radius * self.radius * self.height
    }

    /// 円柱ソリッドの表面積を計算
    ///
    /// 表面積 = 2π × r² + 2π × r × h (底面積 + 側面積)
    pub fn surface_area(&self) -> T {
        let base_area = T::PI * self.radius * self.radius;
        let side_area = T::from_f64(2.0) * T::PI * self.radius * self.height;
        T::from_f64(2.0) * base_area + side_area
    }

    /// 円柱ソリッドの境界ボックスを計算
    pub fn bounding_box(&self) -> BBox3D<T> {
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

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    // ========================================================================
    // Core Containment and Distance Methods (ソリッド特性)
    // ========================================================================

    /// 点が円柱ソリッド内部に含まれるかを判定
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
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
        let radial_component = to_point - axis_component;
        let radial_distance = radial_component.magnitude();

        radial_distance <= self.radius
    }

    /// 点から円柱ソリッド表面までの距離を計算
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
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
        let radial_component = to_point - axis_component;
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
// Backward Compatibility (移行期間中のみ)
// ============================================================================

/// 旧名前との互換性のためのエイリアス
/// 将来のバージョンで削除予定
#[deprecated(since = "0.1.0", note = "Use CylindricalSolid3D instead")]
pub type Cylinder3D<T> = CylindricalSolid3D<T>;
