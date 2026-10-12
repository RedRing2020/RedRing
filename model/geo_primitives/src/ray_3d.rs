//! Ray3D - 3次元半無限直線のCore実装
//!
//! Ray3D は起点から一方向に無限に延びる半無限直線を表現します。
//! パラメータ t は 0 ≤ t < ∞ の範囲で定義されます。
//! Core Traits実装（Constructor, Properties, Measure）も含む

mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, InfiniteLine3D, Point3D, Vector3D};
use geo_contracts::{
    AngleBetween, CrossDistance, ParallelRelation, Ray3DConstructor, Ray3DContainment,
    Ray3DDistance, Ray3DEvaluation, Ray3DProjection, Ray3DProperties, Ray3DTransform, Scalar,
};

/// 3次元半無限直線
///
/// 起点から指定方向に無限に延びる半無限直線を表現します。
/// パラメータ表現: point = origin + t * direction (t ≥ 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D<T: Scalar> {
    /// 起点（t=0での点）
    origin: Point3D<T>,
    /// 方向（単位ベクトル）
    direction: Direction3D<T>,
}

impl<T: Scalar> Ray3D<T> {
    /// 起点と方向ベクトルから Ray3D を作成
    ///
    /// # 引数
    /// * `origin` - 起点
    /// * `direction` - 方向ベクトル（自動的に正規化される）
    ///
    /// # 戻り値
    /// 方向ベクトルがゼロベクトルの場合は None を返す
    pub fn new(origin: Point3D<T>, direction: Vector3D<T>) -> Option<Self> {
        let direction = Direction3D::from_vector(direction)?;
        Some(Self { origin, direction })
    }

    /// 2点を通る Ray3D を作成
    ///
    /// # 引数
    /// * `start` - 起点
    /// * `through` - 通過する点
    ///
    /// # 戻り値
    /// 2点が同じ場合は None を返す
    pub fn from_points(start: Point3D<T>, through: Point3D<T>) -> Option<Self> {
        let direction = through - start;
        Self::new(start, direction)
    }

    /// 起点を取得
    pub fn origin(&self) -> Point3D<T> {
        self.origin
    }

    /// 内部方向ベクトルを取得（Vector3D型）
    pub fn direction_vector(&self) -> Vector3D<T> {
        self.direction.as_vector()
    }

    /// この Ray が乗る無限直線を返す
    pub fn to_line(&self) -> InfiniteLine3D<T> {
        InfiniteLine3D::from_direction(self.origin, self.direction)
    }

    /// パラメータ t での点を計算
    ///
    /// # 引数
    /// * `t` - パラメータ（t ≥ 0）
    ///
    /// # 戻り値
    /// Ray上の点
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let direction_offset = self.direction * t;
        Point3D::new(
            self.origin.x() + direction_offset.x(),
            self.origin.y() + direction_offset.y(),
            self.origin.z() + direction_offset.z(),
        )
    }

    /// 点がRay上にあるかを判定
    ///
    /// # 引数
    /// * `point` - 判定する点
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// Ray までの距離が `tolerance` 以内なら true（起点の後方も起点までの距離で判定する）
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 指定された点に対するパラメータを計算
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let to_point = *point - self.origin;
        self.direction.as_vector().dot(&to_point)
    }

    /// Ray の逆方向を作成
    pub fn reverse(&self) -> Self {
        Self {
            origin: self.origin,
            direction: -self.direction,
        }
    }

    /// 他の Ray と平行かを判定
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        let cross = self.direction.cross(&other.direction);
        cross.length() < geo_contracts::default_distance_tolerance::<T>()
    }

    /// 他の Ray との角度を返す
    pub fn angle_between(&self, other: &Self) -> T {
        let dot = self.direction.dot(&other.direction);
        let clamped = if dot > T::ONE {
            T::ONE
        } else if dot < -T::ONE {
            -T::ONE
        } else {
            dot
        };
        clamped.acos()
    }
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for Ray3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::Ray
    }
}

impl<T: Scalar> geo_contracts::TolerantEq<T> for Ray3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        let origin_distance = self.origin.distance_to(&other.origin);
        let direction_dot = self.direction.dot(&other.direction).abs();
        let direction_similar = direction_dot >= T::from_f64(0.999);

        origin_distance <= tolerance && direction_similar
    }
}

