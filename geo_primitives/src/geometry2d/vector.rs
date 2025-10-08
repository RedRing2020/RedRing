use geo_foundation::abstract_types::Scalar;

/// 型パラメータ化された2Dベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector<T: Scalar> {
    x: T,
    y: T,
}

impl<T: Scalar> Vector<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO)
    }

    /// 単位ベクトル（各軸）
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO)
    }
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE)
    }

    /// 成分アクセス
    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }

    /// 成分設定（geo_core互換）
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }
    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }

    /// ベクトルの長さ（ノルム）
    pub fn length(&self) -> T {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len == T::ZERO {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len))
        }
    }

    /// 正規化（ゼロベクトルの場合はゼロベクトルを返す）
    pub fn normalize_or_zero(&self) -> Self {
        self.normalize().unwrap_or_default()
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// 90度回転（反時計回り）
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    // scaleメソッドを削除 - *演算子を使用

    /// 2点間のベクトル
    pub fn from_points(from: &crate::geometry2d::Point<T>, to: &crate::geometry2d::Point<T>) -> Self {
        Self::new(to.x() - from.x(), to.y() - from.y())
    }

    /// 配列として取得
    pub fn to_array(&self) -> [T; 2] {
        [self.x, self.y]
    }

    /// 2D外積（スカラー値）
    pub fn cross_2d(&self, other: &Self) -> T {
        self.x * other.y - self.y * other.x
    }

    /// ノルム（長さ）- modelトレイト互換
    pub fn norm(&self) -> T {
        self.length()
    }

    /// スカラー加算（スケール加算）- modelトレイト互換
    pub fn add_scaled(&self, other: &Self, scale: T) -> Self {
        Self::new(self.x + other.x * scale, self.y + other.y * scale)
    }

    /// 角度による回転
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }
}

impl<T: Scalar> Default for Vector<T> {
    fn default() -> Self {
        Self::zero()
    }
}

// geo_foundation Vector トレイトの実装
impl<T: Scalar> crate::traits::Vector<2> for Vector<T> {
    type Scalar = T;

    fn from_components(components: [Self::Scalar; 2]) -> Self {
        Self::new(components[0], components[1])
    }

    fn components(&self) -> [Self::Scalar; 2] {
        [self.x, self.y]
    }

    fn zero() -> Self {
        Self::zero()
    }

    fn dot(&self, other: &Self) -> Self::Scalar {
        Vector::dot(self, other)
    }

    fn length(&self) -> Self::Scalar {
        Vector::length(self)
    }

    fn normalize(&self) -> Option<Self> {
        Vector::normalize(self)
    }

    fn is_zero(&self, tolerance: Self::Scalar) -> bool {
        self.length() < tolerance
    }

    fn is_unit(&self, tolerance: Self::Scalar) -> bool {
        (self.length() - T::ONE).abs() < tolerance
    }

    fn is_parallel_to(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        let cross = self.cross_2d(other);
        cross.abs() < tolerance
    }

    fn is_perpendicular_to(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        let dot = self.dot(other);
        dot.abs() < tolerance
    }
}

// VectorExt トレイトの実装
// geo_foundation Vector2D トレイトの実装（Vector<2> を前提とする）
impl<T: Scalar> crate::traits::Vector2D for Vector<T> {
    fn x(&self) -> Self::Scalar {
        self.x
    }

    fn y(&self) -> Self::Scalar {
        self.y
    }

    fn new(x: Self::Scalar, y: Self::Scalar) -> Self {
        Vector::new(x, y)
    }

    fn perpendicular(&self) -> Self {
        Vector::perpendicular(self)
    }

    fn cross_2d(&self, other: &Self) -> Self::Scalar {
        Vector::cross_2d(self, other)
    }

    fn unit_x() -> Self {
        Vector::unit_x()
    }

    fn unit_y() -> Self {
        Vector::unit_y()
    }
}

impl<T: Scalar> crate::traits::Vector2DExt for Vector<T> {
    fn from_angle(angle: Self::Scalar) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    fn angle(&self) -> Self::Scalar {
        self.y.atan2(self.x)
    }

    fn angle_to(&self, other: &Self) -> Self::Scalar {
        let cross = self.cross_2d(other);
        let dot = self.dot(other);
        cross.atan2(dot)
    }
}

// Normalizable トレイトの実装
impl<T: Scalar> crate::traits::Normalizable for Vector<T> {
    type Output = Vector<T>;

    fn normalize(&self) -> Option<Self::Output> {
        self.normalize()
    }

    fn normalize_or_zero(&self) -> Self::Output {
        self.normalize_or_zero()
    }

    fn can_normalize(&self, tolerance: f64) -> bool {
        self.length().to_f64() > tolerance
    }
}

// Index トレイトの実装
impl<T: Scalar> std::ops::Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vector index out of bounds: {}", index),
        }
    }
}

// IndexMut トレイトの実装
impl<T: Scalar> std::ops::IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vector index out of bounds: {}", index),
        }
    }
}

// 算術演算の実装
impl<T: Scalar> std::ops::Add for Vector<T> {
    type Output = Vector<T>;

    fn add(self, other: Vector<T>) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl<T: Scalar> std::ops::Sub for Vector<T> {
    type Output = Vector<T>;

    fn sub(self, other: Vector<T>) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl<T: Scalar> std::ops::Mul<T> for Vector<T> {
    type Output = Vector<T>;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl<T: Scalar> std::ops::Div<T> for Vector<T> {
    type Output = Vector<T>;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl<T: Scalar> std::ops::Neg for Vector<T> {
    type Output = Vector<T>;

    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y)
    }
}

// Display実装は別クレートで実装

// テストコードはunit_tests/Vector_tests.rsに移動

// 型エイリアス（後方互換性確保）
/// f64版の2D Vector（デフォルト）
pub type Vector2D = Vector<f64>;

/// f32版の2D Vector（高速演算用）
pub type Vector2Df = Vector<f32>;
