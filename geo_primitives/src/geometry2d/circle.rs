//! 2D Circle implementation
//!
//! 2次元平面における円の具体的な実装

use crate::geometry2d::{bbox::BBoxF64, Point2D, Vector};
use geo_foundation::GEOMETRIC_TOLERANCE;
use geo_foundation::{abstract_types::geometry::BBox as BBoxTrait, Scalar};

/// 2D平面上の円を表現する構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Circle2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
}

// 後方互換性のための型エイリアス
pub type Circle<T> = Circle2D<T>;

// 型特化版エイリアス
pub type Circle2DF64 = Circle2D<f64>;
pub type Circle2DF32 = Circle2D<f32>;

impl<T: Scalar> Circle2D<T> {
    /// 新しい円を作成
    ///
    /// # Arguments
    /// * `center` - 円の中心点
    /// * `radius` - 円の半径（正の値）
    ///
    /// # Panics
    /// 半径が負の値またはNaNの場合にパニックする
    pub fn new(center: Point2D<T>, radius: T) -> Self {
        assert!(
            radius >= T::ZERO && radius.to_f64().is_finite(),
            "半径は非負の有限値である必要があります"
        );
        Self { center, radius }
    }

    /// 原点を中心とする円を作成
    ///
    /// # Arguments
    /// * `radius` - 円の半径（正の値）
    pub fn origin_circle(radius: T) -> Self {
        Self::new(Point2D::new(T::ZERO, T::ZERO), radius)
    }

    /// 単位円（半径1、原点中心）を作成
    pub fn unit_circle() -> Self {
        Self::new(Point2D::new(T::ZERO, T::ZERO), T::ONE)
    }

    /// 3点を通る円を作成
    ///
    /// # Arguments
    /// * `p1`, `p2`, `p3` - 円周上の3点
    ///
    /// # Returns
    /// 3点を通る円、または3点が一直線上にある場合は`None`
    pub fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self> {
        // 3点が一直線上にないかチェック
        let v1 = Vector::new(p2.x() - p1.x(), p2.y() - p1.y());
        let v2 = Vector::new(p3.x() - p1.x(), p3.y() - p1.y());

        let cross = v1.cross_2d(&v2);
        if cross.abs() < T::from_f64(GEOMETRIC_TOLERANCE) {
            return None; // 3点が一直線上にある
        }

        // 外心を計算
        let d = T::from_f64(2.0) * cross;
        let ux = (v2.y() * v1.length_squared() - v1.y() * v2.length_squared()) / d;
        let uy = (v1.x() * v2.length_squared() - v2.x() * v1.length_squared()) / d;

        let center = Point2D::new(p1.x() + ux, p1.y() + uy);
        let radius = center.distance_to(&p1);

        Some(Self::new(center, radius))
    }

    /// 円が退化しているか（半径が0）を判定
    pub fn is_degenerate(&self) -> bool {
        self.radius < T::from_f64(GEOMETRIC_TOLERANCE)
    }

