//! 円（Circle）の新実装
//!
//! foundation.rs の基盤トレイトに基づく Circle3D の実装

use crate::{BBox3D, Point3D, Vector3D};
use geo_foundation::{abstract_types::geometry::core_foundation::*, Scalar};

/// 3次元空間の円
///
/// 3次元空間内の任意の平面上に存在する円を表現
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle3D<T: Scalar> {
    center: Point3D<T>,
    normal: Vector3D<T>, // 円が存在する平面の法線ベクトル
    radius: T,
}

impl<T: Scalar> Circle3D<T> {
    /// 新しい円を作成
    ///
    /// # 引数
    /// * `center` - 円の中心点
    /// * `normal` - 円が存在する平面の法線ベクトル（正規化される）
    /// * `radius` - 円の半径（正の値である必要がある）
    ///
    /// # 戻り値
    /// * `Some(Circle3D)` - 有効な円が作成できた場合
    /// * `None` - 半径が0以下、または法線ベクトルがゼロベクトルの場合
    pub fn new(center: Point3D<T>, normal: Vector3D<T>, radius: T) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        let normalized_normal = normal.normalize()?;

        Some(Self {
            center,
            normal: normalized_normal,
            radius,
        })
    }

    /// XY平面上の円を作成（Z軸が法線）
    pub fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(center, Vector3D::unit_z(), radius)
    }

    /// XZ平面上の円を作成（Y軸が法線）
    pub fn new_xz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(center, Vector3D::unit_y(), radius)
    }

    /// YZ平面上の円を作成（X軸が法線）
    pub fn new_yz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Self::new(center, Vector3D::unit_x(), radius)
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> Vector3D<T> {
        self.normal
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 円平面のU軸（基準軸）を取得
    pub fn u_axis(&self) -> Vector3D<T> {
        let (u, _) = self.get_plane_basis();
        u
    }

    /// 円平面のV軸を取得
    pub fn v_axis(&self) -> Vector3D<T> {
        let (_, v) = self.get_plane_basis();
        v
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

    /// 円上の点を角度から取得
    ///
    /// # 引数
    /// * `angle` - 角度（ラジアン）
    ///
    /// # 戻り値
    /// 円上の点（円の平面内での極座標から3D座標に変換）
    pub fn point_at_angle(&self, angle: T) -> Point3D<T> {
        // 円の平面内での基準ベクトルを計算
        // 法線ベクトルに垂直な2つのベクトルを求める
        let (u, v) = self.get_plane_basis();

        let x = angle.cos();
        let y = angle.sin();

        // 円上の点 = 中心 + radius * (x * u + y * v)
        let offset = Vector3D::new(
            self.radius * (x * u.x() + y * v.x()),
            self.radius * (x * u.y() + y * v.y()),
            self.radius * (x * u.z() + y * v.z()),
        );

        Point3D::new(
            self.center.x() + offset.x(),
            self.center.y() + offset.y(),
            self.center.z() + offset.z(),
        )
    }

    /// 円の平面における基準ベクトル（u, v）を取得
    /// 法線ベクトルに垂直な正規直交基底
    fn get_plane_basis(&self) -> (Vector3D<T>, Vector3D<T>) {
        // Z軸方向の法線の場合は特別扱い（XY平面）
        if (self.normal.z() - T::ONE).abs() < T::TOLERANCE {
            // XY平面：X軸とY軸を使用
            return (Vector3D::unit_x(), Vector3D::unit_y());
        }

        // Y軸方向の法線の場合（XZ平面）
        if (self.normal.y() - T::ONE).abs() < T::TOLERANCE {
            return (Vector3D::unit_x(), Vector3D::unit_z());
        }

        // X軸方向の法線の場合（YZ平面）
        if (self.normal.x() - T::ONE).abs() < T::TOLERANCE {
            return (Vector3D::unit_y(), Vector3D::unit_z());
        }

        // 一般的な場合：Gram-Schmidt 過程で正規直交基底を作成
        let temp = if self.normal.z().abs() < T::TOLERANCE {
            Vector3D::unit_z()
        } else {
            Vector3D::unit_x()
        };

        // 第一基底ベクトル: normal × temp を正規化
        let first = self
            .normal
            .cross(&temp)
            .normalize()
            .unwrap_or_else(|| Vector3D::unit_x());

        // 第二基底ベクトル: normal × first
        let second = self.normal.cross(&first);

        (first, second)
    }

    /// 点が円の平面上にあるかを判定
    pub fn point_on_plane(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let center_to_point = Vector3D::from_points(&self.center, point);
        let distance_to_plane = center_to_point.dot(&self.normal).abs();
        distance_to_plane <= tolerance
    }

    /// 点から円の中心への距離（3D空間内）
    pub fn distance_to_center(&self, point: &Point3D<T>) -> T {
        self.center.distance_to(point)
    }

    /// 点から円への最短距離
    pub fn distance_to_circle(&self, point: &Point3D<T>) -> T {
        // 点を円の平面に投影
        let center_to_point = Vector3D::from_points(&self.center, point);
        let plane_distance = center_to_point.dot(&self.normal);

        // 平面上での投影点
        let projected_offset = Vector3D::new(
            center_to_point.x() - plane_distance * self.normal.x(),
            center_to_point.y() - plane_distance * self.normal.y(),
            center_to_point.z() - plane_distance * self.normal.z(),
        );

        let radial_distance = projected_offset.length();
        let circle_distance = (radial_distance - self.radius).abs();

        // 3D距離 = √(平面距離² + 円距離²)
        (plane_distance * plane_distance + circle_distance * circle_distance).sqrt()
    }
}

// === foundation トレイト実装 ===

impl<T: Scalar> CoreFoundation<T> for Circle3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;
    type BBox = BBox3D<T>;

    /// 円の境界ボックス
    fn bounding_box(&self) -> Self::BBox {
        // 円の平面での基準ベクトルを取得
        let (u, v) = self.get_plane_basis();

        // 円上の極値点（±radius * u, ±radius * v）を計算
        let extents = [
            (
                self.radius * u.x(),
                self.radius * u.y(),
                self.radius * u.z(),
            ),
            (
                -self.radius * u.x(),
                -self.radius * u.y(),
                -self.radius * u.z(),
            ),
            (
                self.radius * v.x(),
                self.radius * v.y(),
                self.radius * v.z(),
            ),
            (
                -self.radius * v.x(),
                -self.radius * v.y(),
                -self.radius * v.z(),
            ),
        ];

        let mut min_x = self.center.x();
        let mut max_x = self.center.x();
        let mut min_y = self.center.y();
        let mut max_y = self.center.y();
        let mut min_z = self.center.z();
        let mut max_z = self.center.z();

        for (dx, dy, dz) in extents.iter() {
            let x = self.center.x() + *dx;
            let y = self.center.y() + *dy;
            let z = self.center.z() + *dz;

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            min_z = min_z.min(z);
            max_z = max_z.max(z);
        }

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }
}

