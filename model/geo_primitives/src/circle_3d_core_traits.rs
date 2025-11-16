use crate::{Circle3D, Direction3D, Point3D};
use geo_foundation::Scalar;

// ============================================================================
// Circle3D Core Traits - STEP準拠設計
// ============================================================================

/// Circle3D生成のためのConstructorトレイト
pub trait Circle3DConstructor<T: Scalar> {
    fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;
    fn new_with_ref_direction(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self>
    where
        Self: Sized;
    fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;
    fn new_xz_plane(center: Point3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;
    fn new_yz_plane(center: Point3D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;
    fn unit_circle_xy() -> Self
    where
        Self: Sized;
    fn unit_circle_xz() -> Self
    where
        Self: Sized;
    fn unit_circle_yz() -> Self
    where
        Self: Sized;
}

/// Circle3D基本プロパティ取得トレイト
pub trait Circle3DProperties<T: Scalar> {
    fn center(&self) -> Point3D<T>;
    fn radius(&self) -> T;
    fn axis(&self) -> Direction3D<T>;
    fn ref_direction(&self) -> Direction3D<T>;
    fn normal(&self) -> Direction3D<T>; // 後方互換性
    fn diameter(&self) -> T;
    fn is_point(&self) -> bool;
    fn is_unit_circle(&self) -> bool;
    fn is_centered_at_origin(&self) -> bool;
    fn dimension(&self) -> u32;
}

/// Circle3D計量機能トレイト
pub trait Circle3DMeasure<T: Scalar> {
    fn circumference(&self) -> T;
    fn area(&self) -> T;
    fn contains_point(&self, point: Point3D<T>) -> bool;
    fn point_on_circumference(&self, point: Point3D<T>) -> bool;
    fn distance_to_point(&self, point: Point3D<T>) -> T;
    fn point_at_parameter(&self, t: T) -> Point3D<T>;
    fn parameter_at_point(&self, point: Point3D<T>) -> Option<T>;
    fn distance_to_circle(&self, other: &Self) -> T;
    fn closest_point_to(&self, point: Point3D<T>) -> Point3D<T>;
}

// ============================================================================
// 1. Constructor Trait Implementation
// ============================================================================

impl<T: Scalar> Circle3DConstructor<T> for Circle3D<T> {
    fn new(center: Point3D<T>, axis: Direction3D<T>, radius: T) -> Option<Self> {
        Circle3D::new(center, axis, radius)
    }

    fn new_with_ref_direction(
        center: Point3D<T>,
        axis: Direction3D<T>,
        ref_direction: Direction3D<T>,
        radius: T,
    ) -> Option<Self> {
        Circle3D::new_with_ref_direction(center, axis, ref_direction, radius)
    }

    fn new_xy_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Circle3D::new_xy_plane(center, radius)
    }

    fn new_xz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Circle3D::new_xz_plane(center, radius)
    }

    fn new_yz_plane(center: Point3D<T>, radius: T) -> Option<Self> {
        Circle3D::new_yz_plane(center, radius)
    }

    fn unit_circle_xy() -> Self {
        Circle3D::new_xy_plane(Point3D::new(T::ZERO, T::ZERO, T::ZERO), T::ONE)
            .expect("単位円の作成は必ず成功する")
    }

    fn unit_circle_xz() -> Self {
        Circle3D::new_xz_plane(Point3D::new(T::ZERO, T::ZERO, T::ZERO), T::ONE)
            .expect("単位円の作成は必ず成功する")
    }

    fn unit_circle_yz() -> Self {
        Circle3D::new_yz_plane(Point3D::new(T::ZERO, T::ZERO, T::ZERO), T::ONE)
            .expect("単位円の作成は必ず成功する")
    }
}

// ============================================================================
// 2. Properties Trait Implementation
// ============================================================================

impl<T: Scalar> Circle3DProperties<T> for Circle3D<T> {
    fn center(&self) -> Point3D<T> {
        self.center()
    }

    fn radius(&self) -> T {
        self.radius()
    }

    fn axis(&self) -> Direction3D<T> {
        self.axis()
    }

    fn ref_direction(&self) -> Direction3D<T> {
        self.ref_direction()
    }

    fn normal(&self) -> Direction3D<T> {
        self.normal()
    }

    fn diameter(&self) -> T {
        self.radius() + self.radius()
    }

    fn is_point(&self) -> bool {
        self.radius() <= T::EPSILON
    }

