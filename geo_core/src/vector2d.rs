/// 2次元ベクトル実装（f64ベース、modelトレイト互換）/// 2次元ベクトル実装（f64ベース、modelトレイト互換）/// 2次元ベクトル実装（f64ベース、modelトレイト互換）/// 2次元ベクトル実装（f64ベース、modelトレイト互換）/// 2次元ベクトル実装（f64ベース）



use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};

use std::fmt;

use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};



/// 2次元ベクトル（f64ベース）use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]

pub struct Vector2D {use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};/// ///

    x: f64,

    y: f64,

}

/// 2次元ベクトル（f64ベース）use std::fmt;

impl Vector2D {

    /// f64成分から2Dベクトルを作成#[derive(Debug, Clone, Copy, PartialEq)]

    pub fn new(x: f64, y: f64) -> Self {

        Self { x, y }pub struct Vector2D {use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};/// geo_primitivesから移植されたf64ベース実装にmodelトレイトサポートを追加/// geo_primitivesから移植されたf64ベース実装

    }

    x: f64,

    /// 互換性のためのエイリアス

    pub fn from_f64(x: f64, y: f64) -> Self {    y: f64,

        Self::new(x, y)

    }}



    /// X成分を取得/// 2次元ベクトル（f64ベース）

    pub fn x(&self) -> f64 { self.x }

impl Vector2D {

    /// Y成分を取得

    pub fn y(&self) -> f64 { self.y }    /// f64成分から2Dベクトルを作成#[derive(Debug, Clone, Copy, PartialEq)]



    /// X成分を設定    pub fn new(x: f64, y: f64) -> Self {

    pub fn set_x(&mut self, x: f64) { self.x = x; }

        Self { x, y }pub struct Vector2D {use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};use crate::tolerance::{ToleranceContext, TolerantEq, ToleranceProvider};

    /// Y成分を設定

