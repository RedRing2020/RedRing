//! CylindricalSurface3D の拡張機能
//!
//! 境界操作、ISO曲線抽出、NURBS変換、メッシュ生成などの操作

use crate::{CylindricalSurface3D, Point3D, Vector3D};
use geo_contracts::Scalar;

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

    /// 指定された境界内の表面積を計算
    pub fn surface_area_region(&self, u_min: T, u_max: T, v_min: T, v_max: T) -> T {
        // 円柱サーフェスの面積: A = radius * (u_max - u_min) * (v_max - v_min)
        self.radius() * (u_max - u_min) * (v_max - v_min)
    }

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
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
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

        geo_core::Aabb3D::from_points(&points).unwrap_or_else(|| {
            let origin = geo_core::Point3D::new(T::ZERO, T::ZERO, T::ZERO);
            geo_core::Aabb3D::new(origin, origin)
        })
    }
}

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