    fn is_unit_circle(&self) -> bool {
        (self.radius() - T::ONE).abs() <= T::EPSILON
    }

    fn is_centered_at_origin(&self) -> bool {
        self.center().x().abs() <= T::EPSILON
            && self.center().y().abs() <= T::EPSILON
            && self.center().z().abs() <= T::EPSILON
    }

    fn dimension(&self) -> u32 {
        3
    }
}

// ============================================================================
// 3. Measure Trait Implementation
// ============================================================================

impl<T: Scalar> Circle3DMeasure<T> for Circle3D<T> {
    fn circumference(&self) -> T {
        T::TAU * self.radius()
    }

    fn area(&self) -> T {
        T::PI * self.radius() * self.radius()
    }

    fn contains_point(&self, point: Point3D<T>) -> bool {
        // 点が円の平面上にあり、中心からの距離が半径以下かチェック
        let to_point = point - self.center();

        // 平面への投影（軸方向の成分を除去）
        let axis_component = to_point.dot(&self.axis().as_vector());
        if axis_component.abs() > T::EPSILON {
            return false; // 点が平面上にない
        }

        let projected = to_point - self.axis().as_vector() * axis_component;
        let distance_squared = projected.dot(&projected);
        distance_squared <= self.radius() * self.radius()
    }

    fn point_on_circumference(&self, point: Point3D<T>) -> bool {
        // 点が円の平面上にあり、中心からの距離が半径と等しいかチェック
        let to_point = point - self.center();

        // 平面への投影
        let axis_component = to_point.dot(&self.axis().as_vector());
        if axis_component.abs() > T::EPSILON {
            return false; // 点が平面上にない
        }

        let projected = to_point - self.axis().as_vector() * axis_component;
        let distance = projected.dot(&projected).sqrt();
        (distance - self.radius()).abs() <= T::EPSILON
    }

    fn distance_to_point(&self, point: Point3D<T>) -> T {
        let to_point = point - self.center();

        // 平面への投影
        let axis_component = to_point.dot(&self.axis().as_vector());
        let projected = to_point - self.axis().as_vector() * axis_component;
        let radial_distance = projected.dot(&projected).sqrt();

        // 平面からの距離と半径からの距離を組み合わせる
        let plane_distance = axis_component.abs();
        let circle_distance = (radial_distance - self.radius()).abs();

        (plane_distance * plane_distance + circle_distance * circle_distance).sqrt()
    }

    fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle = T::TAU * t;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Y軸方向を計算（axis × ref_direction）
        let y_direction = self
            .axis()
            .as_vector()
            .cross(&self.ref_direction().as_vector());

        // 円周上の点を計算
        let offset = self.ref_direction().as_vector() * (self.radius() * cos_angle)
            + y_direction * (self.radius() * sin_angle);

        self.center() + offset
    }

    fn parameter_at_point(&self, point: Point3D<T>) -> Option<T> {
        if !self.point_on_circumference(point) {
            return None;
        }

        let to_point = point - self.center();
        let axis_component = to_point.dot(&self.axis().as_vector());
        let projected = to_point - self.axis().as_vector() * axis_component;

        // ref_direction との角度を計算
        let x_component = projected.dot(&self.ref_direction().as_vector());
        let y_direction = self
            .axis()
            .as_vector()
            .cross(&self.ref_direction().as_vector());
        let y_component = projected.dot(&y_direction);

        let angle = y_component.atan2(x_component);
        let parameter = if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        } / T::TAU;

        Some(parameter)
    }

    fn distance_to_circle(&self, other: &Self) -> T {
        // 簡略化実装：中心間距離から半径を引く
        let center_distance = (self.center() - other.center()).length();
        let radii_sum = self.radius() + other.radius();

        if center_distance >= radii_sum {
            center_distance - radii_sum
        } else {
            T::ZERO // 交差または内包
        }
    }

    fn closest_point_to(&self, point: Point3D<T>) -> Point3D<T> {
        let to_point = point - self.center();

        // 平面への投影
        let axis_component = to_point.dot(&self.axis().as_vector());
        let projected = to_point - self.axis().as_vector() * axis_component;
        let radial_distance = projected.dot(&projected).sqrt();

        if radial_distance <= T::EPSILON {
            // 点が中心軸上にある場合、任意の円周上の点を返す
            self.center() + self.ref_direction().as_vector() * self.radius()
        } else {
            // 投影された方向に半径分移動
            let direction = projected.normalize();
            self.center() + direction * self.radius()
        }
    }
}
