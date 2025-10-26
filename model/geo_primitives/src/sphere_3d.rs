//! Sphere3D - Core Implementation
//!
//! 3次元球の基本実装とコンストラクタ、アクセサメソッド

use crate::{BBox3D, Point3D, Vector3D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

/// 3次元空間の球
///
/// 中心点と半径で定義される球体
#[derive(Debug, Clone, PartialEq)]
pub struct Sphere3D<T: Scalar> {
    center: Point3D<T>,
    radius: T,
}

impl<T: Scalar> Sphere3D<T> {
    /// 新しい球を作成
    ///
    /// # 引数
    /// * `center` - 球の中心点
    /// * `radius` - 球の半径（正の値である必要がある）
    ///
    /// # 戻り値
    /// * `Some(Sphere3D)` - 有効な球が作成できた場合
    /// * `None` - 半径が0以下の場合
    pub fn new(center: Point3D<T>, radius: T) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        Some(Self { center, radius })
    }

    /// 原点中心の球を作成
    pub fn new_at_origin(radius: T) -> Option<Self> {
        Self::new(Point3D::new(T::ZERO, T::ZERO, T::ZERO), radius)
    }

    /// 直径から球を作成
    pub fn from_diameter(center: Point3D<T>, diameter: T) -> Option<Self> {
        if diameter <= T::ZERO {
            return None;
        }
        Self::new(center, diameter / T::from_f64(2.0))
    }

    // ========================================================================
    // Core Access Methods
    // ========================================================================

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 直径を取得
    pub fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

    /// 球の表面積を計算
    /// S = 4πr²
    pub fn surface_area(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius
    }

    /// 球の体積を計算
    /// V = (4/3)πr³
    pub fn volume(&self) -> T {
        T::from_f64(4.0) * T::PI * self.radius * self.radius * self.radius / T::from_f64(3.0)
    }

    /// 球が退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        self.radius <= DefaultTolerances::distance::<T>()
    }

    // ========================================================================
    // Geometric Queries
    // ========================================================================

    /// 点が球の内部にあるかどうかを判定
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
        let distance_squared = self.center.distance_squared_to(&point);
        distance_squared <= self.radius * self.radius
    }

    /// 点が球の表面にあるかどうかを判定（許容誤差考慮）
    pub fn point_on_surface(&self, point: Point3D<T>) -> bool {
        let distance = self.center.distance_to(&point);
        (distance - self.radius).abs() <= DefaultTolerances::distance::<T>()
    }

    /// 点から球表面への最短距離を計算
    /// 球内部の点の場合は負の値を返す
    pub fn distance_to_surface(&self, point: Point3D<T>) -> T {
        let distance_to_center = self.center.distance_to(&point);
        distance_to_center - self.radius
    }

    /// 球の境界ボックスを取得
    pub fn bounding_box(&self) -> BBox3D<T> {
        let min_point = Point3D::new(
            self.center.x() - self.radius,
            self.center.y() - self.radius,
            self.center.z() - self.radius,
        );
        let max_point = Point3D::new(
            self.center.x() + self.radius,
            self.center.y() + self.radius,
            self.center.z() + self.radius,
        );
        BBox3D::new(min_point, max_point)
    }

    // ========================================================================
    // Surface Operations
    // ========================================================================

    /// 指定された方向ベクトルで球表面上の点を取得
    /// 方向ベクトルは正規化される
    pub fn point_on_surface_in_direction(&self, direction: Vector3D<T>) -> Option<Point3D<T>> {
        let normalized = direction.normalize();
        if normalized == Vector3D::new(T::ZERO, T::ZERO, T::ZERO) {
            return None;
        }

        Some(Point3D::new(
            self.center.x() + normalized.x() * self.radius,
            self.center.y() + normalized.y() * self.radius,
            self.center.z() + normalized.z() * self.radius,
        ))
    }

    /// 球の中心から指定された点への方向の表面点を取得
    pub fn point_on_surface_towards(&self, target: Point3D<T>) -> Option<Point3D<T>> {
        let direction = Vector3D::new(
            target.x() - self.center.x(),
            target.y() - self.center.y(),
            target.z() - self.center.z(),
        );
        self.point_on_surface_in_direction(direction)
    }

    // ========================================================================
    // Geometric Transformations
    // ========================================================================

    /// 球を平行移動
    pub fn translate(&self, translation: Vector3D<T>) -> Self {
        Self {
            center: Point3D::new(
                self.center.x() + translation.x(),
                self.center.y() + translation.y(),
                self.center.z() + translation.z(),
            ),
            radius: self.radius,
        }
    }

    /// 球を均等スケーリング
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        Some(Self {
            center: self.center,
            radius: self.radius * factor,
        })
    }

    /// 球を中心点を基準にスケーリング
    pub fn scale_about_center(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        Some(Self {
            center: self.center,
            radius: self.radius * factor,
        })
    }

    /// 球を指定点を基準にスケーリング
    pub fn scale_about_point(&self, scale_center: Point3D<T>, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        // 中心点をスケーリング
        let center_to_scale_center = Vector3D::new(
            self.center.x() - scale_center.x(),
            self.center.y() - scale_center.y(),
            self.center.z() - scale_center.z(),
        );

        let new_center = Point3D::new(
            scale_center.x() + center_to_scale_center.x() * factor,
            scale_center.y() + center_to_scale_center.y() * factor,
            scale_center.z() + center_to_scale_center.z() * factor,
        );

        Some(Self {
            center: new_center,
            radius: self.radius * factor,
        })
    }
}

impl<T: Scalar> std::fmt::Display for Sphere3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sphere3D(center: {:?}, radius: {})",
            self.center, self.radius
        )
    }
}