impl<T: Scalar> BasicMetrics<T> for Circle3D<T> {
    /// 円の周長
    fn perimeter(&self) -> Option<T> {
        Some(self.circumference())
    }

    /// 円の面積
    fn area(&self) -> Option<T> {
        Some(self.area())
    }
}

impl<T: Scalar> BasicContainment<T> for Circle3D<T> {
    /// 点が円上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.on_boundary(point, T::TOLERANCE)
    }

    /// 点が円の境界上（円周上）にあるかを判定
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        // 点が円の平面上にある必要がある
        if !self.point_on_plane(point, tolerance) {
            return false;
        }

        // 中心からの距離が半径と一致する必要がある
        let center_to_point = Vector3D::from_points(&self.center, point);
        let plane_projected_distance = {
            let plane_distance = center_to_point.dot(&self.normal);
            let projected = Vector3D::new(
                center_to_point.x() - plane_distance * self.normal.x(),
                center_to_point.y() - plane_distance * self.normal.y(),
                center_to_point.z() - plane_distance * self.normal.z(),
            );
            projected.length()
        };

        (plane_projected_distance - self.radius).abs() <= tolerance
    }

    /// 点から円への最短距離
    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to_circle(point)
    }
}

impl<T: Scalar> BasicParametric<T> for Circle3D<T> {
    /// パラメータ範囲（0 から 2π）
    fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU)
    }

    /// パラメータから円上の点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point {
        self.point_at_angle(t)
    }

    /// パラメータでの接線ベクトル
    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        let (u, v) = self.get_plane_basis();

        // 円の接線 = d/dt[cos(t)*u + sin(t)*v] = -sin(t)*u + cos(t)*v
        let cos_t = t.cos();
        let sin_t = t.sin();

        Vector3D::new(
            self.radius * (-sin_t * u.x() + cos_t * v.x()),
            self.radius * (-sin_t * u.y() + cos_t * v.y()),
            self.radius * (-sin_t * u.z() + cos_t * v.z()),
        )
    }
}
