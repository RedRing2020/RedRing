/// 汎用Circleトレイトと構造体実装
///
/// f32/f64両対応のジェネリック設計によるCircle実装。
/// 2D/3D環境での円操作を統一的に提供します。
///
/// # 設計方針
///
/// - **ジェネリック**: f32/f64両対応でゲーム・CAD用途に最適化
/// - **型安全**: Angleを使用した角度の型安全性保証
/// - **拡張性**: 2D/3Dの専門化トレイトで機能拡張
/// - **性能**: インライン化による最適化
use crate::abstract_types::Scalar;
use crate::geometry::{Angle, Point2D, Point3D, Vector2D, Vector3D, BoundingBox2D};
use std::fmt::{Debug, Display};

/// 汎用Circleトレイト
///
/// 次元に依存しない円の基本操作を定義します。
/// 2D/3Dの専門化はCircle2D/Circle3Dトレイトで行います。
pub trait Circle<T: Scalar>: Debug + Clone + PartialEq {
    /// 中心点の型（Point2D, Point3Dなど）
    type Point: Debug + Clone + PartialEq;

    /// ベクトル型（Vector2D, Vector3Dなど）
    type Vector: Debug + Clone + PartialEq;

    /// 円の中心を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> T;

    /// 面積を計算
    fn area(&self) -> T {
        T::PI * self.radius() * self.radius()
    }

    /// 円周を計算
    fn circumference(&self) -> T {
        T::TAU * self.radius()
    }

    /// 直径を計算
    fn diameter(&self) -> T {
        self.radius() * (T::ONE + T::ONE)
    }

    /// 点が円の内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が円周上にあるかを判定（許容誤差考慮）
    fn point_on_circumference(&self, point: &Self::Point) -> bool;

    /// 円周上の指定角度の点を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;

    /// 指定角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector;

    /// 指定角度での法線ベクトルを取得
    fn normal_at_angle(&self, angle: Angle<T>) -> Self::Vector;

    /// 円の妥当性を検証
    fn is_valid(&self) -> bool {
        self.radius() > T::ZERO
    }

    /// 単位円かどうかを判定
    fn is_unit_circle(&self) -> bool {
        self.radius().approx_eq(T::ONE)
    }
}

/// 2D円の専門化トレイト
pub trait Circle2D<T: Scalar>: Circle<T> {
    /// 2D点との距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;

    /// 円を拡大・縮小
    fn scaled(&self, factor: T) -> Self;

    /// 円を平行移動
    fn translated(&self, offset: &Self::Vector) -> Self;

    /// 3点から円を作成
    fn from_three_points(p1: Self::Point, p2: Self::Point, p3: Self::Point) -> Option<Self>
    where
        Self: Sized;

    /// 最小外接円として作成
    fn bounding_circle(points: &[Self::Point]) -> Option<Self>
    where
        Self: Sized;
}

/// 3D円の専門化トレイト
pub trait Circle3D<T: Scalar>: Circle<T> {
    /// 法線ベクトル型
    type Normal: Debug + Clone + PartialEq;

    /// 円の法線を取得
    fn normal(&self) -> Self::Normal;

    /// 円を指定軸周りに回転
    fn rotated_around_axis(&self, axis: &Self::Vector, angle: Angle<T>) -> Self;

    /// 2D円に投影（具体型を返す）
    fn project_to_2d(&self) -> Circle2DImpl<T>;

    /// 3D空間での点との最短距離
    fn distance_to_point_3d(&self, point: &Self::Point) -> T;
}

/// 2D円の具体実装
#[derive(Debug, Clone, PartialEq)]
pub struct Circle2DImpl<T: Scalar> {
    /// 中心点
    center: Point2D<T>,
    /// 半径
    radius: T,
}

impl<T: Scalar> Circle2DImpl<T> {
    /// 新しい円を作成
    ///
    /// # Arguments
    ///
    /// * `center` - 中心点
    /// * `radius` - 半径
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_foundation::geometry::{Circle, Circle2DImpl, Point2D};
    ///
    /// let center = Point2D::<f64>::new(0.0, 0.0);
    /// let circle = Circle2DImpl::<f64>::new(center, 5.0);
    /// assert_eq!(circle.radius(), 5.0);
    /// ```
    pub fn new(center: Point2D<T>, radius: T) -> Self {
        Self { center, radius }
    }

