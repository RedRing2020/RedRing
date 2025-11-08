// torus_solid_3d.rs
// STEP AP214 準拠のトーラス固体実装
//
// トーラス固体は主半径（major_radius）と副半径（minor_radius）を持つ回転固体です。
// 3D CAM での固体加工計算において必須の幾何要素です。
//
// STEP エンティティ: TORUS + AXIS2_PLACEMENT_3D
// 固体としての体積と表面を持ちます。

use crate::{Direction3D, Point3D, TorusSurface3D, Vector3D};
use geo_foundation::Scalar;
use std::f64::consts::PI;

/// STEP AP214 準拠のトーラス固体
///
/// 3D CAM 固体加工計算における基本幾何要素として実装。
/// 主半径（ドーナツの中心軸から管の中心までの距離）と
/// 副半径（管の半径）により定義される3次元固体です。
#[derive(Debug, Clone, PartialEq)]
pub struct TorusSolid3D<T: Scalar> {
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

impl<T: Scalar> TorusSolid3D<T> {
    /// 新しいトーラス固体を作成
    ///
    /// # Arguments
    /// * `origin` - トーラスの中心点
    /// * `z_axis` - トーラスの軸方向
    /// * `x_axis` - 主半径の基準方向（z_axisと直交である必要があります）
    /// * `major_radius` - 主半径（正の値である必要があります）
    /// * `minor_radius` - 副半径（正の値である必要があります）
    ///
    /// # Returns
    /// * `Some(TorusSolid3D)` - 有効なパラメータの場合
    /// * `None` - 無効なパラメータの場合（半径が非正、軸が非直交など）
    pub fn new(
        origin: Point3D<T>,
        z_axis: Direction3D<T>,
        x_axis: Direction3D<T>,
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self> {
        // 半径の妥当性チェック
        if major_radius <= T::ZERO || minor_radius <= T::ZERO {
            return None;
        }

        // トーラス固体の幾何学的制約: major_radius > minor_radius
        // （これにより、固体が適切にドーナツ形状になる）
        if major_radius <= minor_radius {
            return None;
        }

        // 軸の直交性チェック
        let dot_product =
            x_axis.x() * z_axis.x() + x_axis.y() * z_axis.y() + x_axis.z() * z_axis.z();
        let tolerance = T::EPSILON;
        if dot_product.abs() > tolerance {
            return None;
        }

        Some(TorusSolid3D {
            origin,
            z_axis,
            x_axis,
            major_radius,
            minor_radius,
        })
    }

    /// 標準的なトーラス固体を作成（Z軸中心、原点配置）
    ///
    /// # Arguments
    /// * `major_radius` - 主半径
    /// * `minor_radius` - 副半径
    ///
    /// # Returns
    /// * `Some(TorusSolid3D)` - 有効なパラメータの場合
    /// * `None` - 無効なパラメータの場合
    pub fn standard(major_radius: T, minor_radius: T) -> Option<Self> {
        let origin = Point3D::origin();
        let z_axis = Direction3D::from_vector(Vector3D::new(T::ZERO, T::ZERO, T::ONE))?;
        let x_axis = Direction3D::from_vector(Vector3D::new(T::ONE, T::ZERO, T::ZERO))?;

        Self::new(origin, z_axis, x_axis, major_radius, minor_radius)
    }

    /// 最小有効トーラス固体を作成（テスト用）
    pub fn minimal() -> Option<Self> {
        let major_radius = T::from_f64(2.0);
        let minor_radius = T::ONE;
        Self::standard(major_radius, minor_radius)
    }

    // アクセサメソッド
    pub fn origin(&self) -> &Point3D<T> {
        &self.origin
    }
    pub fn z_axis(&self) -> &Direction3D<T> {
        &self.z_axis
    }
    pub fn x_axis(&self) -> &Direction3D<T> {
        &self.x_axis
    }
    pub fn major_radius(&self) -> T {
        self.major_radius
    }
    pub fn minor_radius(&self) -> T {
        self.minor_radius
    }

    /// Y軸方向を計算（右手座標系）
    pub fn y_axis(&self) -> Direction3D<T> {
        let y_vec = Vector3D::new(
            self.z_axis.y() * self.x_axis.z() - self.z_axis.z() * self.x_axis.y(),
            self.z_axis.z() * self.x_axis.x() - self.z_axis.x() * self.x_axis.z(),
            self.z_axis.x() * self.x_axis.y() - self.z_axis.y() * self.x_axis.x(),
        );
        Direction3D::from_vector(y_vec).unwrap_or_else(|| {
            Direction3D::from_vector(Vector3D::new(T::ZERO, T::ONE, T::ZERO)).unwrap()
        })
    }

    /// 対応するトーラス面を取得
    ///
    /// トーラス固体の外表面に対応するトーラス面を返します。
    pub fn outer_surface(&self) -> Option<TorusSurface3D<T>> {
        TorusSurface3D::new(
            self.origin,
            self.z_axis,
            self.x_axis,
            self.major_radius,
            self.minor_radius,
        )
    }

    /// 体積を計算
    ///
    /// トーラス固体の体積 = 2π²R²r (R=主半径, r=副半径)
    pub fn volume(&self) -> T {
        let two = T::from_f64(2.0);
        let pi_squared = T::from_f64(PI * PI);

        two * pi_squared * self.major_radius * self.major_radius * self.minor_radius
    }

    /// 表面積を計算
    ///
    /// トーラス固体の表面積 = 4π²Rr (R=主半径, r=副半径)
    pub fn surface_area(&self) -> T {
        let four = T::from_f64(4.0);
        let pi_squared = T::from_f64(PI * PI);

        four * pi_squared * self.major_radius * self.minor_radius
    }

    /// 点が固体内部にあるかを判定
    ///
    /// # Arguments
    /// * `point` - 判定する点
    ///
    /// # Returns
    /// * `true` - 点が固体内部にある場合
    /// * `false` - 点が固体外部にある場合
    pub fn contains_point(&self, point: &Point3D<T>) -> bool {
        // 原点からの相対位置ベクトル
        let relative = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );

        // Z軸方向の成分
        let z_component = relative.x() * self.z_axis.x()
            + relative.y() * self.z_axis.y()
            + relative.z() * self.z_axis.z();

        // XY平面への投影
        let x_component = relative.x() * self.x_axis.x()
            + relative.y() * self.x_axis.y()
            + relative.z() * self.x_axis.z();

        let y_axis = self.y_axis();
        let y_component =
            relative.x() * y_axis.x() + relative.y() * y_axis.y() + relative.z() * y_axis.z();

        // 中心軸からの距離
        let radial_distance = (x_component * x_component + y_component * y_component).sqrt();

        // トーラス中心線からの距離
        let torus_center_distance = (radial_distance - self.major_radius).abs();

        // 副半径内にあるかチェック
        let cross_section_distance =
            (z_component * z_component + torus_center_distance * torus_center_distance).sqrt();

        cross_section_distance <= self.minor_radius
    }
}
