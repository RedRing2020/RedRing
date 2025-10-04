/// 3次元ベクトル実装
///
/// 座標値はmm単位で格納される

use crate::scalar::Scalar;
use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};
use crate::vector::Vector;
use std::fmt;
use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};

/// 3次元ベクトル
///
/// 座標値はmm単位で格納される
#[derive(Debug, Clone)]
pub struct Vector3D {
    components: [Scalar; 3],
    tolerance_context: ToleranceContext,
}

impl PartialEq for Vector3D {
    fn eq(&self, other: &Self) -> bool {
        self.components[0] == other.components[0] &&
        self.components[1] == other.components[1] &&
        self.components[2] == other.components[2]
    }
}

impl Vector3D {
    /// 成分から3Dベクトルを作成
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self {
            components: [x, y, z],
            tolerance_context: ToleranceContext::default(),
        }
    }

    /// f64から3Dベクトルを作成
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self::new(Scalar::new(x), Scalar::new(y), Scalar::new(z))
    }

    /// X成分を取得
    pub fn x(&self) -> Scalar { self.components[0] }

    /// Y成分を取得
    pub fn y(&self) -> Scalar { self.components[1] }

    /// Z成分を取得
    pub fn z(&self) -> Scalar { self.components[2] }

    /// X成分を設定
    pub fn set_x(&mut self, x: Scalar) { self.components[0] = x; }

    /// Y成分を設定
    pub fn set_y(&mut self, y: Scalar) { self.components[1] = y; }

    /// Z成分を設定
    pub fn set_z(&mut self, z: Scalar) { self.components[2] = z; }

    /// 3Dでの外積
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }

    /// スカラー三重積 a·(b×c)
    pub fn scalar_triple_product(&self, b: &Self, c: &Self) -> Scalar {
        self.dot(&b.cross(c))
    }

    /// ベクトル三重積 a×(b×c) = b(a·c) - c(a·b)
    pub fn vector_triple_product(&self, b: &Self, c: &Self) -> Self {
        b.clone() * self.dot(c) - c.clone() * self.dot(b)
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::from_f64(0.0, 0.0, 0.0)
    }

    /// X軸方向の単位ベクトル
    pub fn x_axis() -> Self {
        Self::from_f64(1.0, 0.0, 0.0)
    }

    /// Y軸方向の単位ベクトル
    pub fn y_axis() -> Self {
        Self::from_f64(0.0, 1.0, 0.0)
    }

    /// Z軸方向の単位ベクトル
    pub fn z_axis() -> Self {
        Self::from_f64(0.0, 0.0, 1.0)
    }

    /// デフォルト許容誤差での正規化
    ///
    /// 標準的な線形許容誤差（1e-6）を使用してベクトルを正規化する。
    /// より細かい制御が必要な場合は、`normalize(&context)`を使用する。
    pub fn normalize_default(&self) -> Option<Self> {
        use crate::tolerance::ToleranceContext;
        let default_context = ToleranceContext::standard();
        self.normalize(&default_context)
    }

    /// デフォルト許容誤差でのゼロベクトル判定
    pub fn is_zero_default(&self) -> bool {
        use crate::tolerance::ToleranceContext;
        let default_context = ToleranceContext::standard();
        self.is_zero(&default_context)
    }
}

/// 正規化された3次元方向ベクトル
#[derive(Debug, Clone, PartialEq)]
pub struct Direction3D {
    vector: Vector3D,
}

impl Direction3D {
    /// Vectorから安全にDirectionを作成
    pub fn from_vector(v: Vector3D, context: &ToleranceContext) -> Option<Self> {
        if let Some(normalized) = v.normalize(context) {
            Some(Self { vector: normalized })
        } else {
            None
        }
    }

    /// 成分から直接Direction作成（正規化チェック付き）
    pub fn new(x: f64, y: f64, z: f64, context: &ToleranceContext) -> Option<Self> {
        Self::from_vector(Vector3D::from_f64(x, y, z), context)
    }

    /// 内部ベクトルへの参照
    pub fn as_vector(&self) -> &Vector3D {
        &self.vector
    }

    /// ベクトルに変換
    pub fn to_vector(&self) -> Vector3D {
        self.vector.clone()
    }