    /// 原点中心の単位円を作成
    pub fn unit() -> Self {
        Self::new(Point2D::origin(), T::ONE)
    }

    /// 中心のx座標を取得
    pub fn center_x(&self) -> T {
        self.center.x()
    }

    /// 中心のy座標を取得
    pub fn center_y(&self) -> T {
        self.center.y()
    }
}

impl<T: Scalar> Circle<T> for Circle2DImpl<T> {
    type Point = Point2D<T>;
    type Vector = Point2D<T>; // 2Dベクトルとして Point2D を使用

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let distance_sq = self.center.distance_squared_to(*point);
        distance_sq <= self.radius * self.radius
    }

    fn point_on_circumference(&self, point: &Self::Point) -> bool {
        let distance = self.center.distance_to(*point);
        distance.approx_eq(self.radius)
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point {
        let (sin_a, cos_a) = (angle.sin(), angle.cos());
        Point2D::new(
            self.center.x() + self.radius * cos_a,
            self.center.y() + self.radius * sin_a,
        )
    }

    fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector {
        let (sin_a, cos_a) = (angle.sin(), angle.cos());
        Point2D::new(-sin_a, cos_a) // 接線方向
    }

    fn normal_at_angle(&self, angle: Angle<T>) -> Self::Vector {
        let (sin_a, cos_a) = (angle.sin(), angle.cos());
        Point2D::new(cos_a, sin_a) // 法線方向（外向き）
    }
}

/// 3D円の具体実装（基本版）
#[derive(Debug, Clone, PartialEq)]
pub struct Circle3DImpl<T: Scalar> {
    /// 中心点
    center: Point3D<T>,
    /// 半径
    radius: T,
    /// 法線ベクトル（正規化済み）
    normal: Vector3D<T>,
}

impl<T: Scalar> Circle3DImpl<T> {
    /// 新しい3D円を作成
    pub fn new(center: Point3D<T>, radius: T, normal: Vector3D<T>) -> Self {
        let normalized_normal = normal.normalized();
        
        Circle3DImpl {
            center,
            radius,
            normal: normalized_normal,
        }
    }

    /// 平面上の点かどうかを判定
    pub fn point_on_plane(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let vec_to_point = Point3D::new(
            point.x() - self.center.x(),
            point.y() - self.center.y(),
            point.z() - self.center.z(),
        );
        // 簡易実装：Z軸との内積のみチェック
        (vec_to_point.z() * self.normal.z()).abs() <= tolerance
    }

    /// U軸を取得（簡易版）
    pub fn u_axis(&self) -> Vector3D<T> {
        // 法線に垂直な任意のベクトルを作成
        if self.normal.x().abs() > T::TOLERANCE {
            Vector3D::new(T::ZERO, T::ONE, T::ZERO)
        } else {
            Vector3D::new(T::ONE, T::ZERO, T::ZERO)
        }
    }

    /// V軸を取得（簡易版）
    pub fn v_axis(&self) -> Vector3D<T> {
        self.normal.cross(&self.u_axis()).normalized()
    }
}

impl<T: Scalar> Circle<T> for Circle3DImpl<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        if !self.point_on_plane(point, T::TOLERANCE) {
            return false;
        }
        
        let distance_sq = self.center.distance_squared_to(*point);
        distance_sq <= self.radius * self.radius
    }

    fn point_on_circumference(&self, point: &Self::Point) -> bool {
        if !self.point_on_plane(point, T::TOLERANCE) {
            return false;
        }
        
        let distance = self.center.distance_to(*point);
        (distance - self.radius).abs() <= T::TOLERANCE
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // 簡易実装：XY平面での円
        Point3D::new(
            self.center.x() + self.radius * cos_a,
            self.center.y() + self.radius * sin_a,
            self.center.z(),
        )
    }

    fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let u_axis = self.u_axis();
        let v_axis = self.v_axis();
        
        u_axis * (-sin_a) + v_axis * cos_a
    }

    fn normal_at_angle(&self, _angle: Angle<T>) -> Self::Vector {
        self.normal
    }
}