    pub fn set_y(&mut self, y: f64) { self.y = y; }    }



    /// 2D外積（スカラー値）を計算    x: f64,

    pub fn cross_2d(&self, other: &Self) -> f64 {

        self.x * other.y - self.y * other.x    /// 互換性のためのエイリアス

    }

    pub fn from_f64(x: f64, y: f64) -> Self {    y: f64,use std::fmt;use crate::vector::Vector;

    /// 指定角度だけ回転

    pub fn rotate(&self, angle: f64) -> Self {        Self::new(x, y)

        let cos_a = angle.cos();

        let sin_a = angle.sin();    }}



        Self::new(

            self.x * cos_a - self.y * sin_a,

            self.x * sin_a + self.y * cos_a,    /// X成分を取得use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};use std::fmt;

        )

    }    pub fn x(&self) -> f64 { self.x }



    /// 垂直ベクトル（反時計回りに90度回転）impl Vector2D {

    pub fn perpendicular(&self) -> Self {

        Self::new(-self.y, self.x)    /// Y成分を取得

    }

    pub fn y(&self) -> f64 { self.y }    /// f64成分から2Dベクトルを作成use std::ops::{Add, Sub, Mul, Neg, Index, IndexMut};

    /// ベクトルの長さ

    pub fn length(&self) -> f64 {

        (self.x * self.x + self.y * self.y).sqrt()

    }    /// X成分を設定    pub fn new(x: f64, y: f64) -> Self {



    /// ベクトルの長さの二乗    pub fn set_x(&mut self, x: f64) { self.x = x; }

    pub fn length_squared(&self) -> f64 {

        self.x * self.x + self.y * self.y        Self { x, y }/// 2次元ベクトル（f64ベース）

    }

    /// Y成分を設定

    /// 正規化（modelトレイト互換 - ゼロベクトルはゼロベクトルを返す）

    pub fn normalize(&self) -> Self {    pub fn set_y(&mut self, y: f64) { self.y = y; }    }

        let len = self.length();

        if len == 0.0 {

            Self::zero()

        } else {    /// 2D外積（スカラー値）を計算/// /// 2次元ベクトル（f64ベース）

            Self::new(self.x / len, self.y / len)

        }    pub fn cross_2d(&self, other: &Self) -> f64 {

    }

        self.x * other.y - self.y * other.x    /// 互換性のためのエイリアス

    /// 正規化（Optionで安全版）

    pub fn try_normalize(&self) -> Option<Self> {    }

        let len = self.length();

        if len == 0.0 {    pub fn from_f64(x: f64, y: f64) -> Self {/// 座標値はmm単位で格納される///

            None

        } else {    /// 指定角度だけ回転

            Some(Self::new(self.x / len, self.y / len))

        }    pub fn rotate(&self, angle: f64) -> Self {        Self::new(x, y)

    }

        let cos_a = angle.cos();

    /// ゼロベクトル

    pub fn zero() -> Self {        let sin_a = angle.sin();    }#[derive(Debug, Clone, Copy, PartialEq)]/// 座標値はmm単位で格納される

        Self::new(0.0, 0.0)

    }



    /// X軸単位ベクトル        Self::new(

    pub fn x_axis() -> Self {

        Self::new(1.0, 0.0)            self.x * cos_a - self.y * sin_a,

    }

            self.x * sin_a + self.y * cos_a,    /// X成分を取得pub struct Vector2D {#[derive(Debug, Clone, Copy, PartialEq)]

    /// Y軸単位ベクトル

    pub fn y_axis() -> Self {        )

        Self::new(0.0, 1.0)

    }    }    pub fn x(&self) -> f64 { self.x }



    /// 内積

    pub fn dot(&self, other: &Self) -> f64 {

        self.x * other.x + self.y * other.y    /// 垂直ベクトル（反時計回りに90度回転）    x: f64,pub struct Vector2D {

    }

    pub fn perpendicular(&self) -> Self {

    /// ノルム（長さ）- modelトレイト互換

    pub fn norm(&self) -> f64 {        Self::new(-self.y, self.x)    /// Y成分を取得

        self.length()

    }    }



    /// スカラー加算（スケール加算）- modelトレイト互換    pub fn y(&self) -> f64 { self.y }    y: f64,    x: f64,

    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {

        Self::new(    /// ベクトルの長さ

            self.x + other.x * scale,

            self.y + other.y * scale,    pub fn length(&self) -> f64 {

        )

    }        (self.x * self.x + self.y * self.y).sqrt()

}

    }    /// X成分を設定}    y: f64,

// 許容誤差ベース比較

impl TolerantEq for Vector2D {

    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {

        let diff = *self - *other;    /// ベクトルの長さの二乗    pub fn set_x(&mut self, x: f64) { self.x = x; }

        diff.length() <= context.linear_tolerance()

    }    pub fn length_squared(&self) -> f64 {

}

        self.x * self.x + self.y * self.y}

impl ToleranceProvider for Vector2D {

    fn tolerance_context(&self) -> &ToleranceContext {    }

