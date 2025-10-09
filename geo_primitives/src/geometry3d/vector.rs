use geo_foundation::Scalar;

/// 型パラメータ化された3Dベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

impl<T: Scalar> Vector<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// 単位ベクトル（各軸）
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO)
    }
    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE, T::ZERO)
    }
    pub fn unit_z() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ONE)
    }

    /// 成分アクセス
    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }
    pub fn z(&self) -> T {
        self.z
    }

    /// 成分設定（geo_core互換）
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }
    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }
    pub fn set_z(&mut self, z: T) {
        self.z = z;
    }

    /// ベクトルの長さ（ノルム）
    pub fn length(&self) -> T {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// ベクトルの長さの二乗
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len == T::ZERO {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        }
    }

    /// 正規化（ゼロベクトルの場合はゼロベクトルを返す）
    pub fn normalize_or_zero(&self) -> Self {
        self.normalize().unwrap_or_default()
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
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
    pub fn norm(&self) -> T {
        self.length()
    }

    /// スカラー加算（スケール加算）- modelトレイト互換
    pub fn add_scaled(&self, other: &Self, scale: T) -> Self {
        Self::new(
            self.x + other.x * scale,
            self.y + other.y * scale,
            self.z + other.z * scale,
        )
    }

    /// 軸エイリアス（geo_core互換）
    pub fn x_axis() -> Self {
        Self::unit_x()
    }
    pub fn y_axis() -> Self {
        Self::unit_y()
    }
    pub fn z_axis() -> Self {
        Self::unit_z()
    }

    /// スカラー三重積 (scalar triple product)
    pub fn scalar_triple_product(&self, b: &Self, c: &Self) -> T {
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
    pub fn from_points(
        from: &crate::geometry3d::point::Point3D<T>,
        to: &crate::geometry3d::point::Point3D<T>,
    ) -> Self {
        Self::new(to.x() - from.x(), to.y() - from.y(), to.z() - from.z())
    }

    /// 2Dベクトルへの投影（Z成分を無視）
    pub fn to_vector2d(&self) -> crate::geometry2d::Vector<T> {
        crate::geometry2d::Vector::new(self.x, self.y)
    }

    /// 配列として取得
    pub fn to_array(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

impl<T: Scalar> Default for Vector<T> {
    fn default() -> Self {
        Self::zero()
    }
}

// geo_foundation Vector トレイトの実装
impl<T: Scalar> crate::traits::Vector<3> for Vector<T> {
    type Scalar = T;

    fn from_components(components: [Self::Scalar; 3]) -> Self {
        Self::new(components[0], components[1], components[2])
    }

    fn components(&self) -> [Self::Scalar; 3] {
        [self.x, self.y, self.z]
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
        let cross = self.cross(other);
        cross.length() < tolerance
    }

    fn is_perpendicular_to(&self, other: &Self, tolerance: Self::Scalar) -> bool {
        let dot = self.dot(other);
        dot.abs() < tolerance
    }
}

// geo_foundation Vector3D トレイトの実装（Vector<3> を前提とする）
impl<T: Scalar> crate::traits::Vector3D for Vector<T> {
    fn x(&self) -> Self::Scalar {
        self.x
    }

    fn y(&self) -> Self::Scalar {
        self.y
    }

    fn z(&self) -> Self::Scalar {
        self.z
    }

    fn new(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Self {
        Vector::new(x, y, z)
    }

    fn cross(&self, other: &Self) -> Self {
        Vector::cross(self, other)
    }

    fn unit_x() -> Self {
        Vector::unit_x()
    }

    fn unit_y() -> Self {
        Vector::unit_y()
    }

    fn unit_z() -> Self {
        Vector::unit_z()
    }
}

impl<T: Scalar> crate::traits::Vector3DExt for Vector<T> {
    fn rotate_around_axis(&self, axis: &Self, angle: Self::Scalar) -> Self {
        // ロドリゲスの回転公式を実装
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let axis = match axis.normalize() {
            Some(normalized) => normalized,
            None => return *self, // 軸がゼロベクトルの場合は回転しない
        };

        let dot_product = self.dot(&axis);
        let cross_product = axis.cross(self);

        *self * cos_angle + cross_product * sin_angle + axis * dot_product * (T::ONE - cos_angle)
    }

    fn any_perpendicular(&self) -> Self {
        if self.x.abs() > self.z.abs() {
            Self::new(-self.y, self.x, T::ZERO)
                .normalize()
                .unwrap_or(Self::unit_z())
        } else {
            Self::new(T::ZERO, -self.z, self.y)
                .normalize()
                .unwrap_or(Self::unit_x())
        }
    }

    fn build_orthonormal_basis(&self) -> (Self, Self, Self) {
        let normalized = match self.normalize() {
            Some(n) => n,
            None => return (Self::unit_x(), Self::unit_y(), Self::unit_z()),
        };

        let u = normalized.any_perpendicular();
        let v = normalized.cross(&u);

        (u, v, normalized)
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
            2 => &self.z,
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
            2 => &mut self.z,
            _ => panic!("Vector index out of bounds: {}", index),
        }
    }
}

// 算術演算の実装
impl<T: Scalar> std::ops::Add for Vector<T> {
    type Output = Vector<T>;

    fn add(self, other: Vector<T>) -> Self::Output {
        Vector::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Scalar> std::ops::Sub for Vector<T> {
    type Output = Vector<T>;

    fn sub(self, other: Vector<T>) -> Self::Output {
        Vector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Scalar> std::ops::Mul<T> for Vector<T> {
    type Output = Vector<T>;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl<T: Scalar> std::ops::Div<T> for Vector<T> {
    type Output = Vector<T>;

    fn div(self, scalar: T) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl<T: Scalar> std::ops::Neg for Vector<T> {
    type Output = Vector<T>;

    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y, -self.z)
    }
}

// Display実装は別クレートで実装

// テストコードはunit_tests/Vector_tests.rsに移動

// 型エイリアス（2Dと統一）
/// f64版の3D Vector（デフォルト）
pub type Vector3D = Vector<f64>;

/// f32版の3D Vector（高速演算用）
pub type Vector3Df = Vector<f32>;