impl<T: Scalar> Circle3D<T> for Circle3DImpl<T> {
    type Normal = Vector3D<T>;

    fn normal(&self) -> Self::Normal {
        self.normal
    }

    fn rotated_around_axis(&self, _axis: &Self::Vector, _angle: Angle<T>) -> Self {
        // TODO: 軸周りの回転実装
        self.clone()
    }

    fn project_to_2d(&self) -> Circle2DImpl<T> {
        // 原点を中心とした2D投影
        Circle2DImpl::new(Point2D::new(T::ZERO, T::ZERO), self.radius)
    }

    fn distance_to_point_3d(&self, point: &Self::Point) -> T {
        // 簡易実装：中心からの距離と半径の差
        let distance_to_center = self.center.distance_to(*point);
        (distance_to_center - self.radius).abs()
    }
}

impl<T: Scalar> Circle2D<T> for Circle2DImpl<T> {
    fn distance_to_point(&self, point: &Self::Point) -> T {
        let distance_to_center = self.center.distance_to(*point);
        (distance_to_center - self.radius).abs()
    }

    fn scaled(&self, factor: T) -> Self {
        Self::new(self.center, self.radius * factor)
    }

    fn translated(&self, offset: &Self::Vector) -> Self {
        Self::new(self.center + *offset, self.radius)
    }

    fn from_three_points(p1: Self::Point, p2: Self::Point, p3: Self::Point) -> Option<Self> {
        // 3点から外接円を計算
        let x1 = p1.x();
        let y1 = p1.y();
        let x2 = p2.x();
        let y2 = p2.y();
        let x3 = p3.x();
        let y3 = p3.y();

        // 行列式を計算して3点が一直線上にないことを確認
        let det = (x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1);
        if det.abs() < T::TOLERANCE {
            return None; // 3点が一直線上にある
        }

        // 外接円の中心を計算
        let a1 = x2 - x1;
        let b1 = y2 - y1;
        let c1 = (x2 * x2 - x1 * x1 + y2 * y2 - y1 * y1) / (T::ONE + T::ONE);

        let a2 = x3 - x1;
        let b2 = y3 - y1;
        let c2 = (x3 * x3 - x1 * x1 + y3 * y3 - y1 * y1) / (T::ONE + T::ONE);

        let det_inv = T::ONE / det;
        let cx = (c1 * b2 - c2 * b1) * det_inv;
        let cy = (a1 * c2 - a2 * c1) * det_inv;

        // 半径を計算
        let center = Point2D::new(cx, cy);
        let radius = center.distance_to(p1);

        Some(Self::new(center, radius))
    }

    fn bounding_circle(points: &[Self::Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        if points.len() == 1 {
            return Some(Self::new(points[0], T::ZERO));
        }

        // 簡単な実装：すべての点を含む最小の円（完全ではない）
        let mut min_x = points[0].x();
        let mut max_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_y = points[0].y();

        for &point in points.iter().skip(1) {
            let x = point.x();
            let y = point.y();
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        let center_x = (min_x + max_x) / (T::ONE + T::ONE);
        let center_y = (min_y + max_y) / (T::ONE + T::ONE);
        let center = Point2D::new(center_x, center_y);

        // 最も遠い点までの距離を半径とする
        let radius = points
            .iter()
            .map(|&point| center.distance_to(point))
            .fold(T::ZERO, |acc, dist| acc.max(dist));

        Some(Self::new(center, radius))
    }
}

// Display実装
impl<T: Scalar> Display for Circle2DImpl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Circle2D(center: {}, radius: {})",
            self.center, self.radius
        )
    }
}

