// torus_surface_3d.rs
// STEP AP214 準拠のトーラス面実装
//
// トーラス面は主半径（major_radius）と副半径（minor_radius）を持つ回転面です。
// 3D CAM での工具オフセット計算において必須の幾何要素です。
//
// STEP エンティティ: TOROIDAL_SURFACE + AXIS2_PLACEMENT_3D
// パラメータ範囲: u ∈ [0, 2π], v ∈ [0, 2π]

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;
use std::f64::consts::PI;

/// STEP AP214 準拠のトーラス面
///
/// 3D CAM 工具オフセット計算における基本幾何要素として実装。
/// 主半径（ドーナツの中心軸から管の中心までの距離）と
/// 副半径（管の半径）により定義されます。
#[derive(Debug, Clone, PartialEq)]
pub struct TorusSurface3D<T: Scalar> {
    /// 原点（トーラスの中心点）
    origin: Point3D<T>,
    /// Z軸方向（トーラスの軸方向）
    z_axis: Direction3D<T>,
    /// X軸方向（主半径の基準方向）
    x_axis: Direction3D<T>,
    /// 主半径（ドーナツの中心軸から管の中心までの距離）
    major_radius: T,
    /// 副半径（管の半径）
    minor_radius: T,
}

impl<T: Scalar> TorusSurface3D<T> {
    /// 新しいトーラス面を作成
    ///
    /// # Arguments
    /// * `origin` - トーラスの中心点
    /// * `z_axis` - トーラスの軸方向
    /// * `x_axis` - 主半径の基準方向（z_axisと直交である必要があります）
    /// * `major_radius` - 主半径（正の値である必要があります）
    /// * `minor_radius` - 副半径（正の値である必要があります）
    ///
    /// # Returns
    /// * `Some(TorusSurface3D)` - 有効なパラメータの場合
    /// * `None` - 無効なパラメータの場合（半径が非正、軸が非直交など）
    pub fn new(
        origin: Point3D<T>,
        z_axis: Direction3D<T>,
        x_axis: Direction3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        // パラメータ検証
        if major_radius <= T::ZERO || minor_radius <= T::ZERO {
            return None;
        }

        // 軸の直交性チェック
        let z_vec = Vector3D::new(z_axis.x(), z_axis.y(), z_axis.z());
        let x_vec = Vector3D::new(x_axis.x(), x_axis.y(), x_axis.z());
        let dot_product = z_vec.dot(&x_vec);

        if dot_product.abs() > T::EPSILON {
            return None; // 軸が直交していない
        }

        Some(TorusSurface3D {
            origin,
            z_axis,
            x_axis,
            major_radius,
            minor_radius,
        })
    }

    /// 標準的なトーラス面を作成（XY平面上、原点中心）
    ///
    /// # Arguments
    /// * `major_radius` - 主半径
    /// * `minor_radius` - 副半径
    pub fn standard(major_radius: T, minor_radius: T) -> Option<Self> {
        let origin = Point3D::origin();
        let z_axis = Direction3D::from_vector(Vector3D::new(T::ZERO, T::ZERO, T::ONE))?;
        let x_axis = Direction3D::from_vector(Vector3D::new(T::ONE, T::ZERO, T::ZERO))?;

        Self::new(origin, z_axis, x_axis, major_radius, minor_radius)
    }

    /// 原点を取得
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// Z軸方向を取得
    pub fn z_axis(&self) -> Direction3D<T> {
        self.z_axis
    }

    /// X軸方向を取得
    pub fn x_axis(&self) -> Direction3D<T> {
        self.x_axis
    }

    /// Y軸方向を計算（右手座標系）
    pub fn y_axis(&self) -> Direction3D<T> {
        let z_vec = Vector3D::new(self.z_axis.x(), self.z_axis.y(), self.z_axis.z());
        let x_vec = Vector3D::new(self.x_axis.x(), self.x_axis.y(), self.x_axis.z());
        let y_vec = z_vec.cross(&x_vec);

        // 正規化されたベクトルのクロス積は正規化済み
        Direction3D::from_vector(y_vec).unwrap()
    }

    /// 主半径を取得
    pub fn major_radius(&self) -> T {
        self.major_radius
    }

    /// 副半径を取得
    pub fn minor_radius(&self) -> T {
        self.minor_radius
    }