        &ToleranceContext::default()

    }    /// Y成分を設定



    fn set_tolerance_context(&mut self, _context: ToleranceContext) {    /// 正規化（modelトレイト互換 - ゼロベクトルはゼロベクトルを返す）

        // f64ベースの実装では許容誤差は内部に保持しない

    }    pub fn normalize(&self) -> Self {    pub fn set_y(&mut self, y: f64) { self.y = y; }impl Vector2D {

}

        let len = self.length();

// 演算子実装

impl Add for Vector2D {        if len == 0.0 {

    type Output = Self;

            Self::zero()

    fn add(self, other: Self) -> Self::Output {

        Self::new(self.x + other.x, self.y + other.y)        } else {    /// 2D外積（スカラー値）を計算    /// f64成分から2Dベクトルを作成impl Vector2D {

    }

}            Self::new(self.x / len, self.y / len)



impl Sub for Vector2D {        }    pub fn cross_2d(&self, other: &Self) -> f64 {

    type Output = Self;

    }

    fn sub(self, other: Self) -> Self::Output {

        Self::new(self.x - other.x, self.y - other.y)        self.x * other.y - self.y * other.x    pub fn new(x: f64, y: f64) -> Self {    /// 成分から2Dベクトルを作成

    }

}    /// 正規化（Optionで安全版）



impl Mul<f64> for Vector2D {    pub fn try_normalize(&self) -> Option<Self> {    }

    type Output = Self;

        let len = self.length();

    fn mul(self, scalar: f64) -> Self::Output {

        Self::new(self.x * scalar, self.y * scalar)        if len == 0.0 {        Self { x, y }    pub fn new(x: Scalar, y: Scalar) -> Self {

    }

}            None



impl Neg for Vector2D {        } else {    /// 指定角度だけ回転

    type Output = Self;

            Some(Self::new(self.x / len, self.y / len))

    fn neg(self) -> Self::Output {

        Self::new(-self.x, -self.y)        }    pub fn rotate(&self, angle: f64) -> Self {    }        Self {

    }

}    }



// 添字アクセス        let cos_a = angle.cos();

impl Index<usize> for Vector2D {

    type Output = f64;    /// ゼロベクトル



    fn index(&self, index: usize) -> &Self::Output {    pub fn zero() -> Self {        let sin_a = angle.sin();            components: [x, y],

        match index {

            0 => &self.x,        Self::new(0.0, 0.0)

            1 => &self.y,

            _ => panic!("Index out of bounds for Vector2D"),    }

        }

    }

}

    /// X軸単位ベクトル        Self::new(    /// 互換性のためのエイリアス            tolerance_context: ToleranceContext::default(),

impl IndexMut<usize> for Vector2D {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {    pub fn x_axis() -> Self {

        match index {

            0 => &mut self.x,        Self::new(1.0, 0.0)            self.x * cos_a - self.y * sin_a,

            1 => &mut self.y,

            _ => panic!("Index out of bounds for Vector2D"),    }

        }

    }            self.x * sin_a + self.y * cos_a,    pub fn from_f64(x: f64, y: f64) -> Self {        }

}

    /// Y軸単位ベクトル

// 表示

impl fmt::Display for Vector2D {    pub fn y_axis() -> Self {        )

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "Vector2D({:.6}, {:.6})", self.x, self.y)        Self::new(0.0, 1.0)

    }

}    }    }        Self::new(x, y)    }



    /// 内積

    pub fn dot(&self, other: &Self) -> f64 {

        self.x * other.x + self.y * other.y    /// 垂直ベクトル（反時計回りに90度回転）    }

    }

    pub fn perpendicular(&self) -> Self {

    /// ノルム（長さ）- modelトレイト互換

    pub fn norm(&self) -> f64 {        Self::new(-self.y, self.x)    /// f64から2Dベクトルを作成

        self.length()

    }    }



    /// スカラー加算（スケール加算）- modelトレイト互換    /// X成分を取得    pub fn from_f64(x: f64, y: f64) -> Self {

    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {

        Self::new(    /// ベクトルの長さ

            self.x + other.x * scale,

            self.y + other.y * scale,    pub fn length(&self) -> f64 {    pub fn x(&self) -> f64 { self.x }        Self::new(Scalar::new(x), Scalar::new(y))

        )

    }        (self.x * self.x + self.y * self.y).sqrt()

}

    }    }

// 許容誤差ベース比較

impl TolerantEq for Vector2D {

    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {

        let diff = *self - *other;    /// ベクトルの長さの二乗    /// Y成分を取得

        diff.length() <= context.linear_tolerance()

    }    pub fn length_squared(&self) -> f64 {

}

        self.x * self.x + self.y * self.y    pub fn y(&self) -> f64 { self.y }    /// X成分を取得

impl ToleranceProvider for Vector2D {

    fn tolerance_context(&self) -> &ToleranceContext {    }

        &ToleranceContext::default()

    }    pub fn x(&self) -> Scalar { self.components[0] }

}

    /// 正規化（modelトレイト互換 - ゼロベクトルはゼロベクトルを返す）

// 演算子実装

impl Add for Vector2D {    pub fn normalize(&self) -> Self {    /// X成分を設定

    type Output = Self;

        let len = self.length();

