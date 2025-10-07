/// f64ベース3Dベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// 単位ベクトル（各軸）
    pub fn unit_x() -> Self { Self::new(1.0, 0.0, 0.0) }
    pub fn unit_y() -> Self { Self::new(0.0, 1.0, 0.0) }
    pub fn unit_z() -> Self { Self::new(0.0, 0.0, 1.0) }

    /// 成分アクセス
    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }

    /// 成分設定（geo_core互換）
    pub fn set_x(&mut self, x: f64) { self.x = x; }
    pub fn set_y(&mut self, y: f64) { self.y = y; }
    pub fn set_z(&mut self, z: f64) { self.z = z; }

    /// ベクトルの長さ（ノルム）
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len == 0.0 {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        }
    }

    /// 正規化（ゼロベクトルの場合はゼロベクトルを返す）
    pub fn normalize_or_zero(&self) -> Self {
        self.normalize().unwrap_or(Self::zero())
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 外積
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
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
            self.z + other.z * scale,
        )
    }

    /// 軸エイリアス（geo_core互換）
    pub fn x_axis() -> Self { Self::unit_x() }
    pub fn y_axis() -> Self { Self::unit_y() }
    pub fn z_axis() -> Self { Self::unit_z() }

    /// スカラー三重積 (scalar triple product)
    pub fn scalar_triple_product(&self, b: &Self, c: &Self) -> f64 {
        let cross = b.cross(c);
        self.dot(&cross)
    }

    /// ベクトル三重積 (vector triple product)
    pub fn vector_triple_product(&self, b: &Self, c: &Self) -> Self {
        // a × (b × c) = b (a·c) - c (a·b)
        let a_dot_c = self.dot(c);
        let a_dot_b = self.dot(b);
        *b * a_dot_c - *c * a_dot_b
    }

    // scaleメソッドを削除 - *演算子を使用

    /// 2点間のベクトル
    pub fn from_points(from: &crate::geometry3d::Point3D, to: &crate::geometry3d::Point3D) -> Self {
        Self::new(to.x() - from.x(), to.y() - from.y(), to.z() - from.z())
    }

    /// 2Dベクトルへの投影（Z成分を無視）
    pub fn to_vector2d(&self) -> crate::geometry2d::Vector2D {
        crate::geometry2d::Vector2D::new(self.x, self.y)
    }

    /// 配列として取得
    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

impl Default for Vector3D {
    fn default() -> Self {
        Self::zero()
    }
}

// Vector トレイトの実装
impl crate::traits::Vector<3> for Vector3D {
    fn new(components: [f64; 3]) -> Self {
        Self::new(components[0], components[1], components[2])
    }

    fn components(&self) -> &[f64; 3] {
        unsafe {
            std::mem::transmute::<&Vector3D, &[f64; 3]>(self)
        }
    }

    fn components_mut(&mut self) -> &mut [f64; 3] {
        unsafe {
            std::mem::transmute::<&mut Vector3D, &mut [f64; 3]>(self)
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
        let cross = self.cross(other);
        cross.length() < tolerance
    }

    fn component_min(&self, other: &Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    fn component_max(&self, other: &Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }

    fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    fn scale(&self, scalar: f64) -> Self {
        *self * scalar
    }

    fn zero() -> Self {
        Self::zero()
    }
}

// Vector3DExt トレイトの実装
impl crate::traits::Vector3DExt for Vector3D {
    fn cross(&self, other: &Self) -> Self {
        self.cross(other)
    }

    fn to_2d_xy(&self) -> crate::geometry2d::Vector2D {
        crate::geometry2d::Vector2D::new(self.x, self.y)
    }

    fn to_2d_xz(&self) -> crate::geometry2d::Vector2D {
        crate::geometry2d::Vector2D::new(self.x, self.z)
    }

    fn to_2d_yz(&self) -> crate::geometry2d::Vector2D {
        crate::geometry2d::Vector2D::new(self.y, self.z)
    }
}

// Normalizable トレイトの実装
impl crate::traits::Normalizable for Vector3D {
    type Output = Vector3D;

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
impl std::ops::Index<usize> for Vector3D {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vector3D index out of bounds: {}", index),
        }
    }
}

// IndexMut トレイトの実装
impl std::ops::IndexMut<usize> for Vector3D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vector3D index out of bounds: {}", index),
        }
    }
}

// 算術演算の実装
impl std::ops::Add for Vector3D {
    type Output = Vector3D;

    fn add(self, other: Vector3D) -> Self::Output {
        Vector3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Vector3D;

    fn sub(self, other: Vector3D) -> Self::Output {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f64> for Vector3D {
    type Output = Vector3D;

    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl std::ops::Mul<Vector3D> for f64 {
    type Output = Vector3D;

    fn mul(self, vector: Vector3D) -> Self::Output {
        Vector3D::new(vector.x * self, vector.y * self, vector.z * self)
    }
}

impl std::ops::Div<f64> for Vector3D {
    type Output = Vector3D;

    fn div(self, scalar: f64) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Vector3D;

    fn neg(self) -> Self::Output {
        Vector3D::new(-self.x, -self.y, -self.z)
    }
}

impl std::fmt::Display for Vector3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector3D({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
    }
}

// テストコードはunit_tests/vector3d_tests.rsに移動
