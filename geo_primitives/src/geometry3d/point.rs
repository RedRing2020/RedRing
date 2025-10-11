use crate::geometry3d::Vector;
use geo_foundation::abstract_types::geometry::{Point as PointTrait, Point3D as Point3DTrait};
use geo_foundation::{Scalar, ToleranceContext, TolerantEq};

/// ジェネリック3D点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

impl<T: Scalar> Point<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }
    pub fn z(&self) -> T {
        self.z
    }

    pub fn distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Point間のベクトルを取得
    pub fn vector_to(&self, other: &Self) -> Vector<T> {
        Vector::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    /// Vectorで平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        Self::new(
            self.x + vector.x(),
            self.y + vector.y(),
            self.z + vector.z(),
        )
    }

    /// スカラー倍（原点からの拡大・縮小）
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// 中点を計算
    pub fn midpoint(&self, other: &Self) -> Self {
        let two = T::ONE + T::ONE;
        Self::new(
            (self.x + other.x) / two,
            (self.y + other.y) / two,
            (self.z + other.z) / two,
        )
    }

    /// 2D点への投影（Z座標を破棄）
    pub fn to_point2d(&self) -> crate::geometry2d::Point2D<T> {
        crate::geometry2d::Point2D::new(self.x, self.y)
    }

    /// XY平面上での距離計算
    pub fn xy_distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

// 演算子オーバーロード
impl<T: Scalar> std::ops::Add<Vector<T>> for Point<T> {
    type Output = Self;

    fn add(self, vector: Vector<T>) -> Self::Output {
        self.translate(&vector)
    }
}

impl<T: Scalar> std::ops::Sub<Point<T>> for Point<T> {
    type Output = Vector<T>;

    fn sub(self, other: Point<T>) -> Self::Output {
        // self - other = Vector from other to self
        other.vector_to(&self)
    }
}

impl<T: Scalar> std::ops::Sub<Vector<T>> for Point<T> {
    type Output = Self;

    fn sub(self, vector: Vector<T>) -> Self::Output {
        Self::new(
            self.x - vector.x(),
            self.y - vector.y(),
            self.z - vector.z(),
        )
    }
}

impl<T: Scalar> std::ops::Mul<T> for Point<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        self.scale(scalar)
    }
}

impl<T: Scalar> std::ops::Div<T> for Point<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl<T: Scalar> Default for Point<T> {
    fn default() -> Self {
        Self::origin()
    }
}

// geo_foundationトレイトの実装
impl<T: Scalar> TolerantEq for Point<T> {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        let distance = self.distance_to(other);
        distance < T::from_f64(context.tolerance())
    }
}

impl<T: Scalar> PointTrait<T, 3> for Point<T> {
    type Vector = Vector<T>;

    fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    fn distance_to(&self, other: &Self) -> T {
        self.distance_to(other)
    }

    fn translate(&self, vector: &Self::Vector) -> Self {
        Self::new(
            self.x + vector.x(),
            self.y + vector.y(),
            self.z + vector.z(),
        )
    }

    fn vector_to(&self, other: &Self) -> Self::Vector {
        Vector::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    fn coords(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

impl<T: Scalar> Point3DTrait<T> for Point<T> {
    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn z(&self) -> T {
        self.z
    }

    fn from_components(x: T, y: T, z: T) -> Self {
        Self::new(x, y, z)
    }
}

// 型エイリアス（後方互換性確保）
/// 3D Point用の型エイリアス
pub type Point3D<T> = Point<T>;

/// f64版の3D Point（デフォルト）
pub type Point3DF64 = Point<f64>;

/// f32版の3D Point（高速演算用）
pub type Point3DF32 = Point<f32>;