// 便利な型エイリアス
pub type Circle2D32 = Circle2DImpl<f32>;
pub type Circle2D64 = Circle2DImpl<f64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_creation() {
        let center = Point2D::<f64>::new(0.0, 0.0);
        let circle = Circle2DImpl::<f64>::new(center, 5.0);
        assert_eq!(circle.center(), center);
        assert_eq!(circle.radius(), 5.0);
        assert!(circle.is_valid());
    }

    #[test]
    fn test_circle_area_and_circumference() {
        let center = Point2D::<f64>::new(0.0, 0.0);
        let circle = Circle2DImpl::<f64>::new(center, 3.0);
        assert!((circle.area() - (std::f64::consts::PI * 9.0)).abs() < 1e-10);
        assert!((circle.circumference() - (std::f64::consts::TAU * 3.0)).abs() < 1e-10);
    }

    #[test]
    fn test_point_containment() {
        let center = Point2D::<f64>::new(0.0, 0.0);
        let circle = Circle2DImpl::<f64>::new(center, 5.0);

        assert!(circle.contains_point(&Point2D::new(0.0, 0.0))); // 中心
        assert!(circle.contains_point(&Point2D::new(3.0, 4.0))); // 内部（3-4-5三角形）
        assert!(!circle.contains_point(&Point2D::new(4.0, 4.0))); // 外部

        assert!(circle.point_on_circumference(&Point2D::new(5.0, 0.0))); // 円周上
        assert!(circle.point_on_circumference(&Point2D::new(0.0, 5.0))); // 円周上
    }

    #[test]
    fn test_point_at_angle() {
        let center = Point2D::<f64>::new(0.0, 0.0);
        let circle = Circle2DImpl::<f64>::new(center, 1.0);

        let point = circle.point_at_angle(Angle::from_degrees(0.0));
        assert!((point.x() - 1.0).abs() < 1e-10);
        assert!(point.y().abs() < 1e-10);

        let point = circle.point_at_angle(Angle::from_degrees(90.0));
        assert!(point.x().abs() < 1e-10);
        assert!((point.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_circle_transformations() {
        let center = Point2D::<f64>::new(1.0, 1.0);
        let circle = Circle2DImpl::<f64>::new(center, 2.0);

        let scaled = circle.scaled(2.0);
        assert_eq!(scaled.radius(), 4.0);
        assert_eq!(scaled.center(), center);

        let offset = Point2D::<f64>::new(3.0, 4.0);
        let translated = circle.translated(&offset);
        assert_eq!(translated.center(), Point2D::new(4.0, 5.0));
        assert_eq!(translated.radius(), 2.0);
    }

    #[test]
    fn test_three_point_circle() {
        let p1 = Point2D::<f64>::new(0.0, 0.0);
        let p2 = Point2D::<f64>::new(1.0, 0.0);
        let p3 = Point2D::<f64>::new(0.0, 1.0);

        let circle = Circle2DImpl::<f64>::from_three_points(p1, p2, p3).unwrap();

        // 3点すべてが円周上にあることを確認
        assert!(circle.point_on_circumference(&p1));
        assert!(circle.point_on_circumference(&p2));
        assert!(circle.point_on_circumference(&p3));
    }

    #[test]
    fn test_distance_to_point() {
        let center = Point2D::<f64>::new(0.0, 0.0);
        let circle = Circle2DImpl::<f64>::new(center, 5.0);

        assert!((circle.distance_to_point(&Point2D::new(0.0, 0.0)) - 5.0).abs() < 1e-10); // 中心からの距離
        assert!((circle.distance_to_point(&Point2D::new(5.0, 0.0))).abs() < 1e-10); // 円周上
        assert!((circle.distance_to_point(&Point2D::new(10.0, 0.0)) - 5.0).abs() < 1e-10);
        // 外部
    }

    #[test]
    fn test_angle_operations() {
        let center = Point2D::<f32>::new(0.0, 0.0);
        let circle = Circle2DImpl::<f32>::new(center, 1.0);
        let angle = Angle::<f32>::from_degrees(45.0);

        let tangent = circle.tangent_at_angle(angle);
        let normal = circle.normal_at_angle(angle);

        // 接線と法線は垂直であることを確認
        let dot_product = tangent.dot(normal);
        assert!(dot_product.abs() < 1e-6);
    }
}
