//! 3次元球ソリッド（SphericalSolid3D）のCore実装
//!
//! STEP準拠のSOLID_SPHERE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：ソリッド（立体）として明確に定義
//! 拡張機能は spherical_solid_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! SOLID_SPHERE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS);
//! ```
//! - location: 球の中心点（center）
//! - axis: Z軸方向（参照軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 球の半径
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

// use crate::{BBox3D, Direction3D, Plane3DCoordinateSystem, Point3D, Vector3D}; // 一時的にコメントアウト
use crate::{BBox3D, Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元球ソリッド（STEP準拠のCore実装）
///
/// STEP AP214の SOLID_SPHERE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：立体として体積・内部判定を持つ
///
/// ## 座標系定義（STEP準拠）
/// - center: 球の中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 参照軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 球の半径
///
/// ## ソリッド特性
/// - 体積計算：V = (4/3)π × r³
/// - 内部判定：点の包含テスト
/// - 表面積計算：S = 4π × r²
/// - 境界ボックス：中心を基準とした立方体
///
/// ## CAD用途
/// - パラメトリック球ソリッドの基準座標系
/// - ブーリアン演算（和・差・積）
/// - STEPファイルとの相互変換
/// - 体積・質量特性計算
#[derive(Debug, Clone, PartialEq)]
pub struct SphericalSolid3D<T: Scalar> {
    /// 球の中心点（STEP: location）
    center: Point3D<T>,

    /// 参照軸方向（STEP: axis）- Z軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    /// axis と直交していなくても自動調整
    ref_direction: Direction3D<T>,

    /// 球の半径
    radius: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> SphericalSolid3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で球ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 球の中心点
    /// * `axis` - 参照軸ベクトル（Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `radius` - 半径（正の値）
    ///
    /// # Returns
    /// 有効な球ソリッドが作成できた場合は `Some(SphericalSolid3D)`、
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
        })
    }

    /// Z軸標準の球ソリッドを作成（簡易コンストラクタ）
    pub fn new_standard(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
        )
    }

    /// 原点中心の球ソリッドを作成（簡易コンストラクタ）
    pub fn new_at_origin(radius: T) -> Option<Self> {
        Self::new_standard(Point3D::new(T::ZERO, T::ZERO, T::ZERO), radius)
    }

    /// 直径から球ソリッドを作成
    pub fn from_diameter(
        center: Point3D<T>,
        axis: Vector3D<T>,
        ref_direction: Vector3D<T>,
        diameter: T,
    ) -> Option<Self> {
        if diameter <= T::ZERO {
            return None;
        }
        Self::new(center, axis, ref_direction, diameter / T::from_f64(2.0))
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 球の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 参照軸方向を取得（正規化済み）
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

    /// 球ソリッドの半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 球ソリッドの位置（座標系）を取得
    // 一時的にコメントアウト: Plane3DCoordinateSystemが未定義
    // pub fn position(&self) -> Plane3DCoordinateSystem<T> {
    //     // Plane3DCoordinateSystemを原点と軸から構築
    //     Plane3DCoordinateSystem::from_origin_and_axes(
    //         self.center,
    //         self.axis.as_vector(),          // 法線方向
    //         self.ref_direction.as_vector(), // U軸方向
    //     )
    //     .expect("Coordinate system creation should succeed with valid sphere parameters")
    // }
    /// 球の直径を取得
    pub fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    // ========================================================================
    // Core Geometric Properties (ソリッド特性)
    // ========================================================================

    /// 球ソリッドの体積を計算
    ///
    /// 体積 = (4/3)π × r³
    pub fn volume(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius * self.radius / T::from_f64(3.0)
    }

    /// 球ソリッドの表面積を計算
    ///
    /// 表面積 = 4π × r²
    pub fn surface_area(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius
    }

    /// 球ソリッドの境界ボックスを計算
    pub fn bounding_box(&self) -> BBox3D<T> {
        let min_point = Point3D::new(
            self.center.x() - self.radius,
            self.center.y() - self.radius,
            self.center.z() - self.radius,
        );
        let max_point = Point3D::new(
            self.center.x() + self.radius,
            self.center.y() + self.radius,
            self.center.z() + self.radius,
        );
        BBox3D::new(min_point, max_point)
    }

    // ========================================================================
    // Core Containment and Distance Methods (ソリッド特性)
    // ========================================================================

    /// 点が球ソリッド内部に含まれるかを判定
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
        let distance_squared = self.center.distance_squared_to(&point);
        distance_squared <= self.radius * self.radius
    }

    /// 点から球ソリッド表面までの距離を計算
    /// 内部の点の場合は負の値を返す
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        let distance_to_center = self.center.distance_to(&point);
        distance_to_center - self.radius
    }

    /// 球ソリッドが退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        self.radius <= T::EPSILON
    }

    // ========================================================================
    // Surface Operations
    // ========================================================================

    /// 指定された方向ベクトルで球表面上の点を取得
    /// 方向ベクトルは正規化される
    pub fn point_on_surface_in_direction(&self, direction: Vector3D<T>) -> Option<Point3D<T>> {
        let normalized = direction.normalize();
        if normalized == Vector3D::new(T::ZERO, T::ZERO, T::ZERO) {
            return None;
        }

        Some(Point3D::new(
            self.center.x() + normalized.x() * self.radius,
            self.center.y() + normalized.y() * self.radius,
            self.center.z() + normalized.z() * self.radius,
        ))
    }

    /// 球の中心から指定された点への方向の表面点を取得
    pub fn point_on_surface_towards(&self, target: Point3D<T>) -> Option<Point3D<T>> {
        let direction = Vector3D::new(
            target.x() - self.center.x(),
            target.y() - self.center.y(),
            target.z() - self.center.z(),
        );
        self.point_on_surface_in_direction(direction)
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for SphericalSolid3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SphericalSolid3D(center: ({}, {}, {}), axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {})",
            self.center.x(),
            self.center.y(),
            self.center.z(),
            self.axis.x(),
            self.axis.y(),
            self.axis.z(),
            self.ref_direction.x(),
            self.ref_direction.y(),
            self.ref_direction.z(),
            self.radius
        )
    }
}

// ============================================================================
// Backward Compatibility (移行期間中のみ)
// ============================================================================

/// 旧名前との互換性のためのエイリアス
/// 将来のバージョンで削除予定
#[deprecated(since = "0.1.0", note = "Use SphericalSolid3D instead")]
pub type SolidSphere3D<T> = SphericalSolid3D<T>;
