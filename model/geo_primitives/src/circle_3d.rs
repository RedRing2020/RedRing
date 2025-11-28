//! Circle3D - Core Implementation
//!
//! 3次元円の基本実装とコンストラクタ、アクセサメソッド
//! STEP (ISO 10303) 準拠の axis2_placement_3d スタイルで実装

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::{
    core::circle_core_traits::{Circle3DConstructor, Circle3DMeasure, Circle3DProperties},
    Scalar,
};

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

    /// 点が円内部にあるか判定（3D空間での判定）
    pub fn contains_point_3d(&self, point: Point3D<T>) -> bool {
        // 点から中心へのベクトル
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 平面上にあるかチェック（法線との内積が0）
        let axis_vec = self.axis.as_vector();
        let dot = to_point.x() * axis_vec.x() + to_point.y() * axis_vec.y() + to_point.z() * axis_vec.z();
        if dot.abs() > T::EPSILON {
            return false; // 平面上にない
        }

        // 中心からの距離をチェック
        let distance_squared = to_point.x() * to_point.x() + to_point.y() * to_point.y() + to_point.z() * to_point.z();
        distance_squared <= self.radius * self.radius
    }

    /// 点から円周への距離（3D空間）
    pub fn distance_to_point_3d(&self, point: Point3D<T>) -> T {
        // 点から中心へのベクトル
        let to_point = Vector3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );

        // 平面への投影距離（法線方向成分）
        let axis_vec = self.axis.as_vector();
        let plane_distance = (to_point.x() * axis_vec.x() + to_point.y() * axis_vec.y() + to_point.z() * axis_vec.z()).abs();

        // 平面上での中心からの距離
        let distance_squared = to_point.x() * to_point.x() + to_point.y() * to_point.y() + to_point.z() * to_point.z();
        let planar_distance_squared = distance_squared - plane_distance * plane_distance;
        let planar_distance = planar_distance_squared.max(T::ZERO).sqrt();

        // 円周への距離
        let radial_distance = (planar_distance - self.radius).abs();

        // 平面距離と半径方向距離の合成
        (plane_distance * plane_distance + radial_distance * radial_distance).sqrt()
    }
}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> Circle3DConstructor<T> for Circle3D<T> {
    fn new(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let axis_vec = Vector3D::new(axis.0, axis.1, axis.2);
        let axis_dir = Direction3D::from_vector(axis_vec)?;
        Self::new(center_point, axis_dir, radius)
    }

    fn new_xy_plane(center: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        Self::new_xy_plane(center_point, radius)
    }

    fn unit_circle_xy() -> Self {
        let center = Point3D::origin();
        Self::new_xy_plane(center, T::ONE).unwrap()
    }
}

impl<T: Scalar> Circle3DProperties<T> for Circle3D<T> {
    fn center(&self) -> (T, T, T) {
        (self.center.x(), self.center.y(), self.center.z())
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn axis(&self) -> (T, T, T) {
        (self.axis.x(), self.axis.y(), self.axis.z())
    }

    fn ref_direction(&self) -> (T, T, T) {
        (
            self.ref_direction.x(),
            self.ref_direction.y(),
            self.ref_direction.z(),
        )
    }

    fn dimension(&self) -> u32 {
        3
    }
}

impl<T: Scalar> Circle3DMeasure<T> for Circle3D<T> {
    fn circumference(&self) -> T {
        self.circumference()
    }

    fn area(&self) -> T {
        self.area()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        self.contains_point_3d(p)
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let p = Point3D::new(point.0, point.1, point.2);
        self.distance_to_point_3d(p)
    }
}
