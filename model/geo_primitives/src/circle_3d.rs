//! Circle3D - Core Implementation
//!
//! 3次元円の基本実装とコンストラクタ、アクセサメソッド

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元空間の円
///
/// 3次元空間内の任意の平面上に存在する円を表現
#[derive(Debug, Clone, PartialEq)]
pub struct Circle3D<T: Scalar> {
    center: Point3D<T>,
    normal: Direction3D<T>, // 円が存在する平面の法線ベクトル
    radius: T,
}

impl<T: Scalar> Circle3D<T> {
    /// 新しい円を作成
    ///
    /// # 引数
    /// * `center` - 円の中心点
    /// * `normal` - 円が存在する平面の法線方向（正規化済み）
    /// * `radius` - 円の半径（正の値である必要がある）
    ///
    /// # 戻り値
    /// * `Some(Circle3D)` - 有効な円が作成できた場合
    /// * `None` - 半径が0以下の場合
    pub fn new(center: Point3D<T>, normal: Direction3D<T>, radius: T) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        Some(Self {
            center,
            normal,
            radius,
        })
    }

    /// Vector3Dから円を作成（後方互換性）
    pub fn from_vector(center: Point3D<T>, normal: Vector3D<T>, radius: T) -> Option<Self> {
        let normal_dir = Direction3D::from_vector(normal)?;
        Self::new(center, normal_dir, radius)
    }

    /// XY平面上の円を作成（Z軸が法線）
    pub fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            radius,
        )
    }

    /// XZ平面上の円を作成（Y軸が法線）
    pub fn new_xz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::from_vector(Vector3D::unit_y()).unwrap(),
            radius,
        )
    }

    /// YZ平面上の円を作成（X軸が法線）
    pub fn new_yz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(
            center,
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            radius,
        )
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 直径を取得
    pub fn diameter(&self) -> T {
        self.radius + self.radius // 2 * radius
    }

    /// 円周を計算
    pub fn circumference(&self) -> T {
        T::TAU * self.radius // 2π * radius
    }

    /// 面積を計算
    pub fn area(&self) -> T {
        T::PI * self.radius * self.radius
    }
}