    fn add(self, other: Self) -> Self::Output {

        Self::new(self.x + other.x, self.y + other.y)        if len == 0.0 {    pub fn set_x(&mut self, x: f64) { self.x = x; }    /// Y成分を取得

    }

}            Self::zero()



impl Sub for Vector2D {        } else {    pub fn y(&self) -> Scalar { self.components[1] }

    type Output = Self;

            Self::new(self.x / len, self.y / len)

    fn sub(self, other: Self) -> Self::Output {

        Self::new(self.x - other.x, self.y - other.y)        }    /// Y成分を設定

    }

}    }



impl Mul<f64> for Vector2D {    pub fn set_y(&mut self, y: f64) { self.y = y; }    /// X成分を設定

    type Output = Self;

    /// 正規化（Optionで安全版）

    fn mul(self, scalar: f64) -> Self::Output {

        Self::new(self.x * scalar, self.y * scalar)    pub fn try_normalize(&self) -> Option<Self> {    pub fn set_x(&mut self, x: Scalar) { self.components[0] = x; }

    }

}        let len = self.length();



impl Neg for Vector2D {        if len == 0.0 {    /// 2D外積（スカラー値）を計算

    type Output = Self;

            None

    fn neg(self) -> Self::Output {

        Self::new(-self.x, -self.y)        } else {    pub fn cross_2d(&self, other: &Self) -> f64 {    /// Y成分を設定

    }

}            Some(Self::new(self.x / len, self.y / len))



// 添字アクセス        }        self.x * other.y - self.y * other.x    pub fn set_y(&mut self, y: Scalar) { self.components[1] = y; }

impl Index<usize> for Vector2D {

    type Output = f64;    }



    fn index(&self, index: usize) -> &Self::Output {    }

        match index {

            0 => &self.x,    /// ゼロベクトル

            1 => &self.y,

            _ => panic!("Index out of bounds for Vector2D"),    pub fn zero() -> Self {    /// 2Dでの外積（スカラー値）

        }

    }        Self::new(0.0, 0.0)

}

    }    /// 指定角度だけ回転    pub fn cross_2d(&self, other: &Self) -> Scalar {

impl IndexMut<usize> for Vector2D {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {

        match index {

            0 => &mut self.x,    /// X軸単位ベクトル    pub fn rotate(&self, angle: f64) -> Self {        self.x() * other.y() - self.y() * other.x()

            1 => &mut self.y,

            _ => panic!("Index out of bounds for Vector2D"),    pub fn x_axis() -> Self {

        }

    }        Self::new(1.0, 0.0)        let cos_a = angle.cos();    }

}

    }

// 表示

impl fmt::Display for Vector2D {        let sin_a = angle.sin();

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "Vector2D({:.6}, {:.6})", self.x, self.y)    /// Y軸単位ベクトル

    }

}    pub fn y_axis() -> Self {            /// 垂直ベクトル（反時計回りに90度回転）

        Self::new(0.0, 1.0)

    }        Self::new(    pub fn perpendicular(&self) -> Self {



    /// 内積            self.x * cos_a - self.y * sin_a,        Self::new(-self.y(), self.x())

    pub fn dot(&self, other: &Self) -> f64 {

        self.x * other.x + self.y * other.y            self.x * sin_a + self.y * cos_a,    }

    }

        )

    /// ノルム（長さ）- modelトレイト互換

    pub fn norm(&self) -> f64 {    }    /// 回転変換

        self.length()

    }    pub fn rotate(&self, angle: Scalar) -> Self {



    /// スカラー加算（スケール加算）- modelトレイト互換    /// 垂直ベクトル（反時計回りに90度回転）        let cos_a = angle.cos();

    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {

        Self::new(    pub fn perpendicular(&self) -> Self {        let sin_a = angle.sin();

            self.x + other.x * scale,

            self.y + other.y * scale,        Self::new(-self.y, self.x)        Self::new(

        )

    }    }            self.x() * cos_a - self.y() * sin_a,

}

            self.x() * sin_a + self.y() * cos_a,

// 許容誤差ベース比較

impl TolerantEq for Vector2D {    /// ベクトルの長さ        )

    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {

        let diff = *self - *other;    pub fn length(&self) -> f64 {    }

        diff.length() <= context.linear_tolerance()

    }        (self.x * self.x + self.y * self.y).sqrt()

}

    }    /// ゼロベクトル

