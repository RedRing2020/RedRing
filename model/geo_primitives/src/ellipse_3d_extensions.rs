//! Ellipse3D Extension 機能
//!
//! Extension Foundation パターンに基づく Ellipse3D の拡張実装

use crate::{Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_contracts::default_distance_tolerance;
use geo_contracts::Scalar;

impl<T: Scalar> Ellipse3D<T> {
    /// XZ平面上の軸に平行な楕円を作成
    pub fn xz_aligned(center: Point3D<T>, semi_major_axis: T, semi_minor_axis: T) -> Option<Self> {
        Self::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_y(),
            Vector3D::unit_x(),
        )
    }

    /// YZ平面上の軸に平行な楕円を作成
    pub fn yz_aligned(center: Point3D<T>, semi_major_axis: T, semi_minor_axis: T) -> Option<Self> {
        Self::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_x(),
            Vector3D::unit_y(),
        )
    }

    /// 任意の平面上に楕円を作成
    pub fn on_plane(
        center: Point3D<T>,
        normal: Vector3D<T>,
        major_axis_direction: Vector3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
    ) -> Option<Self> {
        Self::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            normal,
            major_axis_direction,
        )
    }

    /// 楕円上の点での曲率を計算
    pub fn curvature_at_parameter(&self, t: T) -> T {
        let a = self.semi_major_internal();
        let b = self.semi_minor_internal();

        let cos_t = t.cos();
        let sin_t = t.sin();

        let numerator = a * b;
        let denominator = (a * a * sin_t * sin_t + b * b * cos_t * cos_t).powf(T::from_f64(1.5));

        if denominator.abs() > T::EPSILON {
            numerator / denominator
        } else {
            T::ZERO
        }
    }

    /// 楕円上の点での法線ベクトル（3D空間内）
    pub fn normal_at_parameter(&self, t: T) -> Vector3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円の接線ベクトル
        let tangent_local = Vector3D::new(
            -self.semi_major_internal() * sin_t,
            self.semi_minor_internal() * cos_t,
            T::ZERO,
        );

        // 局所座標系から世界座標系への変換
        let u_axis = self.major_axis_direction();
        let v_axis = self.minor_axis_direction();

        let tangent_world =
            u_axis.to_vector() * tangent_local.x() + v_axis.to_vector() * tangent_local.y();

        // 楕円平面内の法線（接線に直交）
        let tangent_dir =
            Direction3D::from_vector(tangent_world).unwrap_or(Direction3D::positive_x());
        self.normal().cross(&tangent_dir)
    }

    /// 楕円を平行移動
    pub fn translate(&self, offset: Vector3D<T>) -> Self {
        let new_center = Point3D::new(
            self.center_internal().x() + offset.x(),
            self.center_internal().y() + offset.y(),
            self.center_internal().z() + offset.z(),
        );

        Self::new(
            new_center,
            self.semi_major_internal(),
            self.semi_minor_internal(),
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .unwrap() // 既存の楕円から作成するので失敗しない
    }

    /// 楕円を拡大縮小（等方スケール）
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        Some(
            Self::new(
                self.center_internal(),
                self.semi_major_internal() * factor,
                self.semi_minor_internal() * factor,
                self.normal().as_vector(),
                self.major_axis_direction().as_vector(),
            )
            .unwrap(), // 既存の楕円から作成するので失敗しない
        )
    }

    /// 楕円を異方スケール
    pub fn scale_anisotropic(&self, major_scale: T, minor_scale: T) -> Option<Self> {
        if major_scale <= T::ZERO || minor_scale <= T::ZERO {
            return None;
        }

        let new_major = self.semi_major_internal() * major_scale;
        let new_minor = self.semi_minor_internal() * minor_scale;

        // 長軸と短軸の関係を保持
        if new_major >= new_minor {
            Some(
                Self::new(
                    self.center_internal(),
                    new_major,
                    new_minor,
                    self.normal().as_vector(),
                    self.major_axis_direction().as_vector(),
                )
                .unwrap(),
            )
        } else {
            // 軸が逆転した場合の調整
            Some(
                Self::new(
                    self.center_internal(),
                    new_minor,
                    new_major,
                    self.normal().as_vector(),
                    self.minor_axis_direction().as_vector(),
                )
                .unwrap(),
            )
        }
    }

    /// 楕円上の点のパラメータを逆算（近似）
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> Option<T> {
        // 点が楕円平面上にあるかチェック
        let to_point = Vector3D::new(
            point.x() - self.center_internal().x(),
            point.y() - self.center_internal().y(),
            point.z() - self.center_internal().z(),
        );

        let distance_to_plane = to_point.dot(&self.normal()).abs();
        if distance_to_plane > default_distance_tolerance::<T>() {
            return None; // 楕円平面上にない
        }

        // 楕円の局所座標系での座標を計算
        let u_coord = to_point.dot(&self.major_axis_direction());
        let v_coord = to_point.dot(&self.minor_axis_direction());

        // パラメータを計算
        let t = v_coord.atan2(u_coord);
        Some(if t < T::ZERO { t + T::TAU } else { t })
    }

    /// 点が楕円の内部にあるかを判定
    pub fn contains_point(&self, point: &Point3D<T>) -> bool {
        let to_point = Vector3D::new(
            point.x() - self.center_internal().x(),
            point.y() - self.center_internal().y(),
            point.z() - self.center_internal().z(),
        );

        // 楕円の局所座標系での座標
        let u_coord = to_point.dot(&self.major_axis_direction());
        let v_coord = to_point.dot(&self.minor_axis_direction());
        let w_coord = to_point.dot(&self.normal());

        // 楕円平面からの距離チェック
        if w_coord.abs() > default_distance_tolerance::<T>() {
            return false;
        }

        // 楕円の方程式: (u/a)² + (v/b)² <= 1
        let u_normalized = u_coord / self.semi_major_internal();
        let v_normalized = v_coord / self.semi_minor_internal();

        u_normalized * u_normalized + v_normalized * v_normalized <= T::ONE
    }
}
