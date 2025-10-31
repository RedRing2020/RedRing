//! CylindricalSurface3D の拡張機能
//!
//! 境界操作、ISO曲線抽出、NURBS変換、メッシュ生成などの高度な操作
//! サーフェス特有の解析機能とCAD/CAM用途の実装

use crate::{CylindricalSurface3D, Point3D, Vector3D};
use geo_foundation::Scalar;

/// 境界制約された円柱サーフェス領域
#[derive(Debug, Clone, PartialEq)]
pub struct BoundedCylindricalSurface3D<T: Scalar> {
    /// 基本円柱サーフェス
    pub surface: CylindricalSurface3D<T>,
    /// U方向の境界 [u_min, u_max] （角度範囲）
    pub u_bounds: (T, T),
    /// V方向の境界 [v_min, v_max] （軸方向範囲）
    pub v_bounds: (T, T),
}

/// ISO曲線（1つのパラメータを固定した曲線）
#[derive(Debug, Clone, PartialEq)]
pub struct IsoCurve3D<T: Scalar> {
    /// 基本円柱サーフェス
    pub surface: CylindricalSurface3D<T>,
    /// 固定パラメータの種類
    pub iso_type: IsoType<T>,
}

/// ISO曲線の種類
#[derive(Debug, Clone, PartialEq)]
pub enum IsoType<T: Scalar> {
    /// U固定（v方向の直線）
    ConstantU(T),
    /// V固定（u方向の円）
    ConstantV(T),
}

/// CylindricalSurface3D の拡張実装
impl<T: Scalar> CylindricalSurface3D<T> {
    // ========================================================================
    // 境界操作
    // ========================================================================

    /// 境界制約を適用してBoundedCylindricalSurface3Dを作成
    pub fn with_bounds(
        &self,
        u_min: T,
        u_max: T,
        v_min: T,
        v_max: T,
    ) -> BoundedCylindricalSurface3D<T> {
        BoundedCylindricalSurface3D {
            surface: self.clone(),
            u_bounds: (u_min, u_max),
            v_bounds: (v_min, v_max),
        }
    }

    /// 円周の一部をトリム（角度範囲で指定）
    pub fn trim_angular(
        &self,
        start_angle: T,
        end_angle: T,
        v_min: T,
        v_max: T,
    ) -> BoundedCylindricalSurface3D<T> {
        self.with_bounds(start_angle, end_angle, v_min, v_max)
    }

    /// 軸方向の一部をトリム
    pub fn trim_axial(&self, v_min: T, v_max: T) -> BoundedCylindricalSurface3D<T> {
        self.with_bounds(T::ZERO, T::PI * T::from_f64(2.0), v_min, v_max)
    }

    // ========================================================================
    // ISO曲線抽出
    // ========================================================================

    /// V固定のISO曲線（円）を取得
    pub fn iso_curve_u(&self, v: T) -> IsoCurve3D<T> {
        IsoCurve3D {
            surface: self.clone(),
            iso_type: IsoType::ConstantV(v),
        }
    }

    /// U固定のISO曲線（軸方向直線）を取得
    pub fn iso_curve_v(&self, u: T) -> IsoCurve3D<T> {
        IsoCurve3D {
            surface: self.clone(),
            iso_type: IsoType::ConstantU(u),
        }
    }

    // ========================================================================
    // サーフェス解析
    // ========================================================================

    /// 指定された境界内の表面積を計算
    pub fn surface_area_region(&self, u_min: T, u_max: T, v_min: T, v_max: T) -> T {
        // 円柱サーフェスの面積: A = radius * (u_max - u_min) * (v_max - v_min)
        self.radius() * (u_max - u_min) * (v_max - v_min)
    }

    /// 完全な円周の表面積（単位軸長あたり）
    pub fn surface_area_per_unit_length(&self) -> T {
        T::from_f64(2.0) * T::PI * self.radius()
    }

    /// 指定範囲でのサーフェス品質評価（曲率偏差）
    pub fn surface_quality_metric(&self, _u_min: T, _u_max: T, _v_min: T, _v_max: T) -> T {
        // 円柱サーフェスは理想的な2次曲面なので品質は常に最高
        T::ZERO
    }

    // ========================================================================
    // メッシュ生成（サーフェス用）
    // ========================================================================

    /// パラメトリックメッシュを生成
    pub fn to_parametric_mesh(
        &self,
        u_segments: usize,
        v_segments: usize,
        v_min: T,
        v_max: T,
    ) -> Vec<Vec<Point3D<T>>> {
        let u_segments = u_segments.max(3);
        let v_segments = v_segments.max(1);

        let mut mesh = Vec::new();

        for i in 0..=v_segments {
            let v = v_min + (v_max - v_min) * T::from_f64(i as f64 / v_segments as f64);
            let mut row = Vec::new();

            for j in 0..=u_segments {
                let u = T::from_f64(2.0 * std::f64::consts::PI * j as f64 / u_segments as f64);
                let point = self.point_at_uv(u, v);
                row.push(point);
            }

            mesh.push(row);
        }

        mesh
    }

