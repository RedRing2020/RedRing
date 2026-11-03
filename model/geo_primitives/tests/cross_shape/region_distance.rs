//! 領域を持つ形状の点との距離のテスト

use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    ConicalSolid3DConstructor, ConicalSolid3DDistance, EllipsoidalSolid3DConstructor,
    EllipsoidalSolid3DDistance, EllipsoidalSurface3DConstructor, EllipsoidalSurface3DDistance,
    SphericalSolid3DConstructor, SphericalSolid3DDistance, TorusSolid3DConstructor,
    TorusSolid3DDistance, Triangle3DConstructor, Triangle3DDistance,
};
use geo_primitives::{
    ConicalSolid3D, EllipsoidalSolid3D, EllipsoidalSurface3D, Point3D, SphericalSolid3D,
    TorusSolid3D, Triangle3D,
};

#[test]
fn triangle3d_distance_from_both_sides_of_plane() {
    let triangle = Triangle3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        Point3D::new(4.0, 0.0, 0.0),
        Point3D::new(0.0, 4.0, 0.0),
    )
    .unwrap();

    // 平面の表側・裏側のどちらからでも、平面までの距離
    for z in [2.0, -2.0] {
        let distance = triangle.distance_to_point(Point3D::new(1.0, 1.0, z));
        assert!((distance - 2.0).abs() < TOLERANCE_F64);
    }
}

#[test]
fn sphere_distance_is_distance_to_region() {
    let sphere = SphericalSolid3D::new_standard(Point3D::new(0.0_f64, 0.0, 0.0), 2.0).unwrap();
    let distance = |x: f64, y: f64, z: f64| {
        SphericalSolid3DDistance::distance_to_point(&sphere, Point3D::new(x, y, z))
    };

    // 内部・表面上は 0、外部は表面までの距離
    assert!(distance(0.0, 0.0, 0.0).abs() < TOLERANCE_F64);
    assert!(distance(2.0, 0.0, 0.0).abs() < TOLERANCE_F64);
    assert!((distance(0.0, 3.0, 4.0) - 3.0).abs() < TOLERANCE_F64);
}

#[test]
fn torus_distance_is_distance_to_region() {
    let torus = TorusSolid3D::new_standard(Point3D::origin(), 3.0_f64, 1.0).unwrap();
    let distance = |x: f64, y: f64, z: f64| {
        TorusSolid3DDistance::distance_to_point(&torus, Point3D::new(x, y, z))
    };

    // トーラスの内部は 0、外部はトーラスの表面までの距離
    assert!(distance(3.0, 0.0, 0.0).abs() < TOLERANCE_F64);
    assert!(distance(3.5, 0.0, 0.5).abs() < TOLERANCE_F64);
    assert!((distance(0.0, 0.0, 0.0) - 2.0).abs() < TOLERANCE_F64);
    assert!((distance(0.0, 3.0, 3.0) - 2.0).abs() < TOLERANCE_F64);
}

#[test]
fn cone_distance_is_exact_distance_to_region() {
    // 底面の半径 2・高さ 4、底面の中心が原点で Z 軸方向に頂点がある円錐
    let cone = ConicalSolid3D::new_standard(Point3D::new(0.0_f64, 0.0, 0.0), 2.0, 4.0).unwrap();
    let distance = |x: f64, y: f64, z: f64| {
        ConicalSolid3DDistance::distance_to_point(&cone, Point3D::new(x, y, z))
    };

    // 底面の縁・頂点・底面の外側の点
    assert!((distance(5.0, 0.0, 0.0) - 3.0).abs() < TOLERANCE_F64);
    assert!((distance(0.0, 0.0, 6.0) - 2.0).abs() < TOLERANCE_F64);
    assert!((distance(0.0, 0.0, -1.5) - 1.5).abs() < TOLERANCE_F64);

    // プロファイル面上で母線 (2, 0)-(0, 4) の中点 (1, 2) から外向き法線 (2, 1) / √5 の方向に √5 離れた
    // (軸からの距離 3, 高さ 3) の点は、母線までの距離 √5
    assert!((distance(0.0, 3.0, 3.0) - 5.0_f64.sqrt()).abs() < TOLERANCE_F64);

    // 内部は 0
    assert!(distance(0.5, 0.0, 1.0).abs() < TOLERANCE_F64);
}

#[test]
fn ellipsoid_distance_is_exact() {
    let solid =
        EllipsoidalSolid3D::new_standard(Point3D::new(0.0_f64, 0.0, 0.0), 4.0, 1.0, 1.0).unwrap();
    let surface =
        EllipsoidalSurface3D::new_standard(Point3D::new(0.0_f64, 0.0, 0.0), 4.0, 1.0, 1.0).unwrap();

    // 赤道面上の点は、その面の楕円までの距離（楕円の距離の参照値）
    let outside = Point3D::new(5.0, 2.0, 0.0);
    let expected = 2.070_522_627_212_593;
    assert!((solid.distance_to_point(outside) - expected).abs() < TOLERANCE_F64);
    assert!((surface.distance_to_point(outside) - expected).abs() < TOLERANCE_F64);

    // 立体の内部は 0、面までの距離は表面までの距離（中心からは最も短い半軸の長さ）
    let center = Point3D::new(0.0, 0.0, 0.0);
    assert_eq!(solid.distance_to_point(center), 0.0);
    assert!((surface.distance_to_point(center) - 1.0).abs() < TOLERANCE_F64);
}
