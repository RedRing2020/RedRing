//! Ray3D - ジェネリック3Dレイ（半無限直線）の実装
//!
//! 起点と方向を持つ3次元半無限直線をサポート

use crate::geometry3d::{Direction3D, Point3D, Vector};
use geo_foundation::abstract_types::geometry::{
    Direction, Direction3D as Direction3DTrait, Ray, Ray3D as Ray3DTrait,
};
use geo_foundation::abstract_types::geometry::common::{CurveAnalysis3D, AnalyticalCurve, CurveType, DifferentialGeometry};
use geo_foundation::abstract_types::{Angle, Scalar};

/// ジェネリック3Dレイ（半無限直線）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D<T: Scalar> {
    /// レイの起点
    origin: Point3D<T>,
    /// レイの方向ベクトル（正規化済み）
    direction: Direction3D<T>,
}

impl<T: Scalar> Ray3D<T> {
    /// 新しいRay3Dを作成（方向ベクトルは内部で正規化）
    pub fn new(origin: Point3D<T>, direction: Vector<T>) -> Option<Self> {
        Direction3D::from_vector(direction).map(|normalized_dir| Self {
            origin,
            direction: normalized_dir,
        })
    }

    /// Direction3Dから直接作成
    pub fn from_direction(origin: Point3D<T>, direction: Direction3D<T>) -> Self {
        Self { origin, direction }
    }

    /// 2点を通るレイを作成（fromからtoward方向）
    pub fn from_two_points(from: Point3D<T>, toward: Point3D<T>) -> Option<Self> {
        let direction_vector = toward - from;
        Self::new(from, direction_vector)
    }

    /// X軸正方向のレイを作成
    pub fn x_axis(origin: Point3D<T>) -> Self {
        Self {
            origin,
            direction: Direction3D::positive_x(),
        }
    }

    /// Y軸正方向のレイを作成
    pub fn y_axis(origin: Point3D<T>) -> Self {
        Self {
            origin,
            direction: Direction3D::positive_y(),
        }
    }

    /// Z軸正方向のレイを作成
    pub fn z_axis(origin: Point3D<T>) -> Self {
        Self {
            origin,
            direction: Direction3D::positive_z(),
        }
    }

    /// レイを平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        Self {
            origin: self.origin + *vector,
            direction: self.direction,
        }
    }

    /// レイをX軸周りで回転
    pub fn rotate_x(&self, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let rotated_origin = Point3D::new(
            self.origin.x(),
            self.origin.y() * cos_a - self.origin.z() * sin_a,
            self.origin.y() * sin_a + self.origin.z() * cos_a,
        );

        let dir = self.direction.to_vector();
        let rotated_direction_vec = Vector::new(
            dir.x(),
            dir.y() * cos_a - dir.z() * sin_a,
            dir.y() * sin_a + dir.z() * cos_a,
        );

        Self {
            origin: rotated_origin,
            direction: Direction3D::from_vector(rotated_direction_vec).unwrap(),
        }
    }

    /// レイをY軸周りで回転
    pub fn rotate_y(&self, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let rotated_origin = Point3D::new(
            self.origin.x() * cos_a + self.origin.z() * sin_a,
            self.origin.y(),
            -self.origin.x() * sin_a + self.origin.z() * cos_a,
        );

        let dir = self.direction.to_vector();
        let rotated_direction_vec = Vector::new(
            dir.x() * cos_a + dir.z() * sin_a,
            dir.y(),
            -dir.x() * sin_a + dir.z() * cos_a,
        );

        Self {
            origin: rotated_origin,
            direction: Direction3D::from_vector(rotated_direction_vec).unwrap(),
        }
    }

    /// レイをZ軸周りで回転
    pub fn rotate_z(&self, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let rotated_origin = Point3D::new(
            self.origin.x() * cos_a - self.origin.y() * sin_a,
            self.origin.x() * sin_a + self.origin.y() * cos_a,
            self.origin.z(),
        );

        let dir = self.direction.to_vector();
        let rotated_direction_vec = Vector::new(
            dir.x() * cos_a - dir.y() * sin_a,
            dir.x() * sin_a + dir.y() * cos_a,
            dir.z(),
        );

        Self {
            origin: rotated_origin,
            direction: Direction3D::from_vector(rotated_direction_vec).unwrap(),
        }
    }

    /// レイをオイラー角で回転（X, Y, Z軸の順番）
    pub fn rotate_euler(&self, x_angle: Angle<T>, y_angle: Angle<T>, z_angle: Angle<T>) -> Self {
        self.rotate_x(x_angle).rotate_y(y_angle).rotate_z(z_angle)
    }

    /// レイの方向ベクトルを取得
    pub fn direction_vector(&self) -> Vector<T> {
        self.direction.to_vector()
    }
}