    /// 成分アクセス
    pub fn x(&self) -> Scalar { self.vector.x() }
    pub fn y(&self) -> Scalar { self.vector.y() }
    pub fn z(&self) -> Scalar { self.vector.z() }

    /// 他の方向との内積
    pub fn dot(&self, other: &Self) -> Scalar {
        self.vector.dot(&other.vector)
    }

    /// 他の方向との外積（正規化済み）
    pub fn cross(&self, other: &Self, context: &ToleranceContext) -> Option<Self> {
        Self::from_vector(self.vector.cross(&other.vector), context)
    }

    /// 逆方向
    pub fn negate(&self) -> Self {
        Self { vector: -self.vector.clone() }
    }

    /// 直交基底の生成
    pub fn orthonormal_basis(&self, context: &ToleranceContext) -> (Vector3D, Vector3D) {
        // 適当な方向ベクトルを選択
        let up = if self.z().value().abs() < 0.99 {
            Vector3D::z_axis()
        } else {
            Vector3D::x_axis()
        };

        let u = self.vector.cross(&up).normalize(context).unwrap();
        let v = self.vector.cross(&u);
        (u, v)
    }
}

// Vector トレイト実装
impl Vector<3> for Vector3D {
    fn new(components: [Scalar; 3]) -> Self {
        Self {
            components,
            tolerance_context: ToleranceContext::default(),
        }
    }

    fn components(&self) -> &[Scalar; 3] {
        &self.components
    }

    fn components_mut(&mut self) -> &mut [Scalar; 3] {
        &mut self.components
    }

    fn dot(&self, other: &Self) -> Scalar {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z()
    }

    fn norm(&self) -> Scalar {
        self.norm_squared().sqrt()
    }

    fn normalize(&self, context: &ToleranceContext) -> Option<Self> {
        let length = self.norm();
        if length.value() < context.linear {
            None
        } else {
            Some(Self::new(self.x() / length, self.y() / length, self.z() / length))
        }
    }

    fn is_parallel_to(&self, other: &Self, context: &ToleranceContext) -> bool {
        let cross = self.cross(other);
        cross.is_zero(context)
    }

    fn component_min(&self, other: &Self) -> Self {
        Self::new(
            if self.x().value() < other.x().value() { self.x() } else { other.x() },
            if self.y().value() < other.y().value() { self.y() } else { other.y() },
            if self.z().value() < other.z().value() { self.z() } else { other.z() },
        )
    }

    fn component_max(&self, other: &Self) -> Self {
        Self::new(
            if self.x().value() > other.x().value() { self.x() } else { other.x() },
            if self.y().value() > other.y().value() { self.y() } else { other.y() },
            if self.z().value() > other.z().value() { self.z() } else { other.z() },
        )
    }

    fn abs(&self) -> Self {
        Self::new(self.x().abs(), self.y().abs(), self.z().abs())
    }
}

// ToleranceProvider実装
impl ToleranceProvider for Vector3D {
    fn tolerance_context(&self) -> &ToleranceContext {
        &self.tolerance_context
    }

    fn set_tolerance_context(&mut self, context: ToleranceContext) {
        self.tolerance_context = context;
    }
}

// TolerantEq実装
impl TolerantEq for Vector3D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x().tolerant_eq(&other.x(), context) &&
        self.y().tolerant_eq(&other.y(), context) &&
        self.z().tolerant_eq(&other.z(), context)
    }
}

// Index実装
impl Index<usize> for Vector3D {
    type Output = Scalar;

    fn index(&self, index: usize) -> &Self::Output {
        &self.components[index]
    }
}

impl IndexMut<usize> for Vector3D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}

// 算術演算実装
impl Add for Vector3D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x() + other.x(), self.y() + other.y(), self.z() + other.z())
    }
}

impl Sub for Vector3D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x() - other.x(), self.y() - other.y(), self.z() - other.z())
    }
}

impl Mul<Scalar> for Vector3D {
    type Output = Self;

    fn mul(self, scalar: Scalar) -> Self::Output {
        Self::new(self.x() * scalar, self.y() * scalar, self.z() * scalar)
    }
}

impl Neg for Vector3D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y(), -self.z())
    }
}

// Display実装
impl fmt::Display for Vector3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl fmt::Display for Direction3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Direction3D{}", self.vector)
    }
}
