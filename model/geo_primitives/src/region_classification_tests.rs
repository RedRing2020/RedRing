//! 領域を持つ形状の点の分類（内部 / 境界上 / 外部）のテスト

#[cfg(test)]
mod tests {
    use crate::{
        ConicalSolid3D, CylindricalSolid3D, EllipsoidalSolid3D, Point2D, Point3D, Rect2D, Rect3D,
        SphericalSolid3D, TorusSolid3D, Triangle2D, Triangle3D, Vector3D,
    };
    use geo_contracts::PointClassification::{Inside, OnBoundary, Outside};
    use geo_contracts::{
        ConicalSolid3DContainment, CylindricalSolid3DContainment, EllipsoidalSolid3DContainment,
        PointClassification, Rect2DContainment, SphericalSolid3DContainment,
        TorusSolid3DContainment, Triangle2DContainment, Triangle3DContainment,
    };

    const TOLERANCE: f64 = 1e-3;

    /// 境界上の点 `on` と、境界の内向き単位ベクトル `inward` について、許容誤差の境目の分類を確かめる
    fn assert_boundary_band_3d(
        classify: impl Fn(Point3D<f64>) -> PointClassification,
        on: Point3D<f64>,
        inward: Vector3D<f64>,
    ) {
        let shifted = |scale: f64| on + inward * (scale * TOLERANCE);
        assert_eq!(classify(on), OnBoundary);
        assert_eq!(classify(shifted(0.5)), OnBoundary);
        assert_eq!(classify(shifted(-0.5)), OnBoundary);
        assert_eq!(classify(shifted(2.0)), Inside);
        assert_eq!(classify(shifted(-2.0)), Outside);
    }

    #[test]
    fn rect2d_classify_point() {
        let rect = Rect2D::new(Point2D::new(1.0, 1.0), 4.0, 2.0).unwrap();
        let classify = |x: f64, y: f64| rect.classify_point(&Point2D::new(x, y), TOLERANCE);

        assert_eq!(classify(3.0, 2.0), Inside);
        assert_eq!(classify(5.0, 2.0), OnBoundary);
        assert_eq!(classify(5.0 + 0.5 * TOLERANCE, 2.0), OnBoundary);
        assert_eq!(classify(5.0 - 2.0 * TOLERANCE, 2.0), Inside);
        assert_eq!(classify(5.0 + 2.0 * TOLERANCE, 2.0), Outside);

        // contains_point は許容誤差 0 の分類が Outside でないことと一致する
        for (x, y) in [(3.0, 2.0), (5.0, 3.0), (6.0, 2.0)] {
            let point = Point2D::new(x, y);
            assert_eq!(
                rect.contains_point(&point),
                rect.classify_point(&point, 0.0) != Outside
            );
        }

        assert_eq!(
            Rect2DContainment::classify_point(&rect, (1.0, 2.0)),
            OnBoundary
        );
    }

    #[test]
    fn rect3d_classify_point() {
        // XY 平面上、原点から X 方向 4・Y 方向 2 の矩形
        let rect = Rect3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
            Vector3D::new(0.0, 1.0, 0.0),
            4.0,
            2.0,
        )
        .unwrap();
        let classify =
            |x: f64, y: f64, z: f64| rect.classify_point(&Point3D::new(x, y, z), TOLERANCE);

        assert_eq!(classify(2.0, 1.0, 0.0), Inside);
        assert_eq!(classify(4.0, 1.0, 0.0), OnBoundary);
        assert_eq!(classify(6.0, 1.0, 0.0), Outside);

        // 辺までの距離が許容誤差以内なら、平面外でも OnBoundary
        assert_eq!(classify(4.0, 1.0, 0.5 * TOLERANCE), OnBoundary);

        // 平面からの距離が許容誤差を超える点は、矩形の内側の上方でも Outside
        assert_eq!(classify(2.0, 1.0, 0.5 * TOLERANCE), Inside);
        assert_eq!(classify(2.0, 1.0, 2.0 * TOLERANCE), Outside);

