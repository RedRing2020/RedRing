//! ConicalSolid3D 拡張機能実装
//!
//! 円錐ソリッドの高度な幾何計算と解析機能

use crate::{ConicalSolid3D, Plane3D, Point3D, Vector3D};
use geo_contracts::{PointClassification, Scalar};

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
        let center = self.center_internal();
        let relative = Vector3D::new(
            point.x() - center.x(),
            point.y() - center.y(),
            point.z() - center.z(),
        );

        // 軸方向への射影（高さ）
        let axis = self.axis_internal();
        let height_along_axis = relative.dot(&axis.as_vector());

        // 高さが範囲外の場合
        let height = self.height_internal();
        if height_along_axis < T::ZERO || height_along_axis > height {
            return false;
        }

        // その高さでの円錐半径を計算（線形補間）
        let height_ratio = height_along_axis / height;
        let radius = self.radius_internal();
        let radius_at_height = radius * (T::ONE - height_ratio);

        // 軸からの距離を計算
        let axis_projection: Vector3D<T> = axis.as_vector() * height_along_axis;
        let radial_vector = Vector3D::new(
            relative.x() - axis_projection.x(),
            relative.y() - axis_projection.y(),
            relative.z() - axis_projection.z(),
        );
        let distance_from_axis = radial_vector.length();

        distance_from_axis <= radius_at_height
    }

    /// 円錐ソリッドの領域に対する点の位置を分類する
    ///
    /// 表面（側面・底面）までの距離が `tolerance` 以内なら `OnBoundary`、それ以外は内部・外部に分ける。
    /// `contains_point` は `classify_point(point, 0)` が `Outside` でないことと一致する。
    ///
    /// 表面までの距離は、点をプロファイル面上の (軸からの距離, 底面からの高さ) に写し、
    /// プロファイルの底辺と母線までの距離の小さい方として求める（軸上の辺は表面ではない）。
    pub fn classify_point(&self, point: Point3D<T>, tolerance: T) -> PointClassification {
        let (radial, axial) = self.profile_coordinates(point);
        let radius = self.radius_internal();
        let height = self.height_internal();
        let boundary_distance =
            distance_to_segment_2d((radial, axial), (T::ZERO, T::ZERO), (radius, T::ZERO)).min(
                distance_to_segment_2d((radial, axial), (radius, T::ZERO), (T::ZERO, height)),
            );

        if boundary_distance <= tolerance {
            PointClassification::OnBoundary
        } else if self.contains_point(point) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }

    /// プロファイル面上の点の座標（軸からの距離, 底面からの高さ）
    fn profile_coordinates(&self, point: Point3D<T>) -> (T, T) {
        let relative = Vector3D::from_points(&self.center_internal(), &point);
        let axis = self.axis_internal().as_vector();
        let axial = relative.dot(&axis);
        let radial = (relative - axis * axial).length();
        (radial, axial)
    }

    /// 点から円錐ソリッドまでの最短距離
    ///
    /// # Arguments
    /// * `point` - 距離を測る点
    ///
    /// # Returns
    /// 円錐ソリッドまでの最短距離（内部・表面上の場合は0）
    ///
    /// # Algorithm
    /// 円錐ソリッドは、プロファイル面上の三角形（軸・底面の半径・母線で囲まれた領域）を
    /// 軸まわりに回転した形状のため、点をプロファイル面上の (軸からの距離, 底面からの高さ) に写し、
    /// 断面の底辺と母線までの距離の小さい方を求める。外部の点に最も近い断面上の点は、底辺か母線上にある。
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        if self.contains_point(point) {
            return T::ZERO;
        }

        let (radial, axial) = self.profile_coordinates(point);
        let radius = self.radius_internal();
        let height = self.height_internal();
        let to_base =
            distance_to_segment_2d((radial, axial), (T::ZERO, T::ZERO), (radius, T::ZERO));
        let to_slant =
            distance_to_segment_2d((radial, axial), (radius, T::ZERO), (T::ZERO, height));
        to_base.min(to_slant)
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
        let r = self.radius_internal();
        let center = self.center_internal();
        points.push(Point3D::new(center.x() + r, center.y(), center.z()));
        points.push(Point3D::new(center.x(), center.y() + r, center.z()));
        points.push(Point3D::new(center.x() - r, center.y(), center.z()));
        points.push(Point3D::new(center.x(), center.y() - r, center.z()));

        // 頂点も追加
        let apex = self.apex_internal();
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
        self.center_internal()
    }

    /// 指定した高さでの円錐半径を計算
    ///
    /// # Arguments
    /// * `height_from_base` - 底面からの高さ
    ///
    /// # Returns
    /// その高さでの半径（範囲外の場合は None）
    pub fn radius_at_height(&self, height_from_base: T) -> Option<T> {
        let height = self.height_internal();
        if height_from_base < T::ZERO || height_from_base > height {
            return None;
        }

        let height_ratio = height_from_base / height;
        let radius = self.radius_internal() * (T::ONE - height_ratio);
        Some(radius)
    }

    /// 円錐の軸線を取得
    ///
    /// # Returns
    /// 底面中心から頂点への直線
    pub fn axis_line(&self) -> (Point3D<T>, Point3D<T>) {
        let start = self.center_internal();
        let end = self.apex_internal();
        (start, end)
    }
}

/// 2D の点から線分までの最短距離
fn distance_to_segment_2d<T: Scalar>(point: (T, T), start: (T, T), end: (T, T)) -> T {
    let edge = (end.0 - start.0, end.1 - start.1);
    let to_point = (point.0 - start.0, point.1 - start.1);
    let edge_length_squared = edge.0 * edge.0 + edge.1 * edge.1;
    let t = ((to_point.0 * edge.0 + to_point.1 * edge.1) / edge_length_squared)
        .max(T::ZERO)
        .min(T::ONE);
    let dx = to_point.0 - edge.0 * t;
    let dy = to_point.1 - edge.1 * t;
    (dx * dx + dy * dy).sqrt()
}