    /// パラメータ (u, v) での点を計算
    ///
    /// u: 主方向角度 [0, 2π]
    /// v: 副方向角度 [0, 2π]
    ///
    /// # Mathematical Formula
    /// ```text
    /// P(u,v) = origin +
    ///          (major_radius + minor_radius * cos(v)) * (cos(u) * x_axis + sin(u) * y_axis) +
    ///          minor_radius * sin(v) * z_axis
    /// ```
    pub fn point_at(&self, u: T, v: T) -> Point3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();

        let y_axis = self.y_axis();

        // 管の中心を計算
        let radius_at_v = self.major_radius + self.minor_radius * cos_v;

        // 各軸成分を計算
        let x_component = radius_at_v * cos_u;
        let y_component = radius_at_v * sin_u;
        let z_component = self.minor_radius * sin_v;

        // 局所座標系から世界座標系への変換
        let x_contribution =
            Vector3D::new(self.x_axis.x(), self.x_axis.y(), self.x_axis.z()) * x_component;
        let y_contribution = Vector3D::new(y_axis.x(), y_axis.y(), y_axis.z()) * y_component;
        let z_contribution =
            Vector3D::new(self.z_axis.x(), self.z_axis.y(), self.z_axis.z()) * z_component;

        let total_offset = x_contribution + y_contribution + z_contribution;

        Point3D::new(
            self.origin.x() + total_offset.x(),
            self.origin.y() + total_offset.y(),
            self.origin.z() + total_offset.z(),
        )
    }

    /// パラメータ (u, v) での法線ベクトルを計算
    ///
    /// CAM 工具オフセット計算において重要な機能です。
    pub fn normal_at(&self, u: T, v: T) -> Direction3D<T> {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let cos_v = v.cos();
        let sin_v = v.sin();

        let y_axis = self.y_axis();

        // 法線ベクトルの計算（外向き法線）
        let radial_component = cos_v;
        let axial_component = sin_v;

        let x_part = radial_component * cos_u;
        let y_part = radial_component * sin_u;
        let z_part = axial_component;

        // 局所座標系から世界座標系への変換
        let normal_vec = Vector3D::new(self.x_axis.x(), self.x_axis.y(), self.x_axis.z()) * x_part
            + Vector3D::new(y_axis.x(), y_axis.y(), y_axis.z()) * y_part
            + Vector3D::new(self.z_axis.x(), self.z_axis.y(), self.z_axis.z()) * z_part;

        Direction3D::from_vector(normal_vec).unwrap_or(
            // フォールバック: Z軸方向
            self.z_axis,
        )
    }

    /// 表面積を計算
    ///
    /// 数学的公式: 4π² * major_radius * minor_radius
    pub fn surface_area(&self) -> T {
        let two = T::ONE + T::ONE;
        let four = two + two;
        // π² の値を直接使用
        let pi_squared = T::from_f64(PI * PI);

        four * pi_squared * self.major_radius * self.minor_radius
    }

    /// トーラスが有効かどうかをチェック
    ///
    /// 有効な条件:
    /// - 主半径 > 0
    /// - 副半径 > 0
    /// - 軸が単位ベクトル
    /// - 軸が直交
    pub fn is_valid(&self) -> bool {
        if self.major_radius <= T::ZERO || self.minor_radius <= T::ZERO {
            return false;
        }

        // 軸の直交性チェック
        let z_vec = Vector3D::new(self.z_axis.x(), self.z_axis.y(), self.z_axis.z());
        let x_vec = Vector3D::new(self.x_axis.x(), self.x_axis.y(), self.x_axis.z());
        let dot_product = z_vec.dot(&x_vec);

        dot_product.abs() <= T::EPSILON
    }
}

/// f64 での特別な実装
impl TorusSurface3D<f64> {
    /// ドーナツ型トーラス（major_radius > minor_radius）
    pub fn donut(major_radius: f64, minor_radius: f64) -> Option<Self> {
        if major_radius <= minor_radius {
            return None; // ドーナツ型ではない
        }
        Self::standard(major_radius, minor_radius)
    }

    /// 角の近似値を計算（CAM での角度計算用）
    pub fn corner_angle_at(&self, _u: f64, v: f64) -> f64 {
        // 主曲率と副曲率から角度を近似計算
        let principal_curvature_u = 1.0 / (self.major_radius + self.minor_radius * v.cos());
        let principal_curvature_v = 1.0 / self.minor_radius;

        // ガウス曲率から角度を近似
        let gaussian_curvature = principal_curvature_u * principal_curvature_v;
        gaussian_curvature.abs().sqrt()
    }
}
