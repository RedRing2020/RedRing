//! 3次元球サーフェス（SphericalSurface3D）のCore実装
//!
//! STEP準拠のSPHERICAL_SURFACE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//! 拡張機能は spherical_surface_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! SPHERICAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS);
//! ```
//! - location: 球の中心点（center）
//! - axis: Z軸方向（参照軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 球の半径
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::{BBox3D, Direction3D, Plane3DCoordinateSystem, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元球サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の SPHERICAL_SURFACE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 球の中心点（STEP: location）
/// - axis: Z軸方向（STEP: axis）- 参照軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 球の半径
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 方位角パラメータ（経度）
/// - v ∈ [-π/2, π/2]: 仰角パラメータ（緯度）
/// - Point(u, v) = center + radius * (cos(v)*cos(u)*X + cos(v)*sin(u)*Y + sin(v)*Z)
///
/// ## サーフェス特性
/// - 厚みなし：純粋な2次元多様体
/// - パラメータ化：連続的なUV座標系
/// - 法線計算：各点での外向き法線ベクトル
/// - 曲率解析：主曲率・平均曲率・ガウス曲率
/// - 境界操作：トリム・分割・結合
///
/// ## CAD用途
/// - 表面解析・品質評価
/// - CAM工具経路生成
/// - レンダリング・テクスチャマッピング
/// - NURBS変換・高精度表現
#[derive(Debug, Clone, PartialEq)]
pub struct SphericalSurface3D<T: Scalar> {
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

impl<T: Scalar> SphericalSurface3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で球サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 球の中心点
    /// * `axis` - 参照軸ベクトル（Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `radius` - 半径（正の値）
    ///
    /// # Returns
    /// 有効な球サーフェスが作成できた場合は `Some(SphericalSurface3D)`、
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

    /// Z軸標準の球サーフェスを作成（簡易コンストラクタ）
    pub fn new_standard(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
        )
    }

    /// 原点中心の球サーフェスを作成（簡易コンストラクタ）
    pub fn new_at_origin(radius: T) -> Option<Self> {
        Self::new_standard(Point3D::new(T::ZERO, T::ZERO, T::ZERO), radius)
    }

    /// 直径から球サーフェスを作成
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

    /// 球の半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 球の直径を取得
    pub fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    /// 球サーフェスの位置（座標系）を取得
    pub fn position(&self) -> Plane3DCoordinateSystem<T> {
        // Plane3DCoordinateSystemを原点と軸から構築
        Plane3DCoordinateSystem::from_origin_and_axes(
            self.center,
            self.axis.as_vector(),          // 法線方向
            self.ref_direction.as_vector(), // U軸方向
        )
        .expect("Coordinate system creation should succeed with valid sphere parameters")
    }

    /// パラメトリック座標での点を計算
    /// u: 極角 (0 ≤ u ≤ π) - 北極からの角度
    /// v: 方位角 (0 ≤ v ≤ 2π) - 赤道面での角度
    pub fn point_at(&self, u: T, v: T) -> Point3D<T> {
        let sin_u = u.sin();
        let cos_u = u.cos();
        let sin_v = v.sin();
        let cos_v = v.cos();

        let x_component = self.ref_direction.as_vector() * (self.radius * sin_u * cos_v);
        let y_component = self.y_axis().as_vector() * (self.radius * sin_u * sin_v);
        let z_component = self.axis.as_vector() * (self.radius * cos_u);

        let local_point = x_component + y_component + z_component;
        Point3D::new(
            self.center.x() + local_point.x(),
            self.center.y() + local_point.y(),
            self.center.z() + local_point.z(),
        )
    }

    /// パラメトリック座標での法線ベクトルを計算
    pub fn normal_at(&self, u: T, v: T) -> Direction3D<T> {
        let sin_u = u.sin();
        let cos_u = u.cos();
        let sin_v = v.sin();
        let cos_v = v.cos();

        let x_component = self.ref_direction.as_vector() * (sin_u * cos_v);
        let y_component = self.y_axis().as_vector() * (sin_u * sin_v);
        let z_component = self.axis.as_vector() * cos_u;

        let normal_vector = x_component + y_component + z_component;
        Direction3D::from_vector(normal_vector)
            .expect("Normal vector calculation should always succeed")
    }

    /// パラメトリック座標での主曲率を計算
    /// 球面では全ての点で主曲率は 1/r
    pub fn principal_curvatures_at(&self, _u: T, _v: T) -> (T, T) {
        let curvature = T::ONE / self.radius;
        (curvature, curvature)
    }

    /// 球サーフェスの表面積を計算
    pub fn area(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius
    }

    // ========================================================================
    // Core Geometric Properties (サーフェス特性)
    // ========================================================================

    /// 球サーフェスの表面積を計算
    ///
    /// 表面積 = 4π × r²
    pub fn surface_area(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius
    }

    /// 球サーフェスの境界ボックスを計算
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

    /// 球サーフェスが退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        self.radius <= T::EPSILON
    }

    // ========================================================================
    // Surface Distance and Proximity Methods
    // ========================================================================

    /// 点から球サーフェスまでの最短距離を計算
    /// 球内部の点の場合は負の値を返す
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        let distance_to_center = self.center.distance_to(&point);
        distance_to_center - self.radius
    }

    /// 点が球サーフェス上にあるかどうかを判定（許容誤差考慮）
    pub fn point_on_surface(&self, point: Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_surface(point).abs();
        distance <= tolerance
    }

    // ========================================================================
    // Parametric Surface Operations
    // ========================================================================

    /// パラメータ(u, v)から球サーフェス上の点を計算
    /// u: 方位角 [0, 2π], v: 仰角 [-π/2, π/2]
    pub fn point_at_parameters(&self, u: T, v: T) -> Point3D<T> {
        let cos_v = v.cos();
        let sin_v = v.sin();
        let cos_u = u.cos();
        let sin_u = u.sin();

        // ローカル座標系での点
        let local_x = cos_v * cos_u;
        let local_y = cos_v * sin_u;
        let local_z = sin_v;

        // ワールド座標系に変換
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        Point3D::new(
            self.center.x()
                + self.radius
                    * (local_x * x_axis.x() + local_y * y_axis.x() + local_z * z_axis.x()),
            self.center.y()
                + self.radius
                    * (local_x * x_axis.y() + local_y * y_axis.y() + local_z * z_axis.y()),
            self.center.z()
                + self.radius
                    * (local_x * x_axis.z() + local_y * y_axis.z() + local_z * z_axis.z()),
        )
    }

    /// 球サーフェス上の点での法線ベクトルを計算（外向き）
    pub fn normal_at_point(&self, point: Point3D<T>) -> Option<Direction3D<T>> {
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        Direction3D::from_vector(to_point)
    }

    /// 指定された方向ベクトルで球サーフェス上の点を取得
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

    /// 球の中心から指定された点への方向のサーフェス上の点を取得
    pub fn point_on_surface_towards(&self, target: Point3D<T>) -> Option<Point3D<T>> {
        let direction = Vector3D::new(
            target.x() - self.center.x(),
            target.y() - self.center.y(),
            target.z() - self.center.z(),
        );
        self.point_on_surface_in_direction(direction)
    }

    // ========================================================================
    // Curvature Analysis (曲率解析)
    // ========================================================================

    /// 球サーフェスの主曲率を計算（どの点でも同じ）
    /// 球の場合、両方の主曲率は 1/radius
    pub fn principal_curvatures(&self) -> (T, T) {
        let curvature = T::ONE / self.radius;
        (curvature, curvature)
    }

    /// 球サーフェスの平均曲率を計算
    /// H = (k1 + k2) / 2 = 1/radius
    pub fn mean_curvature(&self) -> T {
        T::ONE / self.radius
    }

    /// 球サーフェスのガウス曲率を計算
    /// K = k1 * k2 = 1/radius²
    pub fn gaussian_curvature(&self) -> T {
        T::ONE / (self.radius * self.radius)
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar> std::fmt::Display for SphericalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SphericalSurface3D(center: ({}, {}, {}), axis: ({}, {}, {}), ref_direction: ({}, {}, {}), radius: {})",
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
#[deprecated(since = "0.1.0", note = "Use SphericalSurface3D instead")]
pub type SurfaceSphere3D<T> = SphericalSurface3D<T>;
