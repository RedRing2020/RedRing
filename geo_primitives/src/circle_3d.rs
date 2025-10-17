//! 円（Circle）の新実装
//!
//! foundation.rs の基盤トレイトに基づく Circle3D の実装

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元空間の円
///
/// 3次元空間内の任意の平面上に存在する円を表現
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// 円平面のU軸（基準軸）を取得
    pub fn u_axis(&self) -> Direction3D<T> {
        let (u, _) = self.get_plane_basis();
        Direction3D::from_vector(u).unwrap()
    }

    /// 円平面のV軸を取得
    pub fn v_axis(&self) -> Direction3D<T> {
        let (_, v) = self.get_plane_basis();
        Direction3D::from_vector(v).unwrap()
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
        let first_unnormalized = self.normal.as_vector().cross(&temp);
        let first = first_unnormalized.normalize();

        // 第二基底ベクトル: normal × first
        let second = self.normal.as_vector().cross(&first);

        (first, second)
    }

    /// 点が円の平面上にあるかを判定
    pub fn point_on_plane(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let center_to_point = Vector3D::from_points(&self.center, point);
        let distance_to_plane = center_to_point.dot(&self.normal.as_vector()).abs();
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
        let plane_distance = center_to_point.dot(&self.normal.as_vector());

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

// === 新 traits システム実装（一時無効化） ===
// TODO: 古いFoundationトレイトを新しいtraitsシステムに移行完了後に再実装

/*
// 以下のFoundationトレイト実装は新しいtraitsシステムへの移行完了後に再実装
impl<T: Scalar> CoreFoundation<T> for Circle3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;
    type BBox = BBox3D<T>;

    fn bounding_box(&self) -> Self::BBox {
        // TODO: 新しいtraitsシステムでの実装
        unimplemented!()
    }
}

impl<T: Scalar> BasicIntersection<T, Point3D<T>> for Circle3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // TODO: 新しいtraitsシステムでの実装
        unimplemented!()
    }
}
*/
