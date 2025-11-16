//! Circle3D - Core Implementation
//!
//! 3次元円の基本実装とコンストラクタ、アクセサメソッド
//! STEP (ISO 10303) 準拠の axis2_placement_3d スタイルで実装

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元空間の円
///
/// STEP準拠でaxis（法線）とref_direction（参照方向）を持ち、
/// 3D空間での完全な座標系定義とArc3D変換に対応
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle3D<T: Scalar> {
    center: Point3D<T>,
    /// Z軸方向（円が存在する平面の法線ベクトル）
    axis: Direction3D<T>,
    /// X軸方向（参照方向、角度0度の方向）
    ref_direction: Direction3D<T>,
    radius: T,
}

impl<T: Scalar> Circle3D<T> {
    /// 新しい円を作成（デフォルトでX軸正方向を参照方向とする）
    ///
    /// # 引数
    /// * `center` - 円の中心点
    /// * `axis` - 円が存在する平面の法線方向（Z軸）
    /// * `radius` - 円の半径（正の値である必要がある）
    ///
    /// # 戻り値
    /// * `Some(Circle3D)` - 有効な円が作成できた場合
    /// * `None` - 半径が0以下の場合
    pub fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // デフォルトのref_directionを計算（axisに直交する方向）
        let ref_direction = Self::compute_default_ref_direction(&axis);

        Some(Self {
            center,
            axis,
            ref_direction,
            radius,
        })
    }

    /// 完全な座標系で円を作成
    pub fn new_with_ref_direction(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        // ref_directionがaxisに直交しているか確認
        let dot_product = axis.x() * ref_direction.x()
            + axis.y() * ref_direction.y()
            + axis.z() * ref_direction.z();
        if dot_product.abs() > T::EPSILON {
            return None; // 直交していない
        }

        Some(Self {
            center,
            axis,
            ref_direction,
            radius,
        })
    }

    /// axisに直交するデフォルトref_directionを計算
    fn compute_default_ref_direction(axis: &Direction3D<T>) -> Direction3D<T> {
        // axisに直交する方向を得る
        // まずX軸との外積を試す
        let x_axis = Vector3D::unit_x();
        let cross_x = Vector3D::new(
            axis.y() * x_axis.z() - axis.z() * x_axis.y(),
            axis.z() * x_axis.x() - axis.x() * x_axis.z(),
            axis.x() * x_axis.y() - axis.y() * x_axis.x(),
        );

        if cross_x.length() > T::EPSILON {
            Direction3D::from_vector(cross_x).unwrap()
        } else {
            // axisがX軸と平行な場合はY軸との外積を使用
            let y_axis = Vector3D::unit_y();
            let cross_y = Vector3D::new(
                axis.y() * y_axis.z() - axis.z() * y_axis.y(),
                axis.z() * y_axis.x() - axis.x() * y_axis.z(),
                axis.x() * y_axis.y() - axis.y() * y_axis.x(),
            );
            Direction3D::from_vector(cross_y).unwrap()
        }
    }

    /// Vector3Dから円を作成（後方互換性）
    pub fn from_vector(center: Point3D<T>, axis_vector: Vector3D<T>, radius: T) -> Option<Self> {
        let axis_dir = Direction3D::from_vector(axis_vector)?;
        Self::new(center, axis_dir, radius)
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

    /// Z軸方向（法線ベクトル）を取得
    pub fn axis(&self) -> Direction3D<T> {
        self.axis
    }

    /// 法線ベクトルを取得（後方互換性）
    pub fn normal(&self) -> Direction3D<T> {
        self.axis
    }

    /// X軸方向（参照方向）を取得
    pub fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction
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