/// Ray3DConstructor トレイト実装
impl<T: Scalar> Ray3DConstructor<T> for Ray3D<T> {
    fn new(origin: (T, T, T), direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized,
    {
        let direction_vector = Vector3D::new(direction.0, direction.1, direction.2);
        let origin_point = Point3D::new(origin.0, origin.1, origin.2);
        Ray3D::new(origin_point, direction_vector)
    }

    fn from_points(start: (T, T, T), through: (T, T, T)) -> Option<Self>
    where
        Self: Sized,
    {
        let start_point = Point3D::new(start.0, start.1, start.2);
        let through_point = Point3D::new(through.0, through.1, through.2);
        Ray3D::from_points(start_point, through_point)
    }
}

/// Ray3DProperties トレイト実装
impl<T: Scalar> Ray3DProperties<T> for Ray3D<T> {
    fn origin(&self) -> (T, T, T) {
        (self.origin.x(), self.origin.y(), self.origin.z())
    }

    fn direction(&self) -> (T, T, T) {
        (self.direction.x(), self.direction.y(), self.direction.z())
    }

    fn is_valid(&self) -> bool {
        // Ray3D::new がSomeを返した時点で有効性は保証されている
        true
    }
}

impl<T: Scalar> Ray3DEvaluation<T> for Ray3D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let point = self.point_at_parameter(t);
        (point.x(), point.y(), point.z())
    }

    fn parameter_for_point(&self, point: (T, T, T)) -> T {
        let target_point = Point3D::new(point.0, point.1, point.2);
        self.parameter_for_point(&target_point)
    }
}

impl<T: Scalar> Ray3DProjection<T> for Ray3D<T> {
    fn closest_point(&self, point: (T, T, T)) -> (T, T, T) {
        let target_point = Point3D::new(point.0, point.1, point.2);
        let t = self.parameter_for_point(&target_point);
        let clamped_t = if t < T::ZERO { T::ZERO } else { t };
        let closest = self.point_at_parameter(clamped_t);
        (closest.x(), closest.y(), closest.z())
    }
}

impl<T: Scalar> Ray3DDistance<T> for Ray3D<T> {
    fn distance_to_point(&self, point: (T, T, T)) -> T {
        let target_point = Point3D::new(point.0, point.1, point.2);
        let to_point = target_point - self.origin;
        let projection_length = self.direction.as_vector().dot(&to_point);

        if projection_length <= T::ZERO {
            self.origin.distance_to(&target_point)
        } else {
            let direction_offset = self.direction * projection_length;
            let projection = Point3D::new(
                self.origin.x() + direction_offset.x(),
                self.origin.y() + direction_offset.y(),
                self.origin.z() + direction_offset.z(),
            );
            target_point.distance_to(&projection)
        }
    }
}

impl<T: Scalar> Ray3DContainment<T> for Ray3D<T> {
    fn contains_point(&self, point: (T, T, T)) -> bool {
        let target_point = Point3D::new(point.0, point.1, point.2);
        use geo_contracts::default_distance_tolerance;
        self.contains_point(&target_point, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> Ray3DTransform<T> for Ray3D<T> {
    fn reverse(&self) -> Self
    where
        Self: Sized,
    {
        Ray3D::reverse(self)
    }
}

impl<T: Scalar> CrossDistance<T, Self> for Ray3D<T> {
    fn distance_to(&self, other: &Self) -> T {
        let w = self.origin - other.origin;
        let a = self.direction.dot(&self.direction);
        let b = self.direction.dot(&other.direction);
        let c = other.direction.dot(&other.direction);
        let d = self.direction.as_vector().dot(&w);
        let e = other.direction.as_vector().dot(&w);

        let denom = a * c - b * b;
        use geo_contracts::default_kernel_numerical_zero_tolerance;
        if denom.abs() < default_kernel_numerical_zero_tolerance::<T>() {
            let other_origin = other.origin;
            return self.distance_to_point(&Point3D::new(
                other_origin.x(),
                other_origin.y(),
                other_origin.z(),
            ));
        }

        let sc = (b * e - c * d) / denom;
        let tc = (a * e - b * d) / denom;

        let sc_clamped = if sc < T::ZERO { T::ZERO } else { sc };
        let tc_clamped = if tc < T::ZERO { T::ZERO } else { tc };

        let p1 = self.point_at_parameter(sc_clamped);
        let p2 = other.point_at_parameter(tc_clamped);

        let diff = Vector3D::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        diff.length()
    }
}

impl<T: Scalar> ParallelRelation<Self> for Ray3D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        Ray3D::is_parallel_to(self, other)
    }
}

impl<T: Scalar> AngleBetween<T, Self> for Ray3D<T> {
    fn angle_between(&self, other: &Self) -> T {
        Ray3D::angle_between(self, other)
    }
}
