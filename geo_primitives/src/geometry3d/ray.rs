//! Ray3D - ジェネリック3Dレイ（半無限直線）の実装
//!
//! 起点と方向を持つ3次元半無限直線をサポート
//! 新しい設計では、InfiniteLine3Dを継承した制約版として実装されます

use crate::geometry3d::{Direction3D, Point3D, Vector};
use geo_foundation::abstract_types::geometry::Direction;
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

    /// レイ固有：制約付きの点取得（t >= 0のみ）
    pub fn point_at_parameter_ray(&self, t: T) -> Option<Point3D<T>> {
        if t >= T::ZERO {
            Some(self.origin + self.direction.to_vector() * t)
        } else {
            None // 半無限直線なので t < 0 は無効
        }
    }

    /// レイ固有：制約付きの点判定（前方のみ）
    pub fn contains_point_ray(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point_unlimited(point);
        if distance <= tolerance {
            let param = self.parameter_at_point_unlimited(point);
            param >= -tolerance // 許容誤差を考慮して前方判定
        } else {
            false
        }
    }

    /// 制約なしの距離計算（InfiniteLineの機能）
    pub fn distance_to_point_unlimited(&self, point: &Point3D<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();

        // 点からレイへの投影
        let projection_length = to_point.dot(&direction_vector);
        let projection_point = self.origin + direction_vector * projection_length;
        point.distance_to(&projection_point)
    }

    /// 制約なしのパラメータ計算（InfiniteLineの機能）
    pub fn parameter_at_point_unlimited(&self, point: &Point3D<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        to_point.dot(&direction_vector)
    }

    /// レイの起点を取得（InfiniteLineの origin メソッドと互換）
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// レイの方向を取得（InfiniteLineの direction メソッドと互換）
    pub fn direction(&self) -> Direction3D<T> {
        self.direction
    }

    /// 制約なしの点取得（InfiniteLineの point_at_parameter メソッドと互換）
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        self.origin + self.direction.to_vector() * t
    }
}

// 型エイリアス（後方互換性確保）
/// f64版の3D Ray（デフォルト）
pub type Ray3DF64 = Ray3D<f64>;

/// f32版の3D Ray（高速演算用）
pub type Ray3DF32 = Ray3D<f32>;
