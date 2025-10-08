/// geo_core 統合アダプター
///
/// このモジュールは、
/// geo_core の高精度実装を背景で使用するアダプター

use geo_foundation::{
    Scalar,
    ToleranceContext,
};

/// 2D Vector アダプター（model/geometry2d/vector.rs の代替）
#[derive(Debug, Clone, PartialEq)]
pub struct Vector2D {
    inner: GeoVector2D,
}

impl Vector2D {
    /// x成分とy成分からベクトルを構築
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            inner: GeoVector2D::new(Scalar::new(x), Scalar::new(y))
        }
    }

    /// 原点ベクトル
    pub fn zero() -> Self {
        Self { inner: GeoVector2D::zero() }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }

    /// ベクトルの長さ（ノルム）
    pub fn norm(&self) -> f64 {
        self.inner.magnitude().value()
    }

    /// 内積
    pub fn dot(&self, other: &Vector2D) -> f64 {
        self.inner.dot(&other.inner).value()
    }

    /// 外積（スカラー値）
    pub fn cross(&self, other: &Vector2D) -> f64 {
        // 2D外積は z成分のみ
        self.x() * other.y() - self.y() * other.x()
    }

    /// 方向ベクトルから角度（ラジアン）を取得
    pub fn angle(&self) -> f64 {
        self.y().atan2(self.x())
    }
}

impl std::ops::Add for Vector2D {
    type Output = Vector2D;
    fn add(self, rhs: Vector2D) -> Vector2D {
        Vector2D { inner: self.inner + rhs.inner }
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Vector2D;
    fn sub(self, rhs: Vector2D) -> Vector2D {
        Vector2D { inner: self.inner - rhs.inner }
    }
}

impl std::ops::Mul<f64> for Vector2D {
    type Output = Vector2D;
    fn mul(self, scalar: f64) -> Vector2D {
        Vector2D { inner: self.inner * Scalar::new(scalar) }
    }
}

impl std::ops::Neg for Vector2D {
    type Output = Vector2D;
    fn neg(self) -> Vector2D {
        Vector2D { inner: -self.inner }
    }
}

/// 3D Vector アダプター（model/geometry3d/vector.rs の代替）
#[derive(Debug, Clone, PartialEq)]
pub struct Vector3D {
    inner: GeoVector3D,
}

impl Vector3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoVector3D::new(Scalar::new(x), Scalar::new(y), Scalar::new(z))
        }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }
    pub fn z(&self) -> f64 { self.inner.z().value() }

    pub fn zero() -> Self {
        Self { inner: GeoVector3D::zero() }
    }

    pub fn norm(&self) -> f64 {
        self.inner.magnitude().value()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.inner.dot(&other.inner).value()
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self { inner: self.inner.cross(&other.inner) }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self { inner: self.inner * Scalar::new(factor) }
    }

    /// geo_core への内部アクセス（必要に応じて）
    pub fn as_geo_core(&self) -> &GeoVector3D {
        &self.inner
    }

    /// geo_core から変換
    pub fn from_geo_core(inner: GeoVector3D) -> Self {
        Self { inner }
    }
}

impl std::ops::Add for Vector3D {
    type Output = Vector3D;
    fn add(self, rhs: Vector3D) -> Vector3D {
        Vector3D { inner: self.inner + rhs.inner }
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Vector3D;
    fn sub(self, rhs: Vector3D) -> Vector3D {
        Vector3D { inner: self.inner - rhs.inner }
    }
}

impl std::ops::Mul<f64> for Vector3D {
    type Output = Vector3D;
    fn mul(self, scalar: f64) -> Vector3D {
        Vector3D { inner: self.inner * Scalar::new(scalar) }
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Vector3D;
    fn neg(self) -> Vector3D {
        Vector3D { inner: -self.inner }
    }
}

/// 2D Point アダプター
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    inner: GeoPoint2D,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            inner: GeoPoint2D::from_f64(x, y)
        }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }

    pub fn origin() -> Self {
        Self { inner: GeoPoint2D::origin() }
    }

    pub fn to_vector(&self) -> Vector2D {
        Vector2D { inner: self.inner.to_vector() }
    }

    pub fn distance_to(&self, other: &Point2D) -> f64 {
        self.inner.distance_to(&other.inner).value()
    }

    pub fn to_array(&self) -> [f64; 2] {
        [self.x(), self.y()]
    }

    pub fn from_array(arr: [f64; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }
}

