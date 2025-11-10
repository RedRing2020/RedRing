//! NURBS曲線・サーフェスのFoundation trait実装
//!
//! `RedRing` Foundation パターンに準拠した、NURBS曲線・サーフェスの
//! 基盤・拡張機能実装

use crate::curve_2d::WeightStorage as Curve2DWeightStorage;
use crate::curve_3d::WeightStorage as Curve3DWeightStorage;
use crate::surface::WeightStorage as SurfaceWeightStorage;
use crate::{NurbsCurve2D, NurbsCurve3D, NurbsSurface3D};
use analysis::Scalar;
use geo_foundation::{
    classification::PrimitiveKind, extension_foundation::ExtensionFoundation, BiParametricGeometry,
    NurbsCurve as NurbsCurveTrait, NurbsSurface as NurbsSurfaceTrait, ParametricGeometry,
    WeightedGeometry,
};
use geo_primitives::{Point2D, Point3D, Triangle3D, Vector2D, Vector3D};

// ================================
// NURBS曲線のFoundation実装
// ================================

impl<T: Scalar> ExtensionFoundation<T> for NurbsCurve2D<T> {
    type BBox = geo_primitives::BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::NurbsCurve
    }

    fn bounding_box(&self) -> Self::BBox {
        // 全制御点を包含する境界ボックスを計算（2D曲線だがBBox3Dを使用）
        let mut min_x = T::MAX;
        let mut max_x = T::MIN;
        let mut min_y = T::MAX;
        let mut max_y = T::MIN;

        for i in 0..self.num_points() {
            let point = self.control_point(i);
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
        }

        geo_primitives::BBox3D::new(
            Point3D::new(min_x, min_y, T::ZERO),
            Point3D::new(max_x, max_y, T::ZERO),
        )
    }

    fn measure(&self) -> Option<T> {
        // 曲線の長さを近似計算
        Some(self.approximate_length(100))
    }
}

impl<T: Scalar> NurbsCurveTrait<T> for NurbsCurve2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;

    fn degree(&self) -> usize {
        self.degree()
    }

    fn control_point_count(&self) -> usize {
        self.num_points()
    }

    fn parameter_domain(&self) -> (T, T) {
        self.parameter_domain()
    }

    fn evaluate_at(&self, parameter: T) -> Self::Point {
        self.evaluate_at(parameter)
    }

    fn derivative_at(&self, parameter: T) -> Self::Vector {
        self.derivative_at(parameter)
    }

    fn is_rational(&self) -> bool {
        matches!(self.weights(), Curve2DWeightStorage::Individual(_))
    }

    fn is_closed(&self, tolerance: T) -> bool {
        if self.num_points() < 2 {
            return false;
        }

        let first = self.control_point(0);
        let last = self.control_point(self.num_points() - 1);

        let distance = ((last.x() - first.x()).powi(2) + (last.y() - first.y()).powi(2)).sqrt();
        distance <= tolerance
    }

    fn approximate_length(&self, subdivisions: usize) -> T {
        if subdivisions == 0 {
            return T::ZERO;
        }

        let (t_min, t_max) = self.parameter_domain();
        let dt = (t_max - t_min) / T::from_usize(subdivisions);

        let mut total_length = T::ZERO;
        let mut prev_point = self.evaluate_at(t_min);

        for i in 1..=subdivisions {
            let t = t_min + dt * T::from_usize(i);
            let current_point = self.evaluate_at(t);

            let dx = current_point.x() - prev_point.x();
            let dy = current_point.y() - prev_point.y();
            let segment_length = (dx * dx + dy * dy).sqrt();

            total_length += segment_length;
            prev_point = current_point;
        }

        total_length
    }
}

impl<T: Scalar> WeightedGeometry<T> for NurbsCurve2D<T> {
    fn weight_at(&self, index: usize) -> T {
        self.weight(index)
    }

    fn set_weight_at(&mut self, _index: usize, _weight: T) -> std::result::Result<(), String> {
        // TODO: 重み設定機能の実装
        Err("重み設定機能は未実装です".to_string())
    }

    fn is_uniform_weight(&self) -> bool {
        matches!(self.weights(), Curve2DWeightStorage::Uniform)
    }

    fn make_non_rational(&mut self) {
        // TODO: 非有理変換機能の実装
    }

    fn make_rational(&mut self, _weights: Vec<T>) -> std::result::Result<(), String> {
        // TODO: 有理変換機能の実装
        Err("有理変換機能は未実装です".to_string())
    }
}

impl<T: Scalar> ParametricGeometry<T> for NurbsCurve2D<T> {
    fn normalize_parameter(&self, parameter: T) -> T {
        let (t_min, t_max) = self.parameter_domain();
        (parameter - t_min) / (t_max - t_min)
    }

