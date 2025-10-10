//! InfiniteLine3D - ジェネリック3D無限直線の実装
//!
//! 3D空間における無限直線の具体的な実装
//! 点と方向ベクトルで定義される、CAD/CAMシステムで使用される直線の基本的な操作を提供

use crate::geometry3d::{Direction3D, Point, Vector};
use geo_foundation::abstract_types::geometry::common::{
    AnalyticalCurve, CurveAnalysis3D, CurveType, DifferentialGeometry,
};
use geo_foundation::{
    abstract_types::geometry::{Direction, InfiniteLine3D as InfiniteLine3DTrait},
    Scalar,
};

/// ジェネリック3D無限直線
///
/// 基準点と方向ベクトルで定義される無限に延びる直線
/// 直線の方程式: point = origin + t * direction (t ∈ ℝ)
#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine3D<T: Scalar> {
    /// 直線上の基準点
    origin: Point<T>,
    /// 直線の方向（正規化済み）
    direction: Direction3D<T>,
}

impl<T: Scalar> InfiniteLine3D<T> {
    /// 点と方向ベクトルから無限直線を作成
    pub fn new(origin: Point<T>, direction: Direction3D<T>) -> Self {
        Self { origin, direction }
    }

    /// 2点を通る無限直線を作成
    pub fn from_two_points(point1: Point<T>, point2: Point<T>) -> Option<Self> {
        let diff = Vector::new(
            point2.x() - point1.x(),
            point2.y() - point1.y(),
            point2.z() - point1.z(),
        );
        let direction = Direction3D::from_vector(diff)?;
        Some(Self::new(point1, direction))
    }

    /// X軸方向の無限直線を作成
    pub fn x_axis(origin: Point<T>) -> Self {
        Self::new(origin, Direction3D::positive_x())
    }

    /// Y軸方向の無限直線を作成
    pub fn y_axis(origin: Point<T>) -> Self {
        Self::new(origin, Direction3D::positive_y())
    }

    /// Z軸方向の無限直線を作成
    pub fn z_axis(origin: Point<T>) -> Self {
        Self::new(origin, Direction3D::positive_z())
    }

    /// X軸に平行な直線を作成
    pub fn along_x_axis(y: T, z: T) -> Self {
        Self::new(Point::new(T::ZERO, y, z), Direction3D::positive_x())
    }

    /// Y軸に平行な直線を作成
    pub fn along_y_axis(x: T, z: T) -> Self {
        Self::new(Point::new(x, T::ZERO, z), Direction3D::positive_y())
    }

    /// Z軸に平行な直線を作成
    pub fn along_z_axis(x: T, y: T) -> Self {
        Self::new(Point::new(x, y, T::ZERO), Direction3D::positive_z())
    }

    /// XY平面に投影した2D直線を取得
    pub fn project_to_xy(&self) -> crate::geometry2d::InfiniteLine2D<f64> {
        use crate::geometry2d::{Direction2D, Point2D, Vector};

        let origin_2d = Point2D::new(self.origin.x().to_f64(), self.origin.y().to_f64());
        let dir_2d = Direction2D::from_vector(Vector::new(
            self.direction.x().to_f64(),
            self.direction.y().to_f64(),
        ))
        .unwrap_or_else(|| {
            // Z方向の場合、任意の方向を選択
            Direction2D::from_vector(crate::geometry2d::Vector2D::new(1.0, 0.0)).unwrap()
        });

        crate::geometry2d::InfiniteLine2D::new(origin_2d, dir_2d)
    }

    /// 指定した平面に投影した直線を取得
    /// 平面への投影は実装が複雑なため、一時的に無効化
    pub fn project_to_plane(
        &self,
        _plane_normal: &Vector<T>,
        _plane_point: &Point<T>,
    ) -> Option<Self> {
        // 複雑な投影計算は後で実装
        None
    }
}

impl<T: Scalar> InfiniteLine3DTrait<T> for InfiniteLine3D<T> {
    type Point = Point<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>;
    type Error = String;

    fn origin(&self) -> Self::Point {
        self.origin
    }

    fn direction(&self) -> Self::Direction {
        self.direction
    }

    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let to_point = Vector::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 外積の大きさが距離に相当
        let cross_product = to_point.cross(&dir_vec);
        cross_product.norm()
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = Vector::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 投影係数を計算
        let projection = to_point.dot(&dir_vec);

