//! 3次元円錐サーフェス（ConicalSurface3D）のCore実装
//!
//! STEP準拠のCONICAL_SURFACE + AXIS2_PLACEMENT_3Dに対応
//! 完全ハイブリッドモデラー対応：サーフェス（純粋面）として明確に定義
//! 拡張機能は conical_surface_3d_extensions.rs を参照
//!
//! ## STEP標準対応
//! ```step
//! CONICAL_SURFACE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION), RADIUS, SEMI_ANGLE);
//! ```
//! - location: 円錐軸上の基準点（center）
//! - axis: Z軸方向（円錐軸）- 正規化済み
//! - ref_direction: X軸方向（参照方向）- 正規化済み
//! - derived Y軸: axis × ref_direction で自動計算
//! - radius: 基準点での半径
//! - semi_angle: 半頂角（ラジアン）

use crate::{BBox3D, Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元円錐サーフェス（STEP準拠のCore実装）
///
/// STEP AP214の CONICAL_SURFACE + AXIS2_PLACEMENT_3D エンティティに対応
/// 完全ハイブリッドモデラー：純粋サーフェスとして厚みなし幾何学的表面
///
/// ## 座標系定義（STEP準拠）
/// - center: 円錐軸上の基準点（STEP: location）- 基準半径位置
/// - axis: Z軸方向（STEP: axis）- 円錐軸、正規化済み
/// - ref_direction: X軸方向（STEP: ref_direction）- 参照方向、正規化済み
/// - derived Y軸: axis × ref_direction で自動計算
/// - radius: 基準点での半径
/// - semi_angle: 半頂角（ラジアン）- 円錐の開き角度
///
/// ## パラメータ化 (u, v)
/// - u ∈ [0, 2π]: 円周方向パラメータ（角度）
/// - v ∈ ℝ: 軸方向パラメータ（無限範囲、境界で制限）
/// - Point(u, v) = center + (radius + v * tan(semi_angle)) * (cos(u) * X + sin(u) * Y) + v * Z
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
pub struct ConicalSurface3D<T: Scalar> {
    /// 円錐軸上の基準点（STEP: location）
    center: Point3D<T>,

    /// 円錐の軸方向（STEP: axis）- Z軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    axis: Direction3D<T>,

    /// 参照方向（STEP: ref_direction）- X軸、正規化済み
    /// Direction3D<T>により正規化が保証される
    /// axis と直交していなくても自動調整
    ref_direction: Direction3D<T>,

    /// 基準点での半径
    radius: T,

    /// 半頂角（ラジアン）- 円錐の開き角度
    semi_angle: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> ConicalSurface3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で円錐サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 軸上の基準点
    /// * `axis` - 軸方向ベクトル（円錐軸、Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）、軸と直交していなくても自動調整
    /// * `radius` - 基準点での半径（正の値）
    /// * `semi_angle` - 半頂角（ラジアン、0 < semi_angle < π/2）
    ///
    /// # Returns
    /// 有効な円錐サーフェスが作成できた場合は `Some(ConicalSurface3D)`、
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
        semi_angle: T,
    ) -> Option<Self> {
        // 半径の検証
        if radius <= T::ZERO {
            return None;
        }

        // 半頂角の検証（0 < semi_angle < π/2）
        if semi_angle <= T::ZERO || semi_angle >= T::PI / (T::ONE + T::ONE) {
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
            radius,
            semi_angle,
        })
    }

    /// 原点を中心とした標準円錐サーフェスを作成
    ///
    /// # Arguments
    /// * `radius` - 基準点での半径
    /// * `semi_angle` - 半頂角（ラジアン）
    ///
    /// # Returns
    /// Z軸を軸とし、X軸を参照方向とする円錐サーフェス
    pub fn new_at_origin(radius: T, semi_angle: T) -> Option<Self> {
        Self::new(
            Point3D::origin(),
            Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            radius,
            semi_angle,
        )
    }

    /// カスタム軸系での円錐サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 中心点
    /// * `z_axis` - Z軸方向（円錐軸）
    /// * `x_axis` - X軸方向（参照方向）
    /// * `radius` - 基準点での半径
    /// * `semi_angle` - 半頂角（ラジアン）
    pub fn new_with_custom_axes(
        center: Point3D<T>,
        z_axis: Vector3D<T>,
        x_axis: Vector3D<T>,
        radius: T,
        semi_angle: T,
    ) -> Option<Self> {
        Self::new(center, z_axis, x_axis, radius, semi_angle)
    }

    // ========================================================================
    // アクセサメソッド
    // ========================================================================

    /// 基準点を取得
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

    /// 基準点での半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 半頂角を取得
    pub fn semi_angle(&self) -> T {
        self.semi_angle
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

    /// 指定した軸方向距離での半径を計算
    ///
    /// # Arguments
    /// * `v` - 軸方向パラメータ（基準点からの距離）
    ///
    /// # Returns
    /// その位置での円錐の半径
    pub fn radius_at_v(&self, v: T) -> T {
        self.radius + v * self.semi_angle.tan()
    }

    /// パラメータ (u, v) での点を計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ（ラジアン）
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// サーフェス上の点
    pub fn point_at_uv(&self, u: T, v: T) -> Point3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let r_at_v = self.radius_at_v(v);

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 放射方向ベクトル
        let radial = Vector3D::new(
            r_at_v * (cos_u * x_axis.x() + sin_u * y_axis.x()),
            r_at_v * (cos_u * x_axis.y() + sin_u * y_axis.y()),
            r_at_v * (cos_u * x_axis.z() + sin_u * y_axis.z()),
        );

        // 軸方向ベクトル
        let axial = Vector3D::new(v * z_axis.x(), v * z_axis.y(), v * z_axis.z());

        Point3D::new(
            self.center.x() + radial.x() + axial.x(),
            self.center.y() + radial.y() + axial.y(),
            self.center.z() + radial.z() + axial.z(),
        )
    }

    /// パラメータ (u, v) での法線ベクトルを計算
    ///
    /// # Arguments
    /// * `u` - 円周方向パラメータ（ラジアン）
    /// * `v` - 軸方向パラメータ
    ///
    /// # Returns
    /// その点での単位法線ベクトル
    pub fn normal_at_uv(&self, u: T, _v: T) -> Option<Direction3D<T>> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let tan_angle = self.semi_angle.tan();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 放射方向成分
        let radial_dir = Vector3D::new(
            cos_u * x_axis.x() + sin_u * y_axis.x(),
            cos_u * x_axis.y() + sin_u * y_axis.y(),
            cos_u * x_axis.z() + sin_u * y_axis.z(),
        );

        // 法線 = 放射方向 - tan(角度) * 軸方向
        let normal = Vector3D::new(
            radial_dir.x() - tan_angle * z_axis.x(),
            radial_dir.y() - tan_angle * z_axis.y(),
            radial_dir.z() - tan_angle * z_axis.z(),
        );

        Direction3D::from_vector(normal)
    }

    /// 頂点（apex）の位置を計算
    ///
    /// # Returns
    /// 円錐の頂点位置
    pub fn apex(&self) -> Point3D<T> {
        let distance_to_apex = -self.radius / self.semi_angle.tan();
        let axis_vec = self.axis.as_vector();

        Point3D::new(
            self.center.x() + distance_to_apex * axis_vec.x(),
            self.center.y() + distance_to_apex * axis_vec.y(),
            self.center.z() + distance_to_apex * axis_vec.z(),
        )
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
        // 点を円錐の局所座標系に変換
        let relative = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();
        let z_axis = self.axis.as_vector();

        // 軸方向成分
        let v = relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        // 放射方向成分
        let radial_x =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();
        let radial_y =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();
        let radial_distance = (radial_x * radial_x + radial_y * radial_y).sqrt();

        // 期待される半径
        let expected_radius = self.radius_at_v(v);

        // 許容誤差内での判定
        (radial_distance - expected_radius).abs() <= tolerance
    }

    /// サーフェスからの距離を計算
    ///
    /// # Arguments
    /// * `point` - 距離を計算する点
    ///
    /// # Returns
    /// サーフェスまでの最短距離
    pub fn distance_to_surface(&self, point: &Point3D<T>) -> T {
        // 簡略実装：正確な最短距離は複雑な最適化問題
        // ここでは近似として、軸方向を固定した放射距離を使用
        let relative = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        let z_axis = self.axis.as_vector();
        let v = relative.x() * z_axis.x() + relative.y() * z_axis.y() + relative.z() * z_axis.z();

        let x_axis = self.ref_direction.as_vector();
        let y_axis = self.derived_y_axis().as_vector();

        let radial_x =
            relative.x() * x_axis.x() + relative.y() * x_axis.y() + relative.z() * x_axis.z();
        let radial_y =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();
        let radial_distance = (radial_x * radial_x + radial_y * radial_y).sqrt();

        let expected_radius = self.radius_at_v(v);
        (radial_distance - expected_radius).abs()
    }

    // ========================================================================
    // 検証メソッド
    // ========================================================================

    /// サーフェスの有効性を検証
    pub fn is_valid(&self) -> bool {
        // 基本的な有効性チェック
        self.radius > T::ZERO
            && self.semi_angle > T::ZERO
            && self.semi_angle < T::PI / (T::ONE + T::ONE)
    }

    /// 境界ボックスを計算
    ///
    /// # Arguments
    /// * `min_v` - 軸方向の最小範囲
    /// * `max_v` - 軸方向の最大範囲
    ///
    /// # Returns
    /// 指定範囲での境界ボックス
    pub fn bounding_box(&self, min_v: T, max_v: T) -> BBox3D<T> {
        let r_min = self.radius_at_v(min_v);
        let r_max = self.radius_at_v(max_v);
        let max_radius = if r_min > r_max { r_min } else { r_max };

        // 軸方向の範囲
        let axis_vec = self.axis.as_vector();
        let min_point = Point3D::new(
            self.center.x() + min_v * axis_vec.x(),
            self.center.y() + min_v * axis_vec.y(),
            self.center.z() + min_v * axis_vec.z(),
        );
        let max_point = Point3D::new(
            self.center.x() + max_v * axis_vec.x(),
            self.center.y() + max_v * axis_vec.y(),
            self.center.z() + max_v * axis_vec.z(),
        );

        // 保守的な境界ボックス（最大半径で囲む）
        let min_x = if min_point.x() < max_point.x() {
            min_point.x() - max_radius
        } else {
            max_point.x() - max_radius
        };
        let max_x = if min_point.x() > max_point.x() {
            min_point.x() + max_radius
        } else {
            max_point.x() + max_radius
        };

        let min_y = if min_point.y() < max_point.y() {
            min_point.y() - max_radius
        } else {
            max_point.y() - max_radius
        };
        let max_y = if min_point.y() > max_point.y() {
            min_point.y() + max_radius
        } else {
            max_point.y() + max_radius
        };

        let min_z = if min_point.z() < max_point.z() {
            min_point.z() - max_radius
        } else {
            max_point.z() - max_radius
        };
        let max_z = if min_point.z() > max_point.z() {
            min_point.z() + max_radius
        } else {
            max_point.z() + max_radius
        };

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }
}

// ============================================================================
// Standard Traits
// ============================================================================

impl<T: Scalar> std::fmt::Display for ConicalSurface3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ConicalSurface3D {{ center: {:?}, axis: {:?}, ref_direction: {:?}, radius: {}, semi_angle: {} }}",
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            self.semi_angle()
        )
    }
}

/// ConeRim3D のエイリアス（後方互換性）
pub type ConeRim3D<T> = ConicalSurface3D<T>;