    fn denormalize_parameter(&self, normalized_parameter: T) -> T {
        let (t_min, t_max) = self.parameter_domain();
        t_min + normalized_parameter * (t_max - t_min)
    }

    fn is_parameter_valid(&self, parameter: T) -> bool {
        let (t_min, t_max) = self.parameter_domain();
        parameter >= t_min && parameter <= t_max
    }

    fn clamp_parameter(&self, parameter: T) -> T {
        let (t_min, t_max) = self.parameter_domain();
        parameter.max(t_min).min(t_max)
    }
}

// ================================
// NURBSサーフェスのFoundation実装
// ================================

impl<T: Scalar> ExtensionFoundation<T> for NurbsSurface3D<T> {
    type BBox = geo_primitives::BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::NurbsSurface
    }

    fn bounding_box(&self) -> Self::BBox {
        // 全制御点を包含する境界ボックスを計算
        let (u_count, v_count) = self.grid_size();
        let mut min_x = T::MAX;
        let mut max_x = T::MIN;
        let mut min_y = T::MAX;
        let mut max_y = T::MIN;
        let mut min_z = T::MAX;
        let mut max_z = T::MIN;

        for u in 0..u_count {
            for v in 0..v_count {
                let point = self.control_point(u, v);
                min_x = min_x.min(point.x());
                max_x = max_x.max(point.x());
                min_y = min_y.min(point.y());
                max_y = max_y.max(point.y());
                min_z = min_z.min(point.z());
                max_z = max_z.max(point.z());
            }
        }

        geo_primitives::BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    fn measure(&self) -> Option<T> {
        // サーフェスの面積を近似計算
        Some(self.approximate_area(50, 50))
    }
}

impl<T: Scalar> NurbsSurfaceTrait<T> for NurbsSurface3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;

    fn u_degree(&self) -> usize {
        self.u_degree()
    }

    fn v_degree(&self) -> usize {
        self.v_degree()
    }

    fn grid_size(&self) -> (usize, usize) {
        self.grid_size()
    }

    fn parameter_domain(&self) -> ((T, T), (T, T)) {
        self.parameter_domain()
    }

    fn evaluate_at(&self, u: T, v: T) -> Self::Point {
        self.evaluate_at(u, v)
    }

    fn u_derivative_at(&self, u: T, v: T) -> Self::Vector {
        self.u_derivative_at(u, v)
    }

    fn v_derivative_at(&self, u: T, v: T) -> Self::Vector {
        self.v_derivative_at(u, v)
    }

    fn normal_at(&self, u: T, v: T) -> Self::Vector {
        self.normal_at(u, v)
    }

    fn is_rational(&self) -> bool {
        matches!(self.weights(), SurfaceWeightStorage::Individual(_))
    }

    fn is_u_closed(&self, _tolerance: T) -> bool {
        // TODO: u方向の閉曲面判定
        false
    }

    fn is_v_closed(&self, _tolerance: T) -> bool {
        // TODO: v方向の閉曲面判定
        false
    }

    fn approximate_area(&self, u_subdivisions: usize, v_subdivisions: usize) -> T {
        if u_subdivisions == 0 || v_subdivisions == 0 {
            return T::ZERO;
        }

        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(u_subdivisions);
        let dv = (v_max - v_min) / T::from_usize(v_subdivisions);

        let mut total_area = T::ZERO;

        for i in 0..u_subdivisions {
            for j in 0..v_subdivisions {
                let u1 = u_min + du * T::from_usize(i);
                let u2 = u_min + du * T::from_usize(i + 1);
                let v1 = v_min + dv * T::from_usize(j);
                let v2 = v_min + dv * T::from_usize(j + 1);

                // 4角形パッチの面積を近似計算
                let p00 = self.evaluate_at(u1, v1);
                let p10 = self.evaluate_at(u2, v1);
                let p01 = self.evaluate_at(u1, v2);
                let p11 = self.evaluate_at(u2, v2);

                // 三角形2つに分割して面積計算
                let area1 = if let Some(triangle) = Triangle3D::new(p00, p10, p01) {
                    triangle.area()
                } else {
                    T::ZERO
                };
                let area2 = if let Some(triangle) = Triangle3D::new(p10, p11, p01) {
                    triangle.area()
                } else {
                    T::ZERO
                };

                total_area = total_area + area1 + area2;
            }
        }

        total_area
    }
}

impl<T: Scalar> BiParametricGeometry<T> for NurbsSurface3D<T> {
    fn normalize_u_parameter(&self, u: T) -> T {
        let ((u_min, u_max), _) = self.parameter_domain();
        (u - u_min) / (u_max - u_min)
    }