impl ToleranceProvider for Vector2D {

    fn tolerance_context(&self) -> &ToleranceContext {    pub fn zero() -> Self {

        &ToleranceContext::default()

    }    /// ベクトルの長さの二乗        Self::from_f64(0.0, 0.0)

}

    pub fn length_squared(&self) -> f64 {    }

// 演算子実装

impl Add for Vector2D {        self.x * self.x + self.y * self.y

    type Output = Self;

    }    /// X軸方向の単位ベクトル

    fn add(self, other: Self) -> Self::Output {

        Self::new(self.x + other.x, self.y + other.y)    pub fn x_axis() -> Self {

    }

}    /// 正規化（modelトレイト互換 - ゼロベクトルはゼロベクトルを返す）        Self::from_f64(1.0, 0.0)



impl Sub for Vector2D {    pub fn normalize(&self) -> Self {    }

    type Output = Self;

        let len = self.length();

    fn sub(self, other: Self) -> Self::Output {

        Self::new(self.x - other.x, self.y - other.y)        if len == 0.0 {    /// Y軸方向の単位ベクトル

    }

}            Self::zero()    pub fn y_axis() -> Self {



impl Mul<f64> for Vector2D {        } else {        Self::from_f64(0.0, 1.0)

    type Output = Self;

            Self::new(self.x / len, self.y / len)    }

    fn mul(self, scalar: f64) -> Self::Output {

        Self::new(self.x * scalar, self.y * scalar)        }}

    }

}    }



impl Neg for Vector2D {// Vector トレイト実装

    type Output = Self;

    /// 正規化（Optionで安全版）impl Vector<2> for Vector2D {

    fn neg(self) -> Self::Output {

        Self::new(-self.x, -self.y)    pub fn try_normalize(&self) -> Option<Self> {    fn new(components: [Scalar; 2]) -> Self {

    }

}        let len = self.length();        Self {



// 添字アクセス        if len == 0.0 {            components,

impl Index<usize> for Vector2D {

    type Output = f64;            None            tolerance_context: ToleranceContext::default(),



    fn index(&self, index: usize) -> &Self::Output {        } else {        }

        match index {

            0 => &self.x,            Some(Self::new(self.x / len, self.y / len))    }

            1 => &self.y,

            _ => panic!("Index out of bounds for Vector2D"),        }

        }

    }    }    fn components(&self) -> &[Scalar; 2] {

}

        &self.components

impl IndexMut<usize> for Vector2D {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {    /// ゼロベクトル    }

        match index {

            0 => &mut self.x,    pub fn zero() -> Self {

            1 => &mut self.y,

            _ => panic!("Index out of bounds for Vector2D"),        Self::new(0.0, 0.0)    fn components_mut(&mut self) -> &mut [Scalar; 2] {

        }

    }    }        &mut self.components

}

    }

// 表示

impl fmt::Display for Vector2D {    /// X軸単位ベクトル

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "Vector2D({:.6}, {:.6})", self.x, self.y)    pub fn x_axis() -> Self {    fn dot(&self, other: &Self) -> Scalar {

    }

}        Self::new(1.0, 0.0)        self.x() * other.x() + self.y() * other.y()

    }    }



    /// Y軸単位ベクトル      fn norm(&self) -> Scalar {

    pub fn y_axis() -> Self {        self.norm_squared().sqrt()

        Self::new(0.0, 1.0)    }

    }

    fn normalize(&self, context: &ToleranceContext) -> Option<Self> {

    /// 内積        let length = self.norm();

    pub fn dot(&self, other: &Self) -> f64 {        if length.value() < context.linear {

        self.x * other.x + self.y * other.y            None

    }        } else {

            Some(Self::new(self.x() / length, self.y() / length))

    /// ノルム（長さ）- modelトレイト互換        }

    pub fn norm(&self) -> f64 {    }

        self.length()

    }    fn is_parallel_to(&self, other: &Self, context: &ToleranceContext) -> bool {

        let cross = self.cross_2d(other);

    /// スカラー加算（スケール加算）- modelトレイト互換        cross.tolerant_eq(&Scalar::new(0.0), context)

    pub fn add_scaled(&self, other: &Self, scale: f64) -> Self {    }

        Self::new(

            self.x + other.x * scale,    fn component_min(&self, other: &Self) -> Self {

            self.y + other.y * scale,        Self::new(

        )            if self.x().value() < other.x().value() { self.x() } else { other.x() },

    }            if self.y().value() < other.y().value() { self.y() } else { other.y() },

}        )

    }

