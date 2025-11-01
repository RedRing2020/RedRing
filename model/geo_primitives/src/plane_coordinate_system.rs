//! STEP準拠の3次元平面座標系実装
//!
//! STEP AP214の AXIS2_PLACEMENT_3D + PLANE に対応する拡張平面型
//!
//! **作成日: 2025年10月28日**
//! **最終更新: 2025年10月29日**
//!
//! ## 設計方針
//! - 既存のPlane3D（point + normal）は下位互換で維持
//! - Plane3DCoordinateSystem（origin + normal + u_axis + v_axis）を新規追加
//! - STEP標準準拠とCAD実用性を両立
//!
//! ## STEP標準対応
//! ```step
//! PLANE('', AXIS2_PLACEMENT_3D('', POINT, AXIS, REF_DIRECTION));
//! ```
//! - location: 平面原点（origin）
//! - axis: Z軸方向（normal）- 法線ベクトル
//! - ref_direction: X軸方向（u_axis）- 第一軸
//! - derived Y軸: normal × u_axis で自動計算（v_axis）

use crate::{Direction3D, Plane3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// STEP準拠の3次元平面座標系（Plane3Dの拡張版）
///
/// STEP AP214の AXIS2_PLACEMENT_3D エンティティに対応した平面定義
///
/// ## 座標系定義
/// - origin: 平面原点（STEP: location）
/// - normal: Z軸方向（STEP: axis）- 法線ベクトル
/// - u_axis: X軸方向（STEP: ref_direction）- 第一軸
/// - v_axis: Y軸方向（STEP: derived）- normal × u_axis で自動計算
///
/// ## CAD用途
/// - スケッチ平面の基準座標系
/// - テクスチャマッピングのUV座標系
/// - 2Dパターン配置の基準面
/// - 断面生成時の方向制御
/// - STEPファイルとの相互変換
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane3DCoordinateSystem<T: Scalar> {
    /// 平面原点（STEP: location）
    origin: Point3D<T>,

    /// Z軸方向 - 法線ベクトル（STEP: axis）
    /// Direction3D<T>により正規化が保証される
    normal: Direction3D<T>,

    /// X軸方向 - 第一軸（STEP: ref_direction）
    /// Direction3D<T>により正規化が保証される
    u_axis: Direction3D<T>,

    /// Y軸方向 - 第二軸（STEP: derived, normal × u_axis）
    /// Direction3D<T>により正規化が保証される
    v_axis: Direction3D<T>,
}

impl<T: Scalar> Plane3DCoordinateSystem<T> {
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
    ///
    /// # アルゴリズム
    /// 1. 法線を正規化してZ軸とする
    /// 2. U方向から法線成分を除去（グラム・シュミット正規直交化）
    /// 3. 正規化してU軸とする
    /// 4. V軸 = Z軸 × U軸（右手系）
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
    ///
    /// # Algorithm
    /// 1. origin → point_u ベクトルをU方向とする
    /// 2. origin → point_v ベクトルをV方向とする
    /// 3. 法線 = U × V で計算
    /// 4. from_origin_and_axes で座標系を構築
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

    /// 従来のPlane3Dから座標系を作成
    ///
    /// # Arguments
    /// * `plane` - 従来の平面定義
    /// * `preferred_u_direction` - 優先するU軸方向（オプション）
    ///
    /// # Algorithm
    /// 優先U軸方向が指定されない場合：
    /// 1. ワールドX軸を基準とする
    /// 2. 法線がX軸にほぼ平行の場合はY軸を使用
    pub fn from_simple_plane(
        plane: &Plane3D<T>,
        preferred_u_direction: Option<Vector3D<T>>,
    ) -> Option<Self> {
        let u_direction = preferred_u_direction.unwrap_or_else(|| {
            // デフォルト：ワールドX軸を基準
            let world_x = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
            let world_y = Vector3D::new(T::ZERO, T::ONE, T::ZERO);

            // 法線がX軸にほぼ平行の場合はY軸を使用
            let dot_with_x = plane.normal().dot(&world_x).abs();
            if dot_with_x > T::from_f64(0.9) {
                world_y
            } else {
                world_x
            }
        });

        Self::from_origin_and_axes(plane.point(), plane.normal(), u_direction)
    }

    // ========================================================================
    // 座標変換メソッド
    // ========================================================================

    /// 平面ローカル座標をワールド座標に変換
    ///
    /// # Arguments
    /// * `u` - U軸座標
    /// * `v` - V軸座標
    ///
    /// # Returns
    /// ワールド座標系での点
    ///
    /// # Formula
    /// world_point = origin + u * u_axis + v * v_axis
    pub fn local_to_world(&self, u: T, v: T) -> Point3D<T> {
        Point3D::new(
            self.origin.x() + u * self.u_axis.x() + v * self.v_axis.x(),
            self.origin.y() + u * self.u_axis.y() + v * self.v_axis.y(),
            self.origin.z() + u * self.u_axis.z() + v * self.v_axis.z(),
        )
    }

    /// ワールド座標を平面ローカル座標に変換
    ///
    /// # Arguments
    /// * `world_point` - ワールド座標系の点
    ///
    /// # Returns
    /// (u, v, distance_from_plane) - 平面上の投影点のUV座標と平面からの距離
    ///
    /// # Algorithm
    /// 1. 原点からの相対ベクトルを計算
    /// 2. U軸、V軸、法線への射影を計算
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
    ///
    /// # Arguments
    /// * `world_point` - 投影する点
    ///
    /// # Returns
    /// (u, v) - 平面上の投影点のUV座標
    pub fn project_to_uv(&self, world_point: Point3D<T>) -> (T, T) {
        let (u, v, _) = self.world_to_local(world_point);
        (u, v)
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

    /// 従来のPlane3D形式に変換
    pub fn to_simple_plane(&self) -> Plane3D<T> {
        Plane3D::from_point_and_normal(self.origin, self.normal.as_vector()).unwrap()
    }

    // ========================================================================
    // CAD操作支援
    // ========================================================================

    /// スケッチ座標系のマトリックスを取得（4x4変換行列）
    ///
    /// # Returns
    /// 変換行列:
    /// ```text
    /// [u_axis.x, v_axis.x, normal.x, origin.x]
    /// [u_axis.y, v_axis.y, normal.y, origin.y]
    /// [u_axis.z, v_axis.z, normal.z, origin.z]
    /// [0,        0,        0,        1       ]
    /// ```
    pub fn transformation_matrix(&self) -> [[T; 4]; 4] {
        [
            [
                self.u_axis.x(),
                self.v_axis.x(),
                self.normal.x(),
                self.origin.x(),
            ],
            [
                self.u_axis.y(),
                self.v_axis.y(),
                self.normal.y(),
                self.origin.y(),
            ],
            [
                self.u_axis.z(),
                self.v_axis.z(),
                self.normal.z(),
                self.origin.z(),
            ],
            [T::ZERO, T::ZERO, T::ZERO, T::ONE],
        ]
    }

    /// 2Dスケッチパターンを3D配置
    ///
    /// # Arguments
    /// * `pattern_points` - 平面上のUV座標パターン
    ///
    /// # Returns
    /// ワールド座標系での3D点群
    pub fn pattern_to_3d(&self, pattern_points: &[(T, T)]) -> Vec<Point3D<T>> {
        pattern_points
            .iter()
            .map(|(u, v)| self.local_to_world(*u, *v))
            .collect()
    }

    /// 平面上での回転変換
    ///
    /// # Arguments
    /// * `angle` - 回転角度（ラジアン）
    /// * `rotation_center_uv` - 回転中心のUV座標
    ///
    /// # Returns
    /// 回転後の平面座標系
    pub fn rotate_in_plane(&self, angle: T, _rotation_center_uv: (T, T)) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 新しいU軸とV軸の計算（原点中心回転）
        let new_u_axis = Direction3D::from_vector(
            self.u_axis.as_vector() * cos_a + self.v_axis.as_vector() * sin_a,
        )
        .unwrap();

        let new_v_axis = Direction3D::from_vector(
            -self.u_axis.as_vector() * sin_a + self.v_axis.as_vector() * cos_a,
        )
        .unwrap();

        Self {
            origin: self.origin,
            normal: self.normal,
            u_axis: new_u_axis,
            v_axis: new_v_axis,
        }
    }
}

// ============================================================================
// 従来のPlane3Dとの相互変換
// ============================================================================

impl<T: Scalar> From<Plane3DCoordinateSystem<T>> for Plane3D<T> {
    fn from(coord_system: Plane3DCoordinateSystem<T>) -> Self {
        coord_system.to_simple_plane()
    }
}

// 注意: Plane3D -> Plane3DCoordinateSystem の自動変換は、
// U軸方向が自動決定されるため、意図しない結果になる可能性があります。
// 明示的にfrom_simple_plane()を使用することを推奨します。