    fn normalize_v_parameter(&self, v: T) -> T {
        let (_, (v_min, v_max)) = self.parameter_domain();
        (v - v_min) / (v_max - v_min)
    }

    fn denormalize_u_parameter(&self, normalized_u: T) -> T {
        let ((u_min, u_max), _) = self.parameter_domain();
        u_min + normalized_u * (u_max - u_min)
    }

    fn denormalize_v_parameter(&self, normalized_v: T) -> T {
        let (_, (v_min, v_max)) = self.parameter_domain();
        v_min + normalized_v * (v_max - v_min)
    }

    fn are_parameters_valid(&self, u: T, v: T) -> bool {
        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        u >= u_min && u <= u_max && v >= v_min && v <= v_max
    }

    fn clamp_parameters(&self, u: T, v: T) -> (T, T) {
        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        (u.max(u_min).min(u_max), v.max(v_min).min(v_max))
    }
}

// ================================
// NURBS 3D曲線のFoundation実装
// ================================

impl<T: Scalar> ExtensionFoundation<T> for NurbsCurve3D<T> {
    type BBox = geo_primitives::BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::NurbsCurve
    }

    fn bounding_box(&self) -> Self::BBox {
        // 全制御点を包含する境界ボックスを計算
        let mut min_x = T::MAX;
        let mut max_x = T::MIN;
        let mut min_y = T::MAX;
        let mut max_y = T::MIN;
        let mut min_z = T::MAX;
        let mut max_z = T::MIN;

        for i in 0..self.num_points() {
            let point = self.control_point(i);
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
            min_z = min_z.min(point.z());
            max_z = max_z.max(point.z());
        }

        geo_primitives::BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    fn measure(&self) -> Option<T> {
        // 曲線の長さを近似計算
        Some(self.approximate_length(100))
    }
}

impl<T: Scalar> NurbsCurveTrait<T> for NurbsCurve3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;

    fn degree(&self) -> usize {
        self.degree()
    }

    fn control_point_count(&self) -> usize {
        self.num_points()
    }

    fn parameter_domain(&self) -> (T, T) {
        self.parameter_domain()
    }

    fn evaluate_at(&self, parameter: T) -> Self::Point {
        self.evaluate_at(parameter)
    }

    fn derivative_at(&self, parameter: T) -> Self::Vector {
        self.derivative_at(parameter)
    }

    fn is_rational(&self) -> bool {
        matches!(self.weights(), Curve3DWeightStorage::Individual(_))
    }

    fn is_closed(&self, tolerance: T) -> bool {
        if self.num_points() < 2 {
            return false;
        }

        let first = self.control_point(0);
        let last = self.control_point(self.num_points() - 1);

        let dx = last.x() - first.x();
        let dy = last.y() - first.y();
        let dz = last.z() - first.z();
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        distance <= tolerance
    }

    fn approximate_length(&self, subdivisions: usize) -> T {
        self.approximate_length(subdivisions)
    }
}

impl<T: Scalar> WeightedGeometry<T> for NurbsCurve3D<T> {
    fn weight_at(&self, index: usize) -> T {
        self.weight(index)
    }

    fn set_weight_at(&mut self, _index: usize, _weight: T) -> std::result::Result<(), String> {
        // TODO: 重み設定機能の実装
        Err("重み設定機能は未実装です".to_string())
    }

    fn is_uniform_weight(&self) -> bool {
        matches!(self.weights(), Curve3DWeightStorage::Uniform)
    }

    fn make_non_rational(&mut self) {
        // TODO: 非有理変換機能の実装
    }

    fn make_rational(&mut self, _weights: Vec<T>) -> std::result::Result<(), String> {
        // TODO: 有理変換機能の実装
        Err("有理変換機能は未実装です".to_string())
    }
}

impl<T: Scalar> ParametricGeometry<T> for NurbsCurve3D<T> {
    fn normalize_parameter(&self, parameter: T) -> T {
        let (t_min, t_max) = self.parameter_domain();
        (parameter - t_min) / (t_max - t_min)
    }

    fn denormalize_parameter(&self, normalized_parameter: T) -> T {
        let (t_min, t_max) = self.parameter_domain();
        t_min + normalized_parameter * (t_max - t_min)
    }

    fn is_parameter_valid(&self, parameter: T) -> bool {
        let (t_min, t_max) = self.parameter_domain();
        parameter >= t_min && parameter <= t_max
    }

    fn clamp_parameter(&self, parameter: T) -> T {
        let (t_min, t_max) = self.parameter_domain();
        parameter.max(t_min).min(t_max)
    }
}