impl<T: Scalar> Ray<T> for Ray3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>; // Direction3D<T>を使用
    type Error = String;

    fn origin(&self) -> Self::Point {
        self.origin
    }

    fn direction(&self) -> Self::Direction {
        self.direction
    }

    fn point_at_parameter(&self, t: T) -> Option<Self::Point> {
        if t >= T::ZERO {
            Some(self.origin + self.direction.to_vector() * t)
        } else {
            None // 半無限直線なので t < 0 は無効
        }
    }

    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool {
        let distance = self.distance_to_point(point);
        if distance <= tolerance {
            // レイ上にある場合、パラメータが非負かチェック
            let parameter = self.parameter_at_point(point);
            parameter >= -tolerance // 小さな誤差は許容
        } else {
            false
        }
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();

        // 点からレイへの投影
        let projection_length = to_point.dot(&direction_vector);

        if projection_length <= T::ZERO {
            // 起点より後方の場合、起点までの距離
            self.origin.distance_to(point)
        } else {
            // 投影点がレイ上にある場合
            let projection_point = self.origin + direction_vector * projection_length;
            point.distance_to(&projection_point)
        }
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();

        let projection_length = to_point.dot(&direction_vector);

        if projection_length <= T::ZERO {
            // 起点より後方の場合、起点が最近点
            self.origin
        } else {
            // 投影点がレイ上の最近点
            self.origin + direction_vector * projection_length
        }
    }

    fn parameter_at_point(&self, point: &Self::Point) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        to_point.dot(&direction_vector)
    }

    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        // 方向ベクトルの外積がゼロに近いかチェック
        let cross = self.direction.cross(&other.direction);
        cross.norm() < tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool {
        // 方向が平行かつ起点がもう一方のレイ上にある
        self.is_parallel_to(other, tolerance) && other.contains_point(&self.origin, tolerance)
    }
}

impl<T: Scalar> Ray3DTrait<T> for Ray3D<T> {
    // 現在は3D固有のメソッドはなし
    // 将来必要に応じて追加
}

// 型エイリアス（後方互換性確保）
/// f64版の3D Ray（デフォルト）
pub type Ray3DF64 = Ray3D<f64>;

/// f32版の3D Ray（高速演算用）
pub type Ray3DF32 = Ray3D<f32>;

// =============================================================================
// 統一曲線解析インターフェイスの実装
// =============================================================================

/// Ray3D<T>に統一曲線解析インターフェイスを実装
/// 半直線は直線の特殊ケース（開始点あり、終了点なし）
impl<T: Scalar> CurveAnalysis3D<T> for Ray3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>;

    /// 指定されたパラメータ位置での点を取得
    /// t: 0.0=起点、正の値で方向ベクトル方向に伸びる
    fn point_at_parameter(&self, t: T) -> Self::Point {
        if t < T::ZERO {
            // 負のパラメータでは起点を返す（半直線の制限）
            self.origin
        } else {
            let dir_vec = self.direction.to_vector();
            self.origin + dir_vec * t
        }
    }

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    /// 半直線の接線は常に方向ベクトルと同じ
    fn tangent_at_parameter(&self, _t: T) -> Self::Vector {
        self.direction.to_vector()
    }

    /// 指定されたパラメータ位置での主法線ベクトルを取得（正規化済み）
    /// 直線では主法線が定義されないため、適切なデフォルト値を返す
    fn normal_at_parameter(&self, _t: T) -> Self::Vector {
        // InfiniteLine3Dと同じロジック：方向ベクトルに垂直な任意のベクトル
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
    /// 半直線の双法線は主法線と接線の外積
    fn binormal_at_parameter(&self, t: T) -> Self::Vector {
        let tangent = self.tangent_at_parameter(t);
        let normal = self.normal_at_parameter(t);
        let cross = tangent.cross(&normal);
        cross.normalize().unwrap_or(cross)
    }

    /// 指定されたパラメータ位置での曲率を取得
    /// 半直線の曲率は常にゼロ
    fn curvature_at_parameter(&self, _t: T) -> T {
        T::ZERO
    }

    /// 指定されたパラメータ位置での捩率（ねじれ）を取得
    /// 半直線の捩率は常にゼロ
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

    /// 最大曲率の位置と値を取得（半直線は曲率ゼロ）
    fn max_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ZERO)) // 任意の位置で曲率ゼロ
    }

    /// 最小曲率の位置と値を取得（半直線は曲率ゼロ）
    fn min_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ZERO)) // 任意の位置で曲率ゼロ
    }

    /// 曲率がゼロになる位置を取得（半直線では全ての位置で曲率ゼロ）
    fn inflection_points(&self) -> Vec<T> {
        Vec::new() // 無限に多い点があるため空のベクトルを返す
    }

    /// 曲線が平面曲線かどうかを判定（半直線は常に平面曲線）
    fn is_planar(&self) -> bool {
        true
    }
}

/// Ray3D<T>に解析的曲線インターフェイスを実装
impl<T: Scalar> AnalyticalCurve<T> for Ray3D<T> {
    /// 曲線の種類（直線として扱う）
    fn curve_type(&self) -> CurveType {
        CurveType::Line
    }

    /// 一定曲率かどうか（半直線は常に一定曲率：ゼロ）
    fn has_constant_curvature(&self) -> bool {
        true
    }

    /// 解析的に計算可能な曲率の定数値（半直線の場合：ゼロ）
    fn constant_curvature(&self) -> Option<T> {
        Some(T::ZERO)
    }

    /// 解析的に計算可能な曲率半径の定数値（半直線の場合：無限大）
    fn constant_curvature_radius(&self) -> Option<T> {
        Some(T::INFINITY)
    }
}
