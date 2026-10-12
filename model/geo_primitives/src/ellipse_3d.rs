//! 3D楕円のCore実装
//!
//! Foundation統一システムに基づくEllipse3Dの必須機能のみ

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Angle, Direction3D, Point3D, Vector3D};
use geo_contracts::{default_angle_tolerance, default_distance_tolerance};
use geo_contracts::{
    Ellipse3DConstructor, Ellipse3DContainment, Ellipse3DDistance, Ellipse3DEvaluation,
    Ellipse3DProjection, Ellipse3DProperties, PointClassification, PrimitiveKind,
    PrimitiveMetadata, Scalar,
};

/// 3次元楕円（Core実装）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse3D<T: Scalar> {
    center: Point3D<T>,
    semi_major_axis: T,
    semi_minor_axis: T,
    normal: Direction3D<T>,         // 楕円平面の法線ベクトル（正規化済み）
    major_axis_dir: Direction3D<T>, // 長軸方向ベクトル（正規化済み）
}

impl<T: Scalar> PrimitiveMetadata for Ellipse3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipse
    }
}

impl<T: Scalar> Ellipse3D<T> {
    /// 新しい3D楕円を作成
    ///
    /// 基本的な検証のみ実行
    pub fn new(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        normal: Vector3D<T>,
        major_axis_dir: Vector3D<T>,
    ) -> Option<Self> {
        // 半軸の長さの検証
        if semi_major_axis < semi_minor_axis || semi_minor_axis <= T::ZERO {
            return None;
        }

        // Direction3Dに変換（自動的に正規化される）
        let normal_dir = Direction3D::from_vector(normal)?;
        let major_axis_dir = Direction3D::from_vector(major_axis_dir)?;

        // 基本的な直交性チェック
        let dot_product = normal_dir.dot(&major_axis_dir);
        if dot_product.abs() > default_angle_tolerance::<T>() {
            return None;
        }

        Some(Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            normal: normal_dir,
            major_axis_dir,
        })
    }

    /// 楕円の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 長半軸の長さを取得
    pub fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    pub fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }

    /// 楕円平面の法線ベクトルを取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// 長軸方向ベクトルを取得
    pub fn major_axis_direction(&self) -> Direction3D<T> {
        self.major_axis_dir
    }

    /// 短軸方向ベクトルを取得
    pub fn minor_axis_direction(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.normal.cross(&self.major_axis_dir))
            .expect("Cross product of normalized vectors should be valid")
    }

    /// 楕円の面積を計算
    pub fn area(&self) -> T {
        T::PI * self.semi_major_axis * self.semi_minor_axis
    }

    /// 楕円の周回長を近似計算（ラマヌジャンの公式）
    pub fn circumference(&self) -> T {
        let a = self.semi_major_axis;
        let b = self.semi_minor_axis;
        let h = ((a - b) / (a + b)).powi(2);

        T::PI
            * (a + b)
            * (T::ONE
                + (T::from_f64(3.0) * h)
                    / (T::from_f64(10.0) + (T::from_f64(4.0) - T::from_f64(3.0) * h).sqrt()))
    }

    /// Primitive 局所座標系の angle parameter `t` (`0 <= t <= 2π`) で楕円上の点を計算
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir.as_vector() * (self.semi_major_axis * cos_t);
        let minor_component =
            self.minor_axis_direction().as_vector() * (self.semi_minor_axis * sin_t);

        Point3D::new(
            self.center.x() + major_component.x() + minor_component.x(),
            self.center.y() + major_component.y() + minor_component.y(),
            self.center.z() + major_component.z() + minor_component.z(),
        )
    }

    /// `point_at_parameter` と同じ local angle domain における接線ベクトルを計算
    pub fn tangent_at_parameter(&self, t: T) -> Vector3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir.as_vector() * (-self.semi_major_axis * sin_t);
        let minor_component =
            self.minor_axis_direction().as_vector() * (self.semi_minor_axis * cos_t);

        major_component + minor_component
    }

    /// 角度 θ での楕円上の点を計算
    pub fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        self.point_at_parameter(angle.to_radians())
    }

    /// 3D空間での点から楕円（曲線）への最短距離を計算（内部実装）
    fn distance_to_point_3d_internal(&self, point: (T, T, T)) -> T {
        let (x_local, y_local, z_local) =
            self.local_coordinates(&Point3D::new(point.0, point.1, point.2));
        geo_commons::ellipse_3d_distance_to_point(
            x_local,
            y_local,
            z_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        )
    }

    /// 点が楕円上にあるか判定（楕円までの距離が `tolerance` 以内）
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point_3d_internal((point.x(), point.y(), point.z())) <= tolerance
    }

    /// 楕円が平面上に囲む領域に対する点の位置を分類する
    ///
    /// 楕円までの距離が `tolerance` 以内なら `OnBoundary` とする。それ以外は、平面からの距離が
    /// `tolerance` 以内で、平面へ投影した点が楕円の内側にあれば `Inside`、それ以外を `Outside` とする。
    pub fn classify_point(&self, point: &Point3D<T>, tolerance: T) -> PointClassification {
        if self.contains_point(point, tolerance) {
            return PointClassification::OnBoundary;
        }

        let (x_local, y_local, z_local) = self.local_coordinates(point);
        if z_local.abs() > tolerance {
            return PointClassification::Outside;
        }

        let x_normalized = x_local / self.semi_major_axis;
        let y_normalized = y_local / self.semi_minor_axis;
        if x_normalized * x_normalized + y_normalized * y_normalized < T::ONE {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }

    /// 点に最も近い楕円上の点を取得
    pub fn closest_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let (x_local, y_local, _) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        );

        let u = self.major_axis_dir.as_vector();
        let v = self.minor_axis_direction().as_vector();
        Point3D::new(
            self.center.x() + u.x() * closest_x + v.x() * closest_y,
            self.center.y() + u.y() * closest_x + v.y() * closest_y,
            self.center.z() + u.z() * closest_x + v.z() * closest_y,
        )
    }

    /// 点に最も近い楕円上の点の local angle parameter（`0 <= t < 2π`）を取得
    ///
    /// `point_at_parameter(t)` が点の最近点となる `t` を返す。平面外の点は平面へ投影した点の最近点を使う。
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let (x_local, y_local, _) = self.local_coordinates(point);
        let (closest_x, closest_y) = geo_commons::ellipse_2d_closest_point(
            x_local,
            y_local,
            self.semi_major_axis,
            self.semi_minor_axis,
        );
        let angle = (closest_y / self.semi_minor_axis).atan2(closest_x / self.semi_major_axis);
        if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        }
    }

    /// 中心を原点とし、長軸・短軸・法線を軸とする楕円の局所座標系での点の座標
    fn local_coordinates(&self, point: &Point3D<T>) -> (T, T, T) {
        let translated = Vector3D::from_points(&self.center, point);
        (
            translated.dot(&self.major_axis_dir.as_vector()),
            translated.dot(&self.minor_axis_direction().as_vector()),
            translated.dot(&self.normal.as_vector()),
        )
    }
}