        Point::new(
            self.origin.x() + projection * dir_vec.x(),
            self.origin.y() + projection * dir_vec.y(),
            self.origin.z() + projection * dir_vec.z(),
        )
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        let dir_vec = self.direction.to_vector();
        Point::new(
            self.origin.x() + t * dir_vec.x(),
            self.origin.y() + t * dir_vec.y(),
            self.origin.z() + t * dir_vec.z(),
        )
    }

    fn parameter_at_point(&self, point: &Self::Point) -> T {
        let to_point = Vector::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 方向ベクトルへの投影
        to_point.dot(&dir_vec)
    }

    fn intersect_line(&self, other: &Self) -> Option<Self::Point> {
        let p1 = self.origin;
        let d1 = self.direction.to_vector();
        let p2 = other.origin;
        let d2 = other.direction.to_vector();

        let w = Vector::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        let cross_d1_d2 = d1.cross(&d2);
        let cross_norm_sq = cross_d1_d2.length_squared();

        if cross_norm_sq < T::TOLERANCE * T::TOLERANCE {
            return None; // 平行または同一直線
        }

        // スキューライン（ねじれの位置）の場合の最近点対の中点を返す
        let w_cross_d2 = w.cross(&d2);
        let t1 = w_cross_d2.dot(&cross_d1_d2) / cross_norm_sq;

        let w_cross_d1 = w.cross(&d1);
        let t2 = w_cross_d1.dot(&cross_d1_d2) / cross_norm_sq;

        let point1 = InfiniteLine3DTrait::point_at_parameter(self, t1);
        let point2 = InfiniteLine3DTrait::point_at_parameter(other, t2);

        // 2点間の距離が許容誤差内であれば交点とみなす
        if point1.distance_to(&point2) < T::TOLERANCE {
            Some(Point::new(
                (point1.x() + point2.x()) / (T::ONE + T::ONE),
                (point1.y() + point2.y()) / (T::ONE + T::ONE),
                (point1.z() + point2.z()) / (T::ONE + T::ONE),
            ))
        } else {
            None
        }
    }

    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        let dir1 = self.direction.to_vector();
        let dir2 = other.direction.to_vector();

        // 外積の大きさがゼロに近ければ平行
        let cross_product = dir1.cross(&dir2);
        cross_product.length() < tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool {
        // 平行かつ、一方の点がもう一方の直線上にある
        self.is_parallel_to(other, tolerance) && self.distance_to_point(&other.origin) < tolerance
    }

    fn is_skew_to(&self, other: &Self, tolerance: T) -> bool {
        // 平行でなく、交差もしない（ねじれの位置）
        !self.is_parallel_to(other, tolerance) && self.intersect_line(other).is_none()
    }

    fn distance_to_line(&self, other: &Self) -> T {
        let p1 = self.origin;
        let d1 = self.direction.to_vector();
        let p2 = other.origin;
        let d2 = other.direction.to_vector();

        let w = Vector::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        let cross_d1_d2 = d1.cross(&d2);
        let cross_norm = cross_d1_d2.length();

        if cross_norm < T::TOLERANCE {
            // 平行線の場合
            return self.distance_to_point(&p2);
        }

        // スキューラインの場合
        w.dot(&cross_d1_d2).abs() / cross_norm
    }

    fn closest_points_to_line(&self, other: &Self) -> Option<(Self::Point, Self::Point)> {
        let p1 = self.origin;
        let d1 = self.direction.to_vector();
        let p2 = other.origin;
        let d2 = other.direction.to_vector();

        let w = Vector::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        let cross_d1_d2 = d1.cross(&d2);
        let cross_norm_sq = cross_d1_d2.length_squared();

        if cross_norm_sq < T::TOLERANCE * T::TOLERANCE {
            return None; // 平行線
        }

        let w_cross_d2 = w.cross(&d2);
        let t1 = w_cross_d2.dot(&cross_d1_d2) / cross_norm_sq;

        let w_cross_d1 = w.cross(&d1);
        let t2 = w_cross_d1.dot(&cross_d1_d2) / cross_norm_sq;

        let point1 = InfiniteLine3DTrait::point_at_parameter(self, t1);
        let point2 = InfiniteLine3DTrait::point_at_parameter(other, t2);

        Some((point1, point2))
    }

    fn intersect_plane(
        &self,
        plane_point: &Self::Point,
        plane_normal: &Self::Vector,
    ) -> Option<Self::Point> {
        let dir_vec = self.direction.to_vector();
        let denom = dir_vec.dot(plane_normal);

        if denom.abs() < T::TOLERANCE {
            return None; // 直線が平面に平行
        }

        let to_plane = Vector::new(
            plane_point.x() - self.origin.x(),
            plane_point.y() - self.origin.y(),
            plane_point.z() - self.origin.z(),
        );

        let t = to_plane.dot(plane_normal) / denom;
        Some(InfiniteLine3DTrait::point_at_parameter(self, t))
    }

    fn rotate_around_axis(
        &self,
        _axis_point: &Self::Point,
        _axis_direction: &Self::Direction,
        _angle: T,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // 簡単な実装：軸周りの回転は複雑なため、エラーを返す
        Err("Rotation around axis not implemented yet".to_string())
    }
}

