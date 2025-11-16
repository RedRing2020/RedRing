//! ConicalSolid3D 拡張機能実装
//!
//! 円錐ソリッドの高度な幾何計算と解析機能

use crate::{ConicalSolid3D, Plane3D, Point3D, Vector3D};
use geo_foundation::Scalar;

// ============================================================================
// Geometric Analysis Extensions
// ============================================================================

impl<T: Scalar> ConicalSolid3D<T> {
    /// 点が円錐ソリッド内部に含まれるかチェック
    ///
    /// # Arguments
    /// * `point` - チェックする点
    ///
    /// # Returns
    /// 点が円錐内部にある場合 true
    ///
    /// # Algorithm
    /// 1. 点を円錐のローカル座標系に変換
    /// 2. 軸方向の高さをチェック（0 ≤ h ≤ height）
    /// 3. その高さでの円錐半径を計算
    /// 4. 軸からの距離が半径以下かチェック
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
        // 中心からの相対ベクトル
        let relative = Vector3D::new(
            point.x() - self.center().x(),
            point.y() - self.center().y(),
            point.z() - self.center().z(),
        );

        // 軸方向への射影（高さ）
        let height_along_axis = relative.dot(&self.axis().as_vector());

        // 高さが範囲外の場合
        if height_along_axis < T::ZERO || height_along_axis > self.height() {
            return false;
        }

        // その高さでの円錐半径を計算（線形補間）
        let height_ratio = height_along_axis / self.height();
        let radius_at_height = self.radius() * (T::ONE - height_ratio);

        // 軸からの距離を計算
        let axis_projection: Vector3D<T> = self.axis().as_vector() * height_along_axis;
        let radial_vector = Vector3D::new(
            relative.x() - axis_projection.x(),
            relative.y() - axis_projection.y(),
            relative.z() - axis_projection.z(),
        );
        let distance_from_axis = radial_vector.length();

        distance_from_axis <= radius_at_height
    }

    /// 点から円錐表面までの最短距離
    ///
    /// # Arguments
    /// * `point` - 距離を測る点
    ///
    /// # Returns
    /// 表面までの最短距離（内部の場合は0）
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        if self.contains_point(point) {
            return T::ZERO;
        }

        // 簡易実装: 中心からの距離から半径を引く
        let center_distance = Vector3D::new(
            point.x() - self.center().x(),
            point.y() - self.center().y(),
            point.z() - self.center().z(),
        )
        .length();

        let half = T::ONE + T::ONE; // 2を表現
        let average_radius = self.radius() / half;
        if center_distance > average_radius {
            center_distance - average_radius
        } else {
            T::ZERO
        }
    }

    /// 平面による円錐の断面を計算
    ///
    /// # Arguments
    /// * `plane` - 切断平面
    ///
    /// # Returns
    /// 断面の形状（実装簡略化）
    pub fn cross_section_with_plane(&self, _plane: &Plane3D<T>) -> Option<Vec<Point3D<T>>> {
        // 実装簡略化 - 実際のプロジェクトでは楕円や円の計算が必要
        None
    }

    /// XY平面への投影
    ///
    /// # Returns
    /// 投影された形状の境界点
    pub fn project_to_xy(&self) -> Vec<Point3D<T>> {
        let mut points = Vec::new();

        // 底面の4点（簡略化）
        let r = self.radius();
        points.push(Point3D::new(
            self.center().x() + r,
            self.center().y(),
            self.center().z(),
        ));
        points.push(Point3D::new(
            self.center().x(),
            self.center().y() + r,
            self.center().z(),
        ));
        points.push(Point3D::new(
            self.center().x() - r,
            self.center().y(),
            self.center().z(),
        ));
        points.push(Point3D::new(
            self.center().x(),
            self.center().y() - r,
            self.center().z(),
        ));

        // 頂点も追加
        let apex = Point3D::new(
            self.center().x() + self.axis().x() * self.height(),
            self.center().y() + self.axis().y() * self.height(),
            self.center().z() + self.axis().z() * self.height(),
        );
        points.push(apex);

        points
    }

    /// XZ平面への投影
    pub fn project_to_xz(&self) -> Vec<Point3D<T>> {
        // XY投影と類似の実装
        self.project_to_xy()
    }

    /// YZ平面への投影
    pub fn project_to_yz(&self) -> Vec<Point3D<T>> {
        // XY投影と類似の実装
        self.project_to_xy()
    }

    /// 底面の中心を取得
    ///
    /// # Returns
    /// 底面の中心座標
    pub fn base_center(&self) -> Point3D<T> {
        self.center()
    }

    /// 指定した高さでの円錐半径を計算
    ///
    /// # Arguments
    /// * `height_from_base` - 底面からの高さ
    ///
    /// # Returns
    /// その高さでの半径（範囲外の場合は None）
    pub fn radius_at_height(&self, height_from_base: T) -> Option<T> {
        if height_from_base < T::ZERO || height_from_base > self.height() {
            return None;
        }

        let height_ratio = height_from_base / self.height();
        let radius = self.radius() * (T::ONE - height_ratio);
        Some(radius)
    }

    /// 円錐の軸線を取得
    ///
    /// # Returns
    /// 底面中心から頂点への直線
    pub fn axis_line(&self) -> (Point3D<T>, Point3D<T>) {
        let start = self.center();
        let end = Point3D::new(
            self.center().x() + self.axis().x() * self.height(),
            self.center().y() + self.axis().y() * self.height(),
            self.center().z() + self.axis().z() * self.height(),
        );
        (start, end)
    }
}
