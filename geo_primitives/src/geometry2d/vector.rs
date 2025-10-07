/// f64ベース2Dベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// 単位ベクトル（各軸）
    pub fn unit_x() -> Self {
        Self::new(1.0, 0.0)
    }
    pub fn unit_y() -> Self {
        Self::new(0.0, 1.0)
    }

    /// 成分アクセス
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }

    /// 成分設定（geo_core互換）
    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }
    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }

    /// ベクトルの長さ（ノルム）
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len == 0.0 {
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
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 90度回転（反時計回り）
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    // scaleメソッドを削除 - *演算子を使用

    /// 2点間のベクトル
    pub fn from_points(from: &crate::geometry2d::Point2D, to: &crate::geometry2d::Point2D) -> Self {
        Self::new(to.x() - from.x(), to.y() - from.y())
    }

    /// 配列として取得
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    /// 2D外積（スカラー値）
    pub fn cross_2d(&self, other: &Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// ノルム（長さ）- modelトレイト互換
    pub fn norm(&self) -> f64 {
        self.length()
    }

    /// スカラー加算（スケール加算）- modelトレイト互換
    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {
        Self::new(self.x + other.x * scale, self.y + other.y * scale)
    }

    /// 角度による回転
    pub fn rotate(&self, angle: f64) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }
}

impl Default for Vector {
    fn default() -> Self {
        Self::zero()
    }
}

// geo_foundation Vector トレイトの実装
impl crate::traits::Vector<2> for Vector {
    type Scalar = f64;

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
        (self.length() - 1.0).abs() < tolerance
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
impl crate::traits::Vector2D for Vector {
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

impl crate::traits::Vector2DExt for Vector {
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
impl crate::traits::Normalizable for Vector {
    type Output = Vector;

    fn normalize(&self) -> Option<Self::Output> {
        self.normalize()
    }

    fn normalize_or_zero(&self) -> Self::Output {
        self.normalize_or_zero()
    }

    fn can_normalize(&self, tolerance: f64) -> bool {
        self.length() > tolerance
    }
}

// Index トレイトの実装
impl std::ops::Index<usize> for Vector {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vector index out of bounds: {}", index),
        }
    }
}

// IndexMut トレイトの実装
impl std::ops::IndexMut<usize> for Vector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vector index out of bounds: {}", index),
        }
    }
}

// 算術演算の実装
impl std::ops::Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Self::Output {
        Vector::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Self::Output {
        Vector::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl std::ops::Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, vector: Vector) -> Self::Output {
        Vector::new(vector.x * self, vector.y * self)
    }
}

impl std::ops::Div<f64> for Vector {
    type Output = Vector;

    fn div(self, scalar: f64) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl std::ops::Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y)
    }
}

// Display実装は別クレートで実装

// テストコードはunit_tests/Vector_tests.rsに移動