// =============================================================================
// 統一曲線解析インターフェイスの実装
// =============================================================================

/// InfiniteLine3D<T>に統一曲線解析インターフェイスを実装
/// 直線は曲率がゼロで接線が一定の特殊ケース
impl<T: Scalar> CurveAnalysis3D<T> for InfiniteLine3D<T> {
    type Point = Point<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>;

    /// 指定されたパラメータ位置での点を取得
    /// t: 任意の実数値（無限直線なので制限なし）
    fn point_at_parameter(&self, t: T) -> Self::Point {
        let dir_vec = self.direction.to_vector();
        Point::new(
            self.origin.x() + t * dir_vec.x(),
            self.origin.y() + t * dir_vec.y(),
            self.origin.z() + t * dir_vec.z(),
        )
    }

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    /// 直線の接線は常に方向ベクトルと同じ
    fn tangent_at_parameter(&self, _t: T) -> Self::Vector {
        self.direction.to_vector()
    }

    /// 指定されたパラメータ位置での主法線ベクトルを取得（正規化済み）
    /// 直線では主法線が定義されないため、適切なデフォルト値を返す
    fn normal_at_parameter(&self, _t: T) -> Self::Vector {
        // 直線では法線が無限にあるため、方向ベクトルに垂直な任意のベクトルを選ぶ
        let dir = self.direction.to_vector();

        // Z方向に垂直でない場合はZ軸との外積を使用
        if dir.z().abs() < T::ONE - T::TOLERANCE {
            let z_axis = Vector::new(T::ZERO, T::ZERO, T::ONE);
            let cross = dir.cross(&z_axis);
            cross.normalize().unwrap_or(cross)
        }
        // Z方向の場合はX軸との外積を使用
        else {
            let x_axis = Vector::new(T::ONE, T::ZERO, T::ZERO);
            let cross = dir.cross(&x_axis);
            cross.normalize().unwrap_or(cross)
        }
    }

    /// 指定されたパラメータ位置での双法線ベクトルを取得（正規化済み）
    /// 直線の双法線は主法線と接線の外積
    fn binormal_at_parameter(&self, t: T) -> Self::Vector {
        let tangent = self.tangent_at_parameter(t);
        let normal = self.normal_at_parameter(t);
        let cross = tangent.cross(&normal);
        cross.normalize().unwrap_or(cross)
    }

    /// 指定されたパラメータ位置での曲率を取得
    /// 直線の曲率は常にゼロ
    fn curvature_at_parameter(&self, _t: T) -> T {
        T::ZERO
    }

    /// 指定されたパラメータ位置での捩率（ねじれ）を取得
    /// 直線の捩率は常にゼロ
    fn torsion_at_parameter(&self, _t: T) -> T {
        T::ZERO
    }

    /// 指定されたパラメータ位置での微分幾何学的情報を一括取得（最も効率的）
    fn differential_geometry_at_parameter(&self, t: T) -> DifferentialGeometry<T, Self::Vector> {
        let tangent = self.tangent_at_parameter(t);
        let normal = self.normal_at_parameter(t);
        let curvature = T::ZERO;

        DifferentialGeometry::new(tangent, normal, curvature)
    }

    /// 最大曲率の位置と値を取得（直線は曲率ゼロ）
    fn max_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ZERO)) // 任意の位置で曲率ゼロ
    }

    /// 最小曲率の位置と値を取得（直線は曲率ゼロ）
    fn min_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ZERO)) // 任意の位置で曲率ゼロ
    }

    /// 曲率がゼロになる位置を取得（直線では全ての位置で曲率ゼロ）
    fn inflection_points(&self) -> Vec<T> {
        Vec::new() // 無限に多い点があるため空のベクトルを返す
    }

    /// 曲線が平面曲線かどうかを判定（直線は常に平面曲線）
    fn is_planar(&self) -> bool {
        true
    }
}

/// InfiniteLine3D<T>に解析的曲線インターフェイスを実装
impl<T: Scalar> AnalyticalCurve<T> for InfiniteLine3D<T> {
    /// 曲線の種類（直線）
    fn curve_type(&self) -> CurveType {
        CurveType::Line
    }

    /// 一定曲率かどうか（直線は常に一定曲率：ゼロ）
    fn has_constant_curvature(&self) -> bool {
        true
    }

    /// 解析的に計算可能な曲率の定数値（直線の場合：ゼロ）
    fn constant_curvature(&self) -> Option<T> {
        Some(T::ZERO)
    }

    /// 解析的に計算可能な曲率半径の定数値（直線の場合：無限大）
    fn constant_curvature_radius(&self) -> Option<T> {
        Some(T::INFINITY)
    }
}
