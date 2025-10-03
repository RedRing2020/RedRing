/// 2次元ベクトル実装
/// 
/// 座標値はmm単位で格納される

use crate::scalar::Scalar;
use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};
use crate::vector::Vector;
use std::fmt;
use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};

/// 2次元ベクトル
/// 
/// 座標値はmm単位で格納される
#[derive(Debug, Clone)]
pub struct Vector2D {
    components: [Scalar; 2],
    tolerance_context: ToleranceContext,
}

impl PartialEq for Vector2D {
    fn eq(&self, other: &Self) -> bool {
        self.components[0] == other.components[0] && 
        self.components[1] == other.components[1]
    }
}

impl Vector2D {
    /// 成分から2Dベクトルを作成
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self {
            components: [x, y],
            tolerance_context: ToleranceContext::default(),
        }
    }

    /// f64から2Dベクトルを作成
    pub fn from_f64(x: f64, y: f64) -> Self {
        Self::new(Scalar::new(x), Scalar::new(y))
    }

    /// X成分を取得
    pub fn x(&self) -> Scalar { self.components[0] }

    /// Y成分を取得
    pub fn y(&self) -> Scalar { self.components[1] }

    /// X成分を設定
    pub fn set_x(&mut self, x: Scalar) { self.components[0] = x; }

    /// Y成分を設定
    pub fn set_y(&mut self, y: Scalar) { self.components[1] = y; }

    /// 2Dでの外積（スカラー値）
    pub fn cross_2d(&self, other: &Self) -> Scalar {
        self.x() * other.y() - self.y() * other.x()
    }

    /// 垂直ベクトル（反時計回りに90度回転）
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y(), self.x())
    }

    /// 回転変換
    pub fn rotate(&self, angle: Scalar) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            self.x() * cos_a - self.y() * sin_a,
            self.x() * sin_a + self.y() * cos_a,
        )
    }

    /// ゼロベクトル
    pub fn zero() -> Self {
        Self::from_f64(0.0, 0.0)
    }

    /// X軸方向の単位ベクトル
    pub fn x_axis() -> Self {
        Self::from_f64(1.0, 0.0)
    }

    /// Y軸方向の単位ベクトル
    pub fn y_axis() -> Self {
        Self::from_f64(0.0, 1.0)
    }
}

// Vector トレイト実装
impl Vector<2> for Vector2D {
    fn new(components: [Scalar; 2]) -> Self {
        Self {
            components,
            tolerance_context: ToleranceContext::default(),
        }
    }

    fn components(&self) -> &[Scalar; 2] {
        &self.components
    }

    fn components_mut(&mut self) -> &mut [Scalar; 2] {
        &mut self.components
    }

    fn dot(&self, other: &Self) -> Scalar {
        self.x() * other.x() + self.y() * other.y()
    }

    fn norm(&self) -> Scalar {
        self.norm_squared().sqrt()
    }

    fn normalize(&self, context: &ToleranceContext) -> Option<Self> {
        let length = self.norm();
        if length.value() < context.linear {
            None
        } else {
            Some(Self::new(self.x() / length, self.y() / length))
        }
    }

    fn is_parallel_to(&self, other: &Self, context: &ToleranceContext) -> bool {
        let cross = self.cross_2d(other);
        cross.tolerant_eq(&Scalar::new(0.0), context)
    }

    fn component_min(&self, other: &Self) -> Self {
        Self::new(
            if self.x().value() < other.x().value() { self.x() } else { other.x() },
            if self.y().value() < other.y().value() { self.y() } else { other.y() },
        )
    }

    fn component_max(&self, other: &Self) -> Self {
        Self::new(
            if self.x().value() > other.x().value() { self.x() } else { other.x() },
            if self.y().value() > other.y().value() { self.y() } else { other.y() },
        )
    }

    fn abs(&self) -> Self {
        Self::new(self.x().abs(), self.y().abs())
    }
}

// ToleranceProvider実装
impl ToleranceProvider for Vector2D {
    fn tolerance_context(&self) -> &ToleranceContext {
        &self.tolerance_context
    }

    fn set_tolerance_context(&mut self, context: ToleranceContext) {
        self.tolerance_context = context;
    }
}

// TolerantEq実装
impl TolerantEq for Vector2D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x().tolerant_eq(&other.x(), context) &&
        self.y().tolerant_eq(&other.y(), context)
    }
}

// Index実装
impl Index<usize> for Vector2D {
    type Output = Scalar;

    fn index(&self, index: usize) -> &Self::Output {
        &self.components[index]
    }
}

impl IndexMut<usize> for Vector2D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}

// 算術演算実装
impl Add for Vector2D {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x() + other.x(), self.y() + other.y())
    }
}

impl Sub for Vector2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x() - other.x(), self.y() - other.y())
    }
}

impl Mul<Scalar> for Vector2D {
    type Output = Self;

    fn mul(self, scalar: Scalar) -> Self::Output {
        Self::new(self.x() * scalar, self.y() * scalar)
    }
}

impl Neg for Vector2D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y())
    }
}

// Display実装
impl fmt::Display for Vector2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}