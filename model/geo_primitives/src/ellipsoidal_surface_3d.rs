//! 3次元楕円体サーフェス（EllipsoidalSurface3D）のCore実装
//!
//! STEP準拠の楕円体サーフェス + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//! 拡張機能は ellipsoidal_surface_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! ELLIPSOIDAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), A_RADIUS, B_RADIUS, C_RADIUS);
//! ```
//! - location: 楕円体の中心点（center）
//! - axis: Z軸方向（楕円体軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - a_radius: X軸方向の半径
//! - b_radius: Y軸方向の半径
//! - c_radius: Z軸方向の半径
//!
//! **作成日: 2025年11月15日**
//! **最終更新: 2025年11月15日**

use crate::{BBox3D, Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元楕円体サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の楕円体サーフェス + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
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
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 方位角パラメータ（経度）
/// - v ∈ [-π/2, π/2]: 仰角パラメータ（緯度）
/// - Point(u, v) = center + (a*cos(v)*cos(u)*X + b*cos(v)*sin(u)*Y + c*sin(v)*Z)
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
pub struct EllipsoidalSurface3D<T: Scalar> {
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

impl<T: Scalar> EllipsoidalSurface3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で楕円体サーフェスを作成
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
    /// 有効な楕円体サーフェスが作成できた場合は `Some(EllipsoidalSurface3D)`、
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

        // 軸方向を正規化
        let axis_direction = Direction3D::from_vector(axis)?;

        // グラム・シュミット正規直交化で参照方向を調整
        let axis_vec = axis_direction.as_vector();
        let dot_product = ref_direction.x() * axis_vec.x()
            + ref_direction.y() * axis_vec.y()
            + ref_direction.z() * axis_vec.z();

        // 参照方向から軸成分を除去
        let orthogonal_ref = Vector3D::new(
            ref_direction.x() - dot_product * axis_vec.x(),
            ref_direction.y() - dot_product * axis_vec.y(),
            ref_direction.z() - dot_product * axis_vec.z(),
        );

        // 正規化して参照方向とする
        let ref_dir = Direction3D::from_vector(orthogonal_ref)?;

        Some(Self {
            center,
            axis: axis_direction,
            ref_direction: ref_dir,
            a_radius,
            b_radius,
            c_radius,
        })
    }

    /// 原点を中心とした標準楕円体サーフェスを作成
    ///
    /// # Arguments
    /// * `a_radius` - X軸方向の半径
    /// * `b_radius` - Y軸方向の半径
    /// * `c_radius` - Z軸方向の半径
    ///
    /// # Returns
    /// Z軸を軸とし、X軸を参照方向とする楕円体サーフェス
    pub fn new_at_origin(a_radius: T, b_radius: T, c_radius: T) -> Option<Self> {
        Self::new(
            Point3D::origin(),
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            a_radius,
            b_radius,
            c_radius,
        )
    }

    /// 球形楕円体サーフェスを作成（全軸の半径が同じ）
    ///
    /// # Arguments
    /// * `center` - 中心点
    /// * `radius` - 全軸共通の半径
    pub fn new_spherical(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
            radius,
            radius,
        )
    }

    // ========================================================================
    // アクセサメソッド
    // ========================================================================

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 軸方向を取得
    pub fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    /// 参照方向を取得
    pub fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
    }

    /// X軸方向の半径を取得
    pub fn a_radius(&self) -> T {
        self.a_radius
    }

    /// Y軸方向の半径を取得
    pub fn b_radius(&self) -> T {
        self.b_radius
    }

    /// Z軸方向の半径を取得
    pub fn c_radius(&self) -> T {
        self.c_radius
    }

    /// Y軸方向を計算（派生軸）
    ///
    /// STEP標準：Y = Z × X（右手系）
    pub fn derived_y_axis(&self) -> Direction3D<T> {
        let z = self.axis.as_vector();
        let x = self.ref_direction.as_vector();

        // 外積: Z × X = Y
        let y = Vector3D::new(
            z.y() * x.z() - z.z() * x.y(),
            z.z() * x.x() - z.x() * x.z(),
            z.x() * x.y() - z.y() * x.x(),
        );

        // Direction3D は正規化を保証する
        Direction3D::from_vector(y).expect("Y軸の計算は常に成功する（直交軸系のため）")
    }

    // ========================================================================
    // 幾何計算
    // ========================================================================

    /// パラメータ (u, v) での点を計算
    ///
    /// # Arguments
    /// * `u` - 方位角パラメータ（ラジアン、0から2π）
    /// * `v` - 仰角パラメータ（ラジアン、-π/2からπ/2）
    ///
    /// # Returns
    /// サーフェス上の点
    pub fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 楕円体上の点
        let ellipsoid_point = Vector3D::new(
            self.a_radius * cos_v * cos_u * x_axis.x()
                + self.b_radius * cos_v * sin_u * y_axis.x()
                + self.c_radius * sin_v * z_axis.x(),
            self.a_radius * cos_v * cos_u * x_axis.y()
                + self.b_radius * cos_v * sin_u * y_axis.y()
                + self.c_radius * sin_v * z_axis.y(),
            self.a_radius * cos_v * cos_u * x_axis.z()
                + self.b_radius * cos_v * sin_u * y_axis.z()
                + self.c_radius * sin_v * z_axis.z(),
        );

        Point3D::new(
            self.center.x() + ellipsoid_point.x(),
            self.center.y() + ellipsoid_point.y(),
            self.center.z() + ellipsoid_point.z(),
        )
    }

    /// パラメータ (u, v) での法線ベクトルを計算
    ///
    /// # Arguments
    /// * `u` - 方位角パラメータ（ラジアン）
    /// * `v` - 仰角パラメータ（ラジアン）
    ///
    /// # Returns
    /// その点での単位法線ベクトル
    pub fn normal_at_uv(&self, u: T, v: T) -> Option<Direction3D<T>> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 楕円体の法線（非正規化）
        let normal_x = cos_v * cos_u / self.a_radius;
        let normal_y = cos_v * sin_u / self.b_radius;
        let normal_z = sin_v / self.c_radius;

        // 座標系に変換
        let normal = Vector3D::new(
            normal_x * x_axis.x() + normal_y * y_axis.x() + normal_z * z_axis.x(),
            normal_x * x_axis.y() + normal_y * y_axis.y() + normal_z * z_axis.y(),
            normal_x * x_axis.z() + normal_y * y_axis.z() + normal_z * z_axis.z(),
        );

        Direction3D::from_vector(normal)
    }

    /// 指定点がサーフェス上にあるかを判定
    ///
    /// # Arguments
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    ///
    /// # Returns
    /// サーフェス上にある場合は true
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // 点を楕円体の局所座標系に変換
        let relative = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 局所座標系での座標
        let local_x =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();
        let local_y =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();
        let local_z =
            relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        // 楕円体の方程式： (x/a)² + (y/b)² + (z/c)² = 1
        let normalized_x = local_x / self.a_radius;
        let normalized_y = local_y / self.b_radius;
        let normalized_z = local_z / self.c_radius;

        let equation_value =
            normalized_x * normalized_x + normalized_y * normalized_y + normalized_z * normalized_z;

        (equation_value - T::ONE).abs() <= tolerance
    }

    /// サーフェスからの距離を計算（近似）
    ///
    /// # Arguments
    /// * `point` - 距離を計算する点
    ///
    /// # Returns
    /// サーフェスまでの最短距離（近似値）
    pub fn distance_to_surface(&self, point: &Point3D<T>) -> T {
        // 簡略実装：正確な最短距離は複雑な最適化問題
        // ここでは楕円体の方程式からの偏差を使用
        let relative = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        let local_x =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();
        let local_y =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();
        let local_z =
            relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        let normalized_x = local_x / self.a_radius;
        let normalized_y = local_y / self.b_radius;
        let normalized_z = local_z / self.c_radius;

        let equation_value = (normalized_x * normalized_x
            + normalized_y * normalized_y
            + normalized_z * normalized_z)
            .sqrt();

        // 平均半径を使った近似距離
        let avg_radius = (self.a_radius + self.b_radius + self.c_radius) / T::from_f64(3.0);
        (equation_value - T::ONE).abs() * avg_radius
    }

    // ========================================================================
    // 検証メソッド
    // ========================================================================

    /// サーフェスの有効性を検証
    pub fn is_valid(&self) -> bool {
        self.a_radius > T::ZERO && self.b_radius > T::ZERO && self.c_radius > T::ZERO
    }

    /// 境界ボックスを計算
    pub fn bounding_box(&self) -> BBox3D<T> {
        // 各軸方向の最大伸び
        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 各座標軸での最大・最小値を計算
        let max_x_extent = (self.a_radius * x_axis.x().abs()
            + self.b_radius * y_axis.x().abs()
            + self.c_radius * z_axis.x().abs())
        .max(T::EPSILON);
        let max_y_extent = (self.a_radius * x_axis.y().abs()
            + self.b_radius * y_axis.y().abs()
            + self.c_radius * z_axis.y().abs())
        .max(T::EPSILON);
        let max_z_extent = (self.a_radius * x_axis.z().abs()
            + self.b_radius * y_axis.z().abs()
            + self.c_radius * z_axis.z().abs())
        .max(T::EPSILON);

        BBox3D::new(
            Point3D::new(
                self.center.x() - max_x_extent,
                self.center.y() - max_y_extent,
                self.center.z() - max_z_extent,
            ),
            Point3D::new(
                self.center.x() + max_x_extent,
                self.center.y() + max_y_extent,
                self.center.z() + max_z_extent,
            ),
        )
    }
}

// ============================================================================
// Standard Traits
// ============================================================================

impl<T: Scalar> std::fmt::Display for EllipsoidalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EllipsoidalSurface3D {{ center: {:?}, axis: {:?}, ref_direction: {:?}, a_radius: {}, b_radius: {}, c_radius: {} }}",
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.a_radius(),
            self.b_radius(),
            self.c_radius()
        )
    }
}