    /// 四角形パッチのメッシュを生成（CAD/CAM用）
    pub fn to_quad_patches(
        &self,
        u_divisions: usize,
        v_divisions: usize,
        v_min: T,
        v_max: T,
    ) -> Vec<QuadPatch<T>> {
        let mut patches = Vec::new();

        let du = T::from_f64(2.0 * std::f64::consts::PI / u_divisions as f64);
        let dv = (v_max - v_min) / T::from_f64(v_divisions as f64);

        for i in 0..u_divisions {
            for j in 0..v_divisions {
                let u1 = T::from_f64(i as f64) * du;
                let u2 = T::from_f64((i + 1) as f64) * du;
                let v1 = v_min + T::from_f64(j as f64) * dv;
                let v2 = v_min + T::from_f64((j + 1) as f64) * dv;

                let p00 = self.point_at_uv(u1, v1);
                let p10 = self.point_at_uv(u2, v1);
                let p01 = self.point_at_uv(u1, v2);
                let p11 = self.point_at_uv(u2, v2);

                patches.push(QuadPatch {
                    corners: [p00, p10, p11, p01],
                    u_range: (u1, u2),
                    v_range: (v1, v2),
                });
            }
        }

        patches
    }

    // ========================================================================
    // NURBS変換（準備）
    // ========================================================================

    /// NURBS表現への変換準備（制御点の計算）
    pub fn nurbs_control_points(&self, _u_degree: usize, v_degree: usize) -> Vec<Vec<Point3D<T>>> {
        // 円柱サーフェスの正確なNURBS表現
        // 円周方向: 4つの制御点で正確な円を表現
        // 軸方向: 2つの制御点で直線を表現

        let mut control_points = Vec::new();

        // V方向（軸方向）の制御点数
        let v_points = v_degree + 1;

        for _i in 0..v_points {
            let mut row = Vec::new();

            // U方向（円周方向）の制御点（4点で正確な円）
            let r = self.radius();
            let x_axis = self.ref_direction().as_vector();
            let y_axis = self.y_axis().as_vector();

            // 円の制御点（重み付きで正確な円を表現）
            let center = self.center();
            row.push(Point3D::new(
                center.x() + x_axis.x() * r,
                center.y() + x_axis.y() * r,
                center.z() + x_axis.z() * r,
            )); // 0度
            let y_axis_norm = (x_axis + y_axis).normalize();
            row.push(Point3D::new(
                center.x() + y_axis_norm.x() * r,
                center.y() + y_axis_norm.y() * r,
                center.z() + y_axis_norm.z() * r,
            )); // 45度
            row.push(Point3D::new(
                center.x() + y_axis.x() * r,
                center.y() + y_axis.y() * r,
                center.z() + y_axis.z() * r,
            )); // 90度
            let y_axis_norm_2 = (-x_axis + y_axis).normalize();
            row.push(Point3D::new(
                center.x() + y_axis_norm_2.x() * r,
                center.y() + y_axis_norm_2.y() * r,
                center.z() + y_axis_norm_2.z() * r,
            )); // 135度
            row.push(Point3D::new(
                center.x() - x_axis.x() * r,
                center.y() - x_axis.y() * r,
                center.z() - x_axis.z() * r,
            )); // 180度
            let y_axis_norm_3 = (-x_axis - y_axis).normalize();
            row.push(Point3D::new(
                center.x() + y_axis_norm_3.x() * r,
                center.y() + y_axis_norm_3.y() * r,
                center.z() + y_axis_norm_3.z() * r,
            )); // 225度
            row.push(Point3D::new(
                center.x() - y_axis.x() * r,
                center.y() - y_axis.y() * r,
                center.z() - y_axis.z() * r,
            )); // 270度
            let y_axis_norm_4 = (x_axis - y_axis).normalize();
            row.push(Point3D::new(
                center.x() + y_axis_norm_4.x() * r,
                center.y() + y_axis_norm_4.y() * r,
                center.z() + y_axis_norm_4.z() * r,
            )); // 315度
            row.push(Point3D::new(
                center.x() + x_axis.x() * r,
                center.y() + x_axis.y() * r,
                center.z() + x_axis.z() * r,
            )); // 360度（閉曲線）

            control_points.push(row);
        }

        control_points
    }

    /// NURBS重みを計算（円の正確な表現用）
    pub fn nurbs_weights(&self) -> Vec<Vec<T>> {
        // 円の正確なNURBS表現に必要な重み
        let sqrt2_inv = T::ONE / T::from_f64(std::f64::consts::SQRT_2);
        let weights_row = vec![
            T::ONE,    // 0度
            sqrt2_inv, // 45度
            T::ONE,    // 90度
            sqrt2_inv, // 135度
            T::ONE,    // 180度
            sqrt2_inv, // 225度
            T::ONE,    // 270度
            sqrt2_inv, // 315度
            T::ONE,    // 360度
        ];

        // V方向は常に重み1
        vec![weights_row.clone(), weights_row]
    }
}