    /// 指定された点までの最短距離を取得
    /// 円周までの距離（内部の点では負の値）
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let center_distance = self.center.distance_to(point);
        center_distance - self.radius
    }

    /// 円を指定倍率で拡大縮小
    pub fn scale(&self, factor: T) -> Self {
        assert!(
            factor >= T::ZERO && factor.to_f64().is_finite(),
            "拡大縮小係数は非負の有限値である必要があります"
        );
        Self::new(self.center, self.radius * factor)
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        let new_center = Point2D::new(self.center.x() + vector.x(), self.center.y() + vector.y());
        Self::new(new_center, self.radius)
    }

    /// 境界上の点を含む点の包含判定（許容誤差付き）
    pub fn contains_point_on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point(point).abs();
        distance <= tolerance
    }

    /// 他の円との交差判定
    pub fn intersects_with_circle(&self, other: &Circle2D<T>) -> bool {
        let distance = self.center.distance_to(&other.center);
        let sum_radii = self.radius + other.radius;
        let diff_radii = (self.radius - other.radius).abs();

        distance <= sum_radii && distance >= diff_radii
    }

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 円の面積を計算
    pub fn area(&self) -> T {
        // π * r²
        let pi = T::from_f64(std::f64::consts::PI);
        pi * self.radius * self.radius
    }

    /// 円の周長（円周）を計算
    pub fn circumference(&self) -> T {
        // 2π * r
        let tau = T::from_f64(2.0 * std::f64::consts::PI);
        tau * self.radius
    }

    /// 指定された点が円の内部にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        let distance_squared = (point.x() - self.center.x()) * (point.x() - self.center.x())
            + (point.y() - self.center.y()) * (point.y() - self.center.y());
        distance_squared <= self.radius * self.radius
    }

    /// 指定された点が円周上にあるかを判定（許容誤差内）
    pub fn on_circumference(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance = self.center.distance_to(point);
        (distance - self.radius).abs() <= tolerance
    }

    /// 指定された角度（ラジアン）の位置にある点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        Point2D::new(
            self.center.x() + self.radius * cos_angle,
            self.center.y() + self.radius * sin_angle,
        )
    }

    /// 円の境界ボックス（最小と最大の座標値）を取得
    pub fn bounding_box(&self) -> (Point2D<T>, Point2D<T>) {
        let min_point = Point2D::new(self.center.x() - self.radius, self.center.y() - self.radius);
        let max_point = Point2D::new(self.center.x() + self.radius, self.center.y() + self.radius);
        (min_point, max_point)
    }

    /// 指定された点での接線ベクトルを取得
    pub fn tangent_at_point(&self, point: &Point2D<T>) -> Option<Vector<T>> {
        if !self.on_circumference(point, T::from_f64(GEOMETRIC_TOLERANCE)) {
            return None;
        }

        // 中心から点への方向ベクトル（半径ベクトル）
        let radial = Vector::new(point.x() - self.center.x(), point.y() - self.center.y());

        // 接線ベクトルは半径ベクトルに垂直
        // 2Dでは (x, y) → (-y, x) で90度回転
        Some(Vector::new(-radial.y(), radial.x()))
    }

    /// 指定された角度での接線ベクトルを取得
    pub fn tangent_at_angle(&self, angle: T) -> Vector<T> {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 接線ベクトルは (-sin, cos) 方向に半径分の長さ
        Vector::new(-self.radius * sin_angle, self.radius * cos_angle)
    }
}

impl From<Circle2D<f64>> for BBoxF64 {
    fn from(_circle: Circle2D<f64>) -> Self {
        let (min_point, max_point) = _circle.bounding_box();
        BBoxTrait::new(min_point, max_point)
    }
}

// f64特化版の機能拡張
impl Circle2D<f64> {
    /// 3D円に変換（Z=0のXY平面上）
    pub fn to_3d(&self) -> crate::geometry3d::Circle<f64> {
        use crate::geometry3d::Point3D;
        let center_3d = Point3D::new(self.center.x(), self.center.y(), 0.0);
        crate::geometry3d::Circle::xy_plane_circle(center_3d, self.radius)
    }

    /// 2つの円の交点を計算
    pub fn intersection_points(&self, other: &Circle2D<f64>) -> Vec<Point2D<f64>> {
        let distance = self.center.distance_to(&other.center);

        // 円が交差しない場合
        if distance > self.radius + other.radius || distance < (self.radius - other.radius).abs() {
            return Vec::new();
        }

        // 同心円の場合
        if distance < GEOMETRIC_TOLERANCE {
            return Vec::new(); // 無限個または0個の交点
        }

        // 交点を計算
        let a = (self.radius * self.radius - other.radius * other.radius + distance * distance)
            / (2.0 * distance);
        let h = (self.radius * self.radius - a * a).sqrt();

        let dx = other.center.x() - self.center.x();
        let dy = other.center.y() - self.center.y();

        let p2_x = self.center.x() + a * dx / distance;
        let p2_y = self.center.y() + a * dy / distance;

        if h.abs() < GEOMETRIC_TOLERANCE {
            // 1点で接する
            vec![Point2D::new(p2_x, p2_y)]
        } else {
            // 2点で交差
            let offset_x = h * dy / distance;
            let offset_y = h * dx / distance;

            vec![
                Point2D::new(p2_x + offset_x, p2_y - offset_y),
                Point2D::new(p2_x - offset_x, p2_y + offset_y),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    // テストは unit_tests/ ディレクトリに分離済み
}