        // contains_point は classify_point が Outside でないことと一致する
        for (x, y, z) in [(2.0, 1.0, 0.0), (4.0, 1.0, 0.0), (2.0, 1.0, 1.0)] {
            let point = Point3D::new(x, y, z);
            assert_eq!(
                rect.contains_point(&point, TOLERANCE),
                rect.classify_point(&point, TOLERANCE) != Outside
            );
        }
    }

    #[test]
    fn triangle2d_classify_point() {
        let triangle = Triangle2D::new(
            Point2D::new(0.0, 0.0),
            Point2D::new(4.0, 0.0),
            Point2D::new(0.0, 4.0),
        )
        .unwrap();
        let classify = |x: f64, y: f64| triangle.classify_point(&Point2D::new(x, y), TOLERANCE);

        assert_eq!(classify(1.0, 1.0), Inside);
        assert_eq!(classify(2.0, 0.0), OnBoundary);
        assert_eq!(classify(2.0, 2.0), OnBoundary);
        assert_eq!(classify(3.0, 3.0), Outside);
        assert_eq!(classify(2.0, -0.5 * TOLERANCE), OnBoundary);
        assert_eq!(classify(2.0, 2.0 * TOLERANCE), Inside);
        assert_eq!(classify(2.0, -2.0 * TOLERANCE), Outside);

        for (x, y) in [(1.0, 1.0), (2.0, 0.0), (3.0, 3.0)] {
            let point = Point2D::new(x, y);
            assert_eq!(
                triangle.contains_point(&point),
                triangle.classify_point(&point, 0.0) != Outside
            );
        }

        assert_eq!(
            Triangle2DContainment::classify_point(&triangle, (1.0, 1.0)),
            Inside
        );
    }

    #[test]
    fn triangle3d_classify_point() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(4.0, 0.0, 0.0),
            Point3D::new(0.0, 4.0, 0.0),
        )
        .unwrap();
        let classify =
            |x: f64, y: f64, z: f64| triangle.classify_point(&Point3D::new(x, y, z), TOLERANCE);

        assert_eq!(classify(1.0, 1.0, 0.0), Inside);
        assert_eq!(classify(2.0, 2.0, 0.0), OnBoundary);
        assert_eq!(classify(3.0, 3.0, 0.0), Outside);
        assert_eq!(classify(2.0, 0.0, 0.5 * TOLERANCE), OnBoundary);
        assert_eq!(classify(1.0, 1.0, -0.5 * TOLERANCE), Inside);
        assert_eq!(classify(1.0, 1.0, -2.0 * TOLERANCE), Outside);

        // trait定義の contains_point は平面からの距離も判定する（三角形の上方の点は含まない）
        assert!(Triangle3DContainment::contains_point(
            &triangle,
            (1.0, 1.0, 0.0)
        ));
        assert!(!Triangle3DContainment::contains_point(
            &triangle,
            (1.0, 1.0, 5.0)
        ));
        assert_eq!(
            Triangle3DContainment::classify_point(&triangle, (1.0, 1.0, 0.0)),
            Inside
        );
    }

    #[test]
    fn spherical_solid_classify_point() {
        let sphere = SphericalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 2.0).unwrap();
        let classify = |point: Point3D<f64>| sphere.classify_point(point, TOLERANCE);

        assert_eq!(classify(Point3D::new(1.0, 2.0, 3.0)), Inside);
        assert_eq!(classify(Point3D::new(5.0, 2.0, 3.0)), Outside);
        assert_boundary_band_3d(
            classify,
            Point3D::new(3.0, 2.0, 3.0),
            Vector3D::new(-1.0, 0.0, 0.0),
        );

        for point in [
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(3.0, 2.0, 3.0),
            Point3D::new(5.0, 2.0, 3.0),
        ] {
            assert_eq!(
                sphere.contains_point(point),
                sphere.classify_point(point, 0.0) != Outside
            );
        }
        assert_eq!(
            SphericalSolid3DContainment::classify_point(&sphere, (1.0, 2.0, 3.0)),
            Inside
        );
    }

    #[test]
    fn ellipsoidal_solid_classify_point() {
        let ellipsoid =
            EllipsoidalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 3.0, 2.0, 1.0).unwrap();
        let classify = |point: Point3D<f64>| ellipsoid.classify_point(&point, TOLERANCE);

        assert_eq!(classify(Point3D::new(0.0, 0.0, 0.0)), Inside);
        assert_eq!(classify(Point3D::new(0.0, 0.0, 1.5)), Outside);
        assert_boundary_band_3d(
            classify,
            Point3D::new(0.0, 0.0, 1.0),
            Vector3D::new(0.0, 0.0, -1.0),
        );
        assert_boundary_band_3d(
            classify,
            Point3D::new(3.0, 0.0, 0.0),
            Vector3D::new(-1.0, 0.0, 0.0),
        );

        assert_eq!(
            EllipsoidalSolid3DContainment::classify_point(&ellipsoid, (0.0, 2.0, 0.0)),
            OnBoundary
        );
    }

    #[test]
    fn cylindrical_solid_classify_point() {
        // 底面の中心が原点、Z 軸方向に高さ 4、半径 2 の円柱
        let cylinder =
            CylindricalSolid3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 2.0, 4.0).unwrap();
        let classify = |point: Point3D<f64>| cylinder.classify_point(&point, TOLERANCE);

        assert_eq!(classify(Point3D::new(0.0, 0.0, 2.0)), Inside);
        assert_eq!(classify(Point3D::new(3.0, 0.0, 2.0)), Outside);
        assert_eq!(classify(Point3D::new(0.0, 0.0, 5.0)), Outside);

        // 側面・上下の端面・端面の縁
        assert_boundary_band_3d(
            classify,
            Point3D::new(2.0, 0.0, 2.0),
            Vector3D::new(-1.0, 0.0, 0.0),
        );
        assert_boundary_band_3d(
            classify,
            Point3D::new(0.5, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
        );
        assert_boundary_band_3d(
            classify,
            Point3D::new(0.5, 0.0, 4.0),
            Vector3D::new(0.0, 0.0, -1.0),
        );
        assert_eq!(classify(Point3D::new(2.0, 0.0, 4.0)), OnBoundary);

        assert_eq!(
            CylindricalSolid3DContainment::classify_point(&cylinder, (1.0, 0.0, 1.0)),
            Inside
        );
    }

    #[test]
    fn conical_solid_classify_point() {
        // 底面の半径 2・高さ 4、底面の中心が原点で Z 軸方向に頂点がある円錐
        let cone = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 2.0, 4.0).unwrap();
        let classify = |point: Point3D<f64>| cone.classify_point(point, TOLERANCE);

        assert_eq!(classify(Point3D::new(0.0, 0.0, 1.0)), Inside);
        assert_eq!(classify(Point3D::new(3.0, 0.0, 1.0)), Outside);

        // 底面・側面（プロファイルの母線 (2, 0)-(0, 4) の中点 (1, 2)、内向き法線 (-2, -1) / √5）・頂点
        assert_boundary_band_3d(
            classify,
            Point3D::new(0.5, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
        );
        let sqrt5 = 5.0_f64.sqrt();
        assert_boundary_band_3d(
            classify,
            Point3D::new(1.0, 0.0, 2.0),
            Vector3D::new(-2.0 / sqrt5, 0.0, -1.0 / sqrt5),
        );
        assert_eq!(classify(Point3D::new(0.0, 0.0, 4.0)), OnBoundary);

        // 軸上の点は、底面・側面から離れていれば Inside
        assert_eq!(classify(Point3D::new(0.0, 0.0, 2.0)), Inside);

        assert_eq!(
            ConicalSolid3DContainment::classify_point(&cone, (0.0, 0.0, 1.0)),
            Inside
        );
    }

    #[test]
    fn torus_solid_classify_point() {
        // 主半径 3・管の半径 1、Z 軸まわりのトーラス
        let torus = TorusSolid3D::standard(3.0, 1.0).unwrap();
        let classify = |point: Point3D<f64>| torus.classify_point(&point, TOLERANCE);

        assert_eq!(classify(Point3D::new(3.0, 0.0, 0.0)), Inside);
        assert_eq!(classify(Point3D::new(0.0, 0.0, 0.0)), Outside);
        assert_boundary_band_3d(
            classify,
            Point3D::new(4.0, 0.0, 0.0),
            Vector3D::new(-1.0, 0.0, 0.0),
        );
        assert_boundary_band_3d(
            classify,
            Point3D::new(0.0, 3.0, 1.0),
            Vector3D::new(0.0, 0.0, -1.0),
        );

        for point in [
            Point3D::new(3.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(0.0, 0.0, 0.0),
        ] {
            assert_eq!(
                torus.contains_point(&point),
                torus.classify_point(&point, 0.0) != Outside
            );
        }
        assert_eq!(
            TorusSolid3DContainment::classify_point(&torus, (0.0, 3.0, 0.0)),
            Inside
        );
    }
}