/// 3D Point アダプター
#[derive(Debug, Clone, PartialEq)]
pub struct Point3D {
    inner: GeoPoint3D,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoPoint3D::from_f64(x, y, z)
        }
    }

    pub fn x(&self) -> f64 { self.inner.x().value() }
    pub fn y(&self) -> f64 { self.inner.y().value() }
    pub fn z(&self) -> f64 { self.inner.z().value() }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.inner.distance_to(&other.inner).value()
    }

    pub fn to_vector(&self) -> Vector3D {
        Vector3D { inner: self.inner.to_vector() }
    }

    /// geo-core への内部アクセス
    pub fn as_geo_core(&self) -> &GeoPoint3D {
        &self.inner
    }

    pub fn from_geo_core(inner: GeoPoint3D) -> Self {
        Self { inner }
    }
}

/// 3D Direction アダプター
#[derive(Debug, Clone, PartialEq)]
pub struct Direction3D {
    inner: Option<GeoDirection3D>,
}

impl Direction3D {
    pub fn new(x: f64, y: f64, z: f64) -> Option<Self> {
        let context = ToleranceContext::standard();
        let vector = GeoVector3D::new(Scalar::new(x), Scalar::new(y), Scalar::new(z));
        GeoDirection3D::from_vector(vector, &context).map(|d| Self { inner: Some(d) })
    }

    pub fn from_vector(v: Vector3D) -> Option<Self> {
        let context = ToleranceContext::standard();
        GeoDirection3D::from_vector(v.inner, &context).map(|d| Self { inner: Some(d) })
    }

    pub fn new_unchecked(x: f64, y: f64, z: f64) -> Self {
        // geo-coreには「unchecked」相当はないので、通常の作成を使用
        Self::new(x, y, z).unwrap_or_else(|| panic!("Cannot create direction from zero vector"))
    }

    pub fn length(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, |d| d.magnitude().value())
    }

    pub fn x(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, |d| d.x().value())
    }

    pub fn y(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, |d| d.y().value())
    }

    pub fn z(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, |d| d.z().value())
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> f64 {
        match (&self.inner, &other.inner) {
            (Some(a), Some(b)) => a.dot(b.as_vector()).value(),
            _ => 0.0,
        }
    }
}

/// アダプター間の便利な変換関数
impl From<Point3D> for Vector3D {
    fn from(point: Point3D) -> Self {
        Vector3D { inner: point.inner.to_vector() }
    }
}

impl std::ops::Add<Vector3D> for Point3D {
    type Output = Point3D;
    fn add(self, rhs: Vector3D) -> Point3D {
        Point3D::new(
            self.x() + rhs.x(),
            self.y() + rhs.y(),
            self.z() + rhs.z(),
        )
    }
}

impl std::ops::Sub for Point3D {
    type Output = Vector3D;
    fn sub(self, rhs: Point3D) -> Vector3D {
        Vector3D::new(
            self.x() - rhs.x(),
            self.y() - rhs.y(),
            self.z() - rhs.z(),
        )
    }
}

// Normalize と Normed トレイトの実装（互換性のため）
pub trait Normalize {
    fn normalize(&self) -> Self;
}

pub trait Normed {
    fn norm(&self) -> f64;
}

impl Normalize for Vector3D {
    fn normalize(&self) -> Self {
        let n = self.norm();
        if n == 0.0 {
            *self
        } else {
            *self * (1.0 / n)
        }
    }
}

impl Normed for Vector3D {
    fn norm(&self) -> f64 {
        self.norm()
    }
}

impl Normalize for Vector2D {
    fn normalize(&self) -> Self {
        let n = self.norm();
        if n == 0.0 {
            *self
        } else {
            *self * (1.0 / n)
        }
    }
}

impl Normed for Vector2D {
    fn norm(&self) -> f64 {
        self.norm()
    }
}
