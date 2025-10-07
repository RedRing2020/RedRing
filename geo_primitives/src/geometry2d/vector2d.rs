/// f64ベース2Dベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    x: f64,
    y: f64,
}

impl Vector2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// 単位ベクトル（各軸）
    pub fn unit_x() -> Self { Self::new(1.0, 0.0) }
    pub fn unit_y() -> Self { Self::new(0.0, 1.0) }

    /// 成分アクセス
    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }

    /// 成分設定（geo_core互換）
    pub fn set_x(&mut self, x: f64) { self.x = x; }
    pub fn set_y(&mut self, y: f64) { self.y = y; }

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
        Self::new(
            self.x + other.x * scale,
            self.y + other.y * scale,
        )
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

impl Default for Vector2D {
    fn default() -> Self {
        Self::zero()
    }
}

// Vector トレイトの実装
impl crate::traits::Vector<2> for Vector2D {
    fn new(components: [f64; 2]) -> Self {
        Self::new(components[0], components[1])
    }

    fn components(&self) -> &[f64; 2] {
        // 配列表現に変換して返す必要があるため、一時的に配列を作成
        unsafe {
            std::mem::transmute::<&Vector2D, &[f64; 2]>(self)
        }
    }

    fn components_mut(&mut self) -> &mut [f64; 2] {
        unsafe {
            std::mem::transmute::<&mut Vector2D, &mut [f64; 2]>(self)
        }
    }

    fn dot(&self, other: &Self) -> f64 {
        self.dot(other)
    }

    fn norm(&self) -> f64 {
        self.length()
    }

    fn normalize(&self) -> Option<Self> {
        self.normalize()
    }

    fn is_parallel_to(&self, other: &Self, tolerance: f64) -> bool {
        let cross = self.cross_2d(other);
        cross.abs() < tolerance
    }

    fn component_min(&self, other: &Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    fn component_max(&self, other: &Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    fn scale(&self, scalar: f64) -> Self {
        *self * scalar
    }

    fn zero() -> Self {
        Self::zero()
    }
}

// Vector2DExt トレイトの実装
impl crate::traits::Vector2DExt for Vector2D {
    fn perpendicular(&self) -> Self {
        self.perpendicular()
    }

    fn cross_2d(&self, other: &Self) -> f64 {
        self.cross_2d(other)
    }
}

// Normalizable トレイトの実装
impl crate::traits::Normalizable for Vector2D {
    type Output = Vector2D;

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
impl std::ops::Index<usize> for Vector2D {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vector2D index out of bounds: {}", index),
        }
    }
}

// IndexMut トレイトの実装
impl std::ops::IndexMut<usize> for Vector2D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vector2D index out of bounds: {}", index),
        }
    }
}

// 算術演算の実装
impl std::ops::Add for Vector2D {
    type Output = Vector2D;

    fn add(self, other: Vector2D) -> Self::Output {
        Vector2D::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Vector2D;

    fn sub(self, other: Vector2D) -> Self::Output {
        Vector2D::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Mul<f64> for Vector2D {
    type Output = Vector2D;

    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl std::ops::Mul<Vector2D> for f64 {
    type Output = Vector2D;

    fn mul(self, vector: Vector2D) -> Self::Output {
        Vector2D::new(vector.x * self, vector.y * self)
    }
}

impl std::ops::Div<f64> for Vector2D {
    type Output = Vector2D;

    fn div(self, scalar: f64) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

impl std::ops::Neg for Vector2D {
    type Output = Vector2D;

    fn neg(self) -> Self::Output {
        Vector2D::new(-self.x, -self.y)
    }
}

// Display実装は別クレートで実装

// テストコードはunit_tests/vector2d_tests.rsに移動
