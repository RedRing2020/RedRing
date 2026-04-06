//! `NurbsSurface3D` の Bounds 実装
//!
//! `Bounded` trait と関連テストを提供する。

use crate::NurbsSurface3D;
use crate::Scalar;
use geo_contracts::Bounded;

impl<T: Scalar> Bounded<T> for NurbsSurface3D<T> {
    type Aabb = geo_core::Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 制御点ベースの境界ボックスを計算
        let mut min_x = T::from_f64(f64::INFINITY);
        let mut min_y = T::from_f64(f64::INFINITY);
        let mut min_z = T::from_f64(f64::INFINITY);
        let mut max_x = T::from_f64(f64::NEG_INFINITY);
        let mut max_y = T::from_f64(f64::NEG_INFINITY);
        let mut max_z = T::from_f64(f64::NEG_INFINITY);

        for u in 0..self.grid_size().0 {
            for v in 0..self.grid_size().1 {
                let point = self.control_point(u, v);
                min_x = min_x.min(point.x());
                min_y = min_y.min(point.y());
                min_z = min_z.min(point.z());
                max_x = max_x.max(point.x());
                max_y = max_y.max(point.y());
                max_z = max_z.max(point.z());
            }
        }

        Some(geo_core::Aabb3D::new(
            geo_core::Point3D::new(min_x, min_y, min_z),
            geo_core::Point3D::new(max_x, max_y, max_z),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::{
        NurbsSurface3DConstructor, NurbsSurface3DDerived, NurbsSurface3DProperties,
    };
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_constructor_new() {
        // 2x2 制御点グリッド
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 1.0)],
        ];

        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let result = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            Some(weights),
            u_knots,
            v_knots,
            1, // u_degree
            1, // v_degree
        );

        assert!(result.is_ok());
        let surface = result.unwrap();

        // Properties検証
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_degree(&surface),
            1
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_degree(&surface),
            1
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_count(&surface),
            2
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_count(&surface),
            2
        );
        assert!(<NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::is_rational(&surface));
    }

    #[test]
    fn test_constructor_from_control_points() {
        // クランプドノットベクトルを自動生成
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 2.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.5), (1.0, 2.0, 0.0)],
            vec![(2.0, 0.0, 0.0), (2.0, 1.0, 0.0), (2.0, 2.0, 1.0)],
        ];

        let result = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::from_control_points(
            control_points,
            2, // u_degree
            2, // v_degree
        );

        assert!(result.is_ok());
        let surface = result.unwrap();

        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_count(&surface),
            3
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_count(&surface),
            3
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_degree(&surface),
            2
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_degree(&surface),
            2
        );
        assert!(!<NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::is_rational(&surface));
    }

    #[test]
    fn test_constructor_unit_plane() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_degree(&surface),
            1
        );
        assert_eq!(
            <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_degree(&surface),
            1
        );

        // 単位平面の点評価
        let (x, y, z) =
            <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::point_at_uv(
                &surface, 0.5, 0.5,
            );
        assert!((x - 0.5).abs() < 1e-10);
        assert!((y - 0.5).abs() < 1e-10);
        assert!(z.abs() < 1e-10);
    }

    #[test]
    fn test_properties_knot_vectors() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 2.0, 2.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            None,
            u_knots.clone(),
            v_knots.clone(),
            1,
            1,
        )
        .unwrap();

        let u_knots_ref = <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::u_knots(&surface);
        let v_knots_ref = <NurbsSurface3D<f64> as NurbsSurface3DProperties<f64>>::v_knots(&surface);

        assert_eq!(u_knots_ref, &u_knots[..]);
        assert_eq!(v_knots_ref, &v_knots[..]);
    }

    #[test]
    fn test_measure_point_at_uv() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            1,
            1,
        )
        .unwrap();

        // 平面上の点評価
        let (x, y, z) =
            <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::point_at_uv(
                &surface, 0.0, 0.0,
            );
        assert!((x - 0.0).abs() < 1e-10);
        assert!((y - 0.0).abs() < 1e-10);
        assert!(z.abs() < 1e-10);

        let (x, y, z) =
            <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::point_at_uv(
                &surface, 1.0, 1.0,
            );
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 1.0).abs() < 1e-10);
        assert!(z.abs() < 1e-10);
    }

    #[test]
    fn test_measure_normal_at() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            1,
            1,
        )
        .unwrap();

        // 平面の法線ベクトルはz方向
        let (nx, ny, nz) =
            <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::normal_at(
                &surface, 0.5, 0.5,
            );

        // 正規化確認
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        assert!((len - 1.0).abs() < 1e-10);

        // z方向を向いている
        assert!(nz.abs() > 0.9);
    }

    #[test]
    fn test_measure_surface_area() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            1,
            1,
        )
        .unwrap();

        // 1x1 平面の面積は約1.0
        let area = <NurbsSurface3D<f64> as NurbsSurface3DDerived<f64>>::surface_area(&surface);
        assert!((area - 1.0).abs() < 0.1); // 数値積分の誤差を許容
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn test_measure_tangent_vectors_at() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            1,
            1,
        )
        .unwrap();

        let ((du_x, du_y, du_z), (dv_x, dv_y, dv_z)) =
            <NurbsSurface3D<f64> as geo_contracts::NurbsSurface3DEvaluation<f64>>::tangent_vectors_at(
                &surface, 0.5, 0.5,
            );

        // 平面のタンジェントベクトル
        // u方向はx軸方向
        assert!((du_x - 1.0).abs() < 0.1);
        assert!(du_y.abs() < 0.1);
        assert!(du_z.abs() < 0.1);

        // v方向はy軸方向
        assert!(dv_x.abs() < 0.1);
        assert!((dv_y - 1.0).abs() < 0.1);
        assert!(dv_z.abs() < 0.1);
    }

    #[test]
    fn test_metadata_and_surface_area_capabilities() {
        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::unit_plane();

        // PrimitiveKind
        let kind = <NurbsSurface3D<f64> as PrimitiveMetadata>::primitive_kind(&surface);
        assert_eq!(kind, PrimitiveKind::NurbsSurface3D);

        // Derived capability (面積)
        let area = <NurbsSurface3D<f64> as NurbsSurface3DDerived<f64>>::surface_area(&surface);
        assert!((area - 1.0).abs() < 0.1); // 単位平面の面積は1.0
    }

    #[test]
    fn test_bounded_aabb() {
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 2.0, 0.0)],
            vec![(3.0, 0.0, 0.0), (3.0, 2.0, 1.0)],
        ];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::from_control_points(
            control_points,
            1,
            1,
        )
        .unwrap();

        let aabb_opt = <NurbsSurface3D<f64> as Bounded<f64>>::aabb(&surface);
        assert!(aabb_opt.is_some());

        let aabb = aabb_opt.unwrap();

        // 制御点ベースの境界ボックス
        let min = aabb.min();
        let max = aabb.max();

        assert!((min.x() - 0.0).abs() < 1e-10);
        assert!((min.y() - 0.0).abs() < 1e-10);
        assert!((min.z() - 0.0).abs() < 1e-10);

        assert!((max.x() - 3.0).abs() < 1e-10);
        assert!((max.y() - 2.0).abs() < 1e-10);
        assert!((max.z() - 1.0).abs() < 1e-10);
    }
}