// 許容誤差ベース比較

impl TolerantEq for Vector2D {    fn component_max(&self, other: &Self) -> Self {

    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {        Self::new(

        let diff = *self - *other;            if self.x().value() > other.x().value() { self.x() } else { other.x() },

        diff.length() <= context.linear_tolerance()            if self.y().value() > other.y().value() { self.y() } else { other.y() },

    }        )

}    }



impl ToleranceProvider for Vector2D {    fn abs(&self) -> Self {

    fn tolerance_context(&self) -> &ToleranceContext {        Self::new(self.x().abs(), self.y().abs())

        &ToleranceContext::default()    }

    }}

}

// ToleranceProvider実装

// 演算子実装impl ToleranceProvider for Vector2D {

impl Add for Vector2D {    fn tolerance_context(&self) -> &ToleranceContext {

    type Output = Self;        &self.tolerance_context

    }

    fn add(self, other: Self) -> Self::Output {

        Self::new(self.x + other.x, self.y + other.y)    fn set_tolerance_context(&mut self, context: ToleranceContext) {

    }        self.tolerance_context = context;

}    }

}

impl Sub for Vector2D {

    type Output = Self;// TolerantEq実装

impl TolerantEq for Vector2D {

    fn sub(self, other: Self) -> Self::Output {    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {

        Self::new(self.x - other.x, self.y - other.y)        self.x().tolerant_eq(&other.x(), context) &&

    }        self.y().tolerant_eq(&other.y(), context)

}    }

}

impl Mul<f64> for Vector2D {

    type Output = Self;// Index実装

impl Index<usize> for Vector2D {

    fn mul(self, scalar: f64) -> Self::Output {    type Output = Scalar;

        Self::new(self.x * scalar, self.y * scalar)

    }    fn index(&self, index: usize) -> &Self::Output {

}        &self.components[index]

    }

impl Neg for Vector2D {}

    type Output = Self;

impl IndexMut<usize> for Vector2D {

    fn neg(self) -> Self::Output {    fn index_mut(&mut self, index: usize) -> &mut Self::Output {

        Self::new(-self.x, -self.y)        &mut self.components[index]

    }    }

}}



// 添字アクセス// 算術演算実装

impl Index<usize> for Vector2D {impl Add for Vector2D {

    type Output = f64;    type Output = Self;



    fn index(&self, index: usize) -> &Self::Output {    fn add(self, other: Self) -> Self::Output {

        match index {        Self::new(self.x() + other.x(), self.y() + other.y())

            0 => &self.x,    }

            1 => &self.y,}

            _ => panic!("Index out of bounds for Vector2D"),

        }impl Sub for Vector2D {

    }    type Output = Self;

}

    fn sub(self, other: Self) -> Self::Output {

impl IndexMut<usize> for Vector2D {        Self::new(self.x() - other.x(), self.y() - other.y())

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {    }

        match index {}

            0 => &mut self.x,

            1 => &mut self.y,impl Mul<Scalar> for Vector2D {

            _ => panic!("Index out of bounds for Vector2D"),    type Output = Self;

        }

    }    fn mul(self, scalar: Scalar) -> Self::Output {

}        Self::new(self.x() * scalar, self.y() * scalar)

    }

// 表示}

impl fmt::Display for Vector2D {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {impl Neg for Vector2D {

        write!(f, "Vector2D({:.6}, {:.6})", self.x, self.y)    type Output = Self;

    }

}    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y())
    }
}

// Display実装
impl fmt::Display for Vector2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}