/// 四角形パッチ
#[derive(Debug, Clone, PartialEq)]
pub struct QuadPatch<T: Scalar> {
    /// 4つの角の座標 [p00, p10, p11, p01]
    pub corners: [Point3D<T>; 4],
    /// Uパラメータ範囲
    pub u_range: (T, T),
    /// Vパラメータ範囲
    pub v_range: (T, T),
}

// ============================================================================
// BoundedCylindricalSurface3D の実装
// ============================================================================

impl<T: Scalar> BoundedCylindricalSurface3D<T> {
    /// 境界内の点かを判定
    pub fn contains_uv(&self, u: T, v: T) -> bool {
        u >= self.u_bounds.0 && u <= self.u_bounds.1 && v >= self.v_bounds.0 && v <= self.v_bounds.1
    }

    /// 境界制約された表面積を計算
    pub fn bounded_surface_area(&self) -> T {
        let du = self.u_bounds.1 - self.u_bounds.0;
        let dv = self.v_bounds.1 - self.v_bounds.0;
        self.surface.radius() * du * dv
    }

    /// 境界ボックスを計算
    pub fn bounding_box(&self) -> crate::BBox3D<T> {
        // 境界を考慮した正確な境界ボックス計算
        let mut points = vec![
            self.surface.point_at_uv(self.u_bounds.0, self.v_bounds.0),
            self.surface.point_at_uv(self.u_bounds.1, self.v_bounds.0),
            self.surface.point_at_uv(self.u_bounds.0, self.v_bounds.1),
            self.surface.point_at_uv(self.u_bounds.1, self.v_bounds.1),
        ];

        // U方向の極値点も考慮（0, π/2, π, 3π/2）
        let critical_u_values = [
            T::ZERO,
            T::PI / T::from_f64(2.0),
            T::PI,
            T::PI * T::from_f64(3.0) / T::from_f64(2.0),
        ];

        for &u in &critical_u_values {
            if u >= self.u_bounds.0 && u <= self.u_bounds.1 {
                points.push(self.surface.point_at_uv(u, self.v_bounds.0));
                points.push(self.surface.point_at_uv(u, self.v_bounds.1));
            }
        }

        crate::BBox3D::from_points(&points)
            .unwrap_or_else(|| crate::BBox3D::new(Point3D::origin(), Point3D::origin()))
    }
}

// ============================================================================
// IsoCurve3D の実装
// ============================================================================

impl<T: Scalar> IsoCurve3D<T> {
    /// パラメータtでの曲線上の点を取得
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        match &self.iso_type {
            IsoType::ConstantU(u) => {
                // V方向の直線
                self.surface.point_at_uv(*u, t)
            }
            IsoType::ConstantV(v) => {
                // U方向の円
                self.surface.point_at_uv(t, *v)
            }
        }
    }

    /// 曲線の接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: T) -> Vector3D<T> {
        match &self.iso_type {
            IsoType::ConstantU(u) => {
                // V方向の接線
                self.surface.tangent_v_at_uv(*u, t)
            }
            IsoType::ConstantV(v) => {
                // U方向の接線
                self.surface.tangent_u_at_uv(t, *v)
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;
    use approx::assert_relative_eq;

    fn create_test_surface() -> CylindricalSurface3D<f64> {
        CylindricalSurface3D::new_z_axis(Point3D::origin(), 5.0).unwrap()
    }

    #[test]
    fn test_bounded_surface() {
        let surface = create_test_surface();
        let bounded = surface.with_bounds(0.0, std::f64::consts::PI, 0.0, 10.0);

        assert!(bounded.contains_uv(1.0, 5.0));
        assert!(!bounded.contains_uv(4.0, 5.0)); // U範囲外
        assert!(!bounded.contains_uv(1.0, 15.0)); // V範囲外

        let area = bounded.bounded_surface_area();
        assert_relative_eq!(area, 5.0 * std::f64::consts::PI * 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_iso_curves() {
        let surface = create_test_surface();

        // V固定の円
        let circle = surface.iso_curve_u(5.0);
        let point = circle.point_at_parameter(0.0);
        assert_relative_eq!(point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 5.0, epsilon = 1e-10);

        // U固定の直線
        let line = surface.iso_curve_v(0.0);
        let point = line.point_at_parameter(10.0);
        assert_relative_eq!(point.x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(point.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(point.z(), 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_surface_area_region() {
        let surface = create_test_surface();
        let area = surface.surface_area_region(0.0, std::f64::consts::PI, 0.0, 10.0);

        // 半円柱の表面積: radius * π * height
        assert_relative_eq!(area, 5.0 * std::f64::consts::PI * 10.0, epsilon = 1e-10);
    }

    #[test]
    fn test_parametric_mesh() {
        let surface = create_test_surface();
        let mesh = surface.to_parametric_mesh(4, 2, 0.0, 10.0);

        assert_eq!(mesh.len(), 3); // v_segments + 1
        assert_eq!(mesh[0].len(), 5); // u_segments + 1

        // 最初の点は (radius, 0, 0)
        assert_relative_eq!(mesh[0][0].x(), 5.0, epsilon = 1e-10);
        assert_relative_eq!(mesh[0][0].y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(mesh[0][0].z(), 0.0, epsilon = 1e-10);
    }
}