impl<T: Scalar> Ellipse3DConstructor<T> for Ellipse3D<T> {
    /// 基本コンストラクタ（中心点、平面法線、長軸半径、短軸半径、長軸方向）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        major_axis_direction: (T, T, T),
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let major_dir_vec = Vector3D::new(
            major_axis_direction.0,
            major_axis_direction.1,
            major_axis_direction.2,
        );

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            normal_vec,
            major_dir_vec,
        )
    }

    /// XY平面上の楕円作成
    fn new_xy_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();
        let major_dir = Vector3D::new(cos_rot, sin_rot, T::ZERO);

        Self::new(
            center_point,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_z(),
            major_dir,
        )
    }
}

impl<T: Scalar> Ellipse3DProperties<T> for Ellipse3D<T> {
    /// 楕円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> (T, T, T) {
        (self.normal.x(), self.normal.y(), self.normal.z())
    }

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> (T, T, T) {
        (
            self.major_axis_dir.x(),
            self.major_axis_dir.y(),
            self.major_axis_dir.z(),
        )
    }

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> (T, T, T) {
        let minor_axis = self.minor_axis_direction();
        (minor_axis.x(), minor_axis.y(), minor_axis.z())
    }

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }
}

impl<T: Scalar> Ellipse3DEvaluation<T> for Ellipse3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let p = Ellipse3D::point_at_parameter(self, t);
        (p.x(), p.y(), p.z())
    }
}

impl<T: Scalar> Ellipse3DContainment<T> for Ellipse3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        self.distance_to_point_3d_internal(point) <= default_distance_tolerance::<T>()
    }

    fn classify_point(&self, point: (T, T, T)) -> PointClassification {
        let p = Point3D::new(point.0, point.1, point.2);
        Ellipse3D::classify_point(self, &p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> Ellipse3DDistance<T> for Ellipse3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        self.distance_to_point_3d_internal(point)
    }
}

impl<T: Scalar> Ellipse3DProjection<T> for Ellipse3D<T> {
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = Point3D::new(point.0, point.1, point.2);
        let closest = Ellipse3D::closest_point(self, &p);
        (closest.x(), closest.y(), closest.z())
    }
}
