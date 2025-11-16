//! Plane3D CAD実装
//!
//! CAD用の座標系付き平面実装（旧Plane3DCoordinateSystem）
//! STEP AP214準拠の完全な平面座標系を提供

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// CAD用3次元平面（座標系付き）
///
/// STEP AP214の AXIS2_PLACEMENT_3D + PLANE に対応
/// 完全な座標系情報（原点 + X軸 + Y軸 + Z軸）を持つ平面
///
/// ## 座標系定義
/// - origin: 平面原点（STEP: location）
/// - normal: Z軸方向（STEP: axis）- 法線ベクトル
/// - u_axis: X軸方向（STEP: ref_direction）- 第一軸
/// - v_axis: Y軸方向（STEP: derived）- normal × u_axis で自動計算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane3D<T: Scalar> {
    /// 平面原点（STEP: location）
    origin: Point3D<T>,

    /// Z軸方向 - 法線ベクトル（STEP: axis）
    normal: Direction3D<T>,

    /// X軸方向 - 第一軸（STEP: ref_direction）
    u_axis: Direction3D<T>,

    /// Y軸方向 - 第二軸（STEP: derived, normal × u_axis）
    v_axis: Direction3D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Plane3D<T> {
    // ========================================================================
    // STEP準拠のコンストラクタ
    // ========================================================================

    /// STEP AXIS2_PLACEMENT_3D 形式で平面座標系を作成
    ///
    /// # Arguments
    /// * `origin` - 平面原点
    /// * `normal` - 法線方向（Z軸）
    /// * `u_direction` - 第一軸方向（X軸）、法線と直交していなくても自動調整
    ///
    /// # Returns
    /// 新しい平面座標系、法線がゼロベクトルの場合は None
    pub fn from_origin_and_axes(
        origin: Point3D<T>,
        normal: Vector3D<T>,
        u_direction: Vector3D<T>,
    ) -> Option<Self> {
        // 法線の正規化
        let z_axis = Direction3D::from_vector(normal)?;

        // U軸の法線成分を除去して正規化（グラム・シュミット正規直交化）
        let normal_component = u_direction.dot(&z_axis.as_vector());
        let orthogonal_u = u_direction - z_axis.as_vector() * normal_component;
        let u_axis = Direction3D::from_vector(orthogonal_u)?;

        // V軸 = Z軸 × U軸（右手系）
        let v_vector = z_axis.as_vector().cross(&u_axis.as_vector());
        let v_axis = Direction3D::from_vector(v_vector)?;

        Some(Self {
            origin,
            normal: z_axis,
            u_axis,
            v_axis,
        })
    }

    /// 3点から平面座標系を作成
    ///
    /// # Arguments
    /// * `origin` - 原点
    /// * `point_u` - U軸方向の参照点
    /// * `point_v` - V軸方向の参照点
    pub fn from_three_points(
        origin: Point3D<T>,
        point_u: Point3D<T>,
        point_v: Point3D<T>,
    ) -> Option<Self> {
        let u_vector = Vector3D::new(
            point_u.x() - origin.x(),
            point_u.y() - origin.y(),
            point_u.z() - origin.z(),
        );
        let v_vector = Vector3D::new(
            point_v.x() - origin.x(),
            point_v.y() - origin.y(),
            point_v.z() - origin.z(),
        );

        // 法線 = U × V（右手系）
        let normal = u_vector.cross(&v_vector);

        Self::from_origin_and_axes(origin, normal, u_vector)
    }

    /// XY平面を作成（z = constant）
    pub fn xy_plane(z: T) -> Self {
        Self::from_origin_and_axes(
            Point3D::new(T::ZERO, T::ZERO, z),
            Vector3D::unit_z(),
            Vector3D::unit_x(),
        )
        .unwrap()
    }

    /// XZ平面を作成（y = constant）
    pub fn xz_plane(y: T) -> Self {
        Self::from_origin_and_axes(
            Point3D::new(T::ZERO, y, T::ZERO),
            Vector3D::unit_y(),
            Vector3D::unit_x(),
        )
        .unwrap()
    }

    /// YZ平面を作成（x = constant）
    pub fn yz_plane(x: T) -> Self {
        Self::from_origin_and_axes(
            Point3D::new(x, T::ZERO, T::ZERO),
            Vector3D::unit_x(),
            Vector3D::unit_y(),
        )
        .unwrap()
    }

    /// 点と法線から平面を作成（レガシー互換性のため）
    ///
    /// # Arguments
    /// * `point` - 平面上の点（原点として使用）
    /// * `normal` - 法線ベクトル
    ///
    /// # Returns
    /// 新しい平面座標系、法線がゼロベクトルの場合は None
    pub fn from_point_and_normal(point: Point3D<T>, normal: Vector3D<T>) -> Option<Self> {
        // デフォルトのU軸候補を法線に対して垂直になるように選択
        let candidate_u = if normal.x().abs() < T::from_f64(0.9) {
            Vector3D::unit_x()
        } else {
            Vector3D::unit_y()
        };

        Self::from_origin_and_axes(point, normal, candidate_u)
    }

    // ========================================================================
    // アクセサメソッド
    // ========================================================================

    /// 平面原点を取得
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// 法線方向（Z軸）を取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// U軸方向（X軸）を取得
    pub fn u_axis(&self) -> Direction3D<T> {
        self.u_axis
    }

    /// V軸方向（Y軸）を取得
    pub fn v_axis(&self) -> Direction3D<T> {
        self.v_axis
    }

    /// 従来の点+法線ベクトル形式で原点と法線を取得
    pub fn point(&self) -> Point3D<T> {
        self.origin
    }

    // ========================================================================
    // Core Geometric Operations
    // ========================================================================

    /// 点が平面上にあるかチェック
    pub fn contains_point(&self, point: Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point(point).abs();
        distance <= tolerance
    }

    /// 点から平面までの符号付き距離
    pub fn distance_to_point(&self, point: Point3D<T>) -> T {
        let relative = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        relative.dot(&self.normal.as_vector())
    }

    /// 点を平面に投影
    pub fn project_point(&self, point: Point3D<T>) -> Point3D<T> {
        let distance = self.distance_to_point(point);
        let offset = self.normal.as_vector() * distance;
        Point3D::new(
            point.x() - offset.x(),
            point.y() - offset.y(),
            point.z() - offset.z(),
        )
    }

    /// 平面ローカル座標をワールド座標に変換
    pub fn local_to_world(&self, u: T, v: T) -> Point3D<T> {
        Point3D::new(
            self.origin.x() + u * self.u_axis.x() + v * self.v_axis.x(),
            self.origin.y() + u * self.u_axis.y() + v * self.v_axis.y(),
            self.origin.z() + u * self.u_axis.z() + v * self.v_axis.z(),
        )
    }

    /// ワールド座標を平面ローカル座標に変換
    pub fn world_to_local(&self, world_point: Point3D<T>) -> (T, T, T) {
        let relative = Vector3D::new(
            world_point.x() - self.origin.x(),
            world_point.y() - self.origin.y(),
            world_point.z() - self.origin.z(),
        );

        let u = relative.dot(&self.u_axis.as_vector());
        let v = relative.dot(&self.v_axis.as_vector());
        let distance = relative.dot(&self.normal.as_vector());

        (u, v, distance)
    }

    /// 平面上に点を投影してUV座標を取得
    pub fn project_to_uv(&self, world_point: Point3D<T>) -> (T, T) {
        let (u, v, _) = self.world_to_local(world_point);
        (u, v)
    }

    /// 平面の方程式係数を取得
    /// Returns (a, b, c, d) where ax + by + cz + d = 0
    pub fn equation_coefficients(&self) -> (T, T, T, T) {
        let a = self.normal.x();
        let b = self.normal.y();
        let c = self.normal.z();
        let d = -(a * self.origin.x() + b * self.origin.y() + c * self.origin.z());
        (a, b, c, d)
    }

    /// 平面座標系が有効かチェック
    pub fn is_valid(&self) -> bool {
        // 各軸がDirection3Dで正規化されていることを前提
        // 直交性のチェック
        let dot_uv = self.u_axis.as_vector().dot(&self.v_axis.as_vector()).abs();
        let dot_un = self.u_axis.as_vector().dot(&self.normal.as_vector()).abs();
        let dot_vn = self.v_axis.as_vector().dot(&self.normal.as_vector()).abs();

        let tolerance = T::from_f64(1e-10);
        dot_uv < tolerance && dot_un < tolerance && dot_vn < tolerance
    }
}

impl<T: Scalar> Default for Plane3D<T> {
    /// デフォルトはXY平面（z = 0）
    fn default() -> Self {
        Self::xy_plane(T::ZERO)
    }
}

// ============================================================================
// Constants (注: ジェネリック型では const は制限があるため、メソッドで提供)
// ============================================================================

impl<T: Scalar> Plane3D<T> {
    /// XY平面（z = 0）の参照
    pub fn xy() -> Self {
        Self::xy_plane(T::ZERO)
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar + std::fmt::Display> std::fmt::Display for Plane3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plane3D(origin: ({}, {}, {}), normal: ({}, {}, {}), u_axis: ({}, {}, {}), v_axis: ({}, {}, {}))",
            self.origin.x(),
            self.origin.y(),
            self.origin.z(),
            self.normal.x(),
            self.normal.y(),
            self.normal.z(),
            self.u_axis.x(),
            self.u_axis.y(),
            self.u_axis.z(),
            self.v_axis.x(),
            self.v_axis.y(),
            self.v_axis.z()
        )
    }
}
