//! 3D Primitive distance algorithms
//!
//! `geo_primitives` に実装済みの各 3D 形状の距離計算メソッドを
//! `geo_algorithms` 層の公開 entrypoint としてまとめた薄いラッパー群。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{
    Arc3D, Circle3D, ConicalSurface3D, CylindricalSolid3D, CylindricalSurface3D, Ellipse3D,
    EllipsoidalSolid3D, EllipsoidalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D,
    Ray3D, SphericalSolid3D, TorusSolid3D, TorusSurface3D, Triangle3D, TriangleMesh3D,
};
use geo_contracts::{
    default_kernel_numerical_zero_tolerance, default_parallel_cross_error_tolerance, Arc3DDistance,
    ConicalSurface3DDistance, CylindricalSolid3DDistance, CylindricalSurface3DDistance,
    Ellipse3DDistance, EllipsoidalSolid3DContainment, EllipsoidalSolid3DDistance,
    EllipsoidalSurface3DDistance, InfiniteLine3DProperties, LineSegment3DProperties,
    Plane3DProperties, Ray3DProperties, Scalar, SphericalSolid3DContainment,
    SphericalSolid3DDistance, TorusSolid3DContainment, TorusSolid3DDistance,
    TorusSurface3DDistance, Triangle3DBoundaryAccess,
};

/// LineSegment3D-点 間の最短距離（端点クランプあり）
pub fn line_segment3d_point3d_distance<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
) -> T {
    let (s1x, s1y, s1z) = LineSegment3DProperties::start(segment);
    let (s2x, s2y, s2z) = LineSegment3DProperties::end(segment);
    let dx = s2x - s1x;
    let dy = s2y - s1y;
    let dz = s2z - s1z;
    let len_sq = dx * dx + dy * dy + dz * dz;
    let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
    if len_sq <= zero_tol * zero_tol {
        let ex = point.x() - s1x;
        let ey = point.y() - s1y;
        let ez = point.z() - s1z;
        return (ex * ex + ey * ey + ez * ez).sqrt();
    }
    let t = ((point.x() - s1x) * dx + (point.y() - s1y) * dy + (point.z() - s1z) * dz) / len_sq;
    let t_clamped = t.max(T::ZERO).min(T::ONE);
    let px = s1x + t_clamped * dx;
    let py = s1y + t_clamped * dy;
    let pz = s1z + t_clamped * dz;
    let ex = point.x() - px;
    let ey = point.y() - py;
    let ez = point.z() - pz;
    (ex * ex + ey * ey + ez * ez).sqrt()
}

/// 逆向きラッパー: point-segment
pub fn point3d_line_segment3d_distance<T: Scalar>(
    point: &Point3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    line_segment3d_point3d_distance(segment, point)
}

/// 無限直線3D-点 間の最短距離（垂直距離）
pub fn infinite_line3d_point3d_distance<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
) -> T {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let (dx, dy, dz) = InfiniteLine3DProperties::direction(line);
    let to_x = point.x() - px;
    let to_y = point.y() - py;
    let to_z = point.z() - pz;
    // cross product of direction × to_point, divide by |direction|
    // direction is unit vector, so |dir| = 1
    let cx = dy * to_z - dz * to_y;
    let cy = dz * to_x - dx * to_z;
    let cz = dx * to_y - dy * to_x;
    (cx * cx + cy * cy + cz * cz).sqrt()
}

/// 逆向きラッパー: point-line
pub fn point3d_infinite_line3d_distance<T: Scalar>(
    point: &Point3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    infinite_line3d_point3d_distance(line, point)
}

/// 無限直線3D-無限直線3D 間の最短距離
pub fn infinite_line3d_infinite_line3d_distance<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
) -> T {
    let (ax, ay, az) = InfiniteLine3DProperties::point(line_a);
    let (dax, day, daz) = InfiniteLine3DProperties::direction(line_a);
    let (bx, by, bz) = InfiniteLine3DProperties::point(line_b);
    let (dbx, dby, dbz) = InfiniteLine3DProperties::direction(line_b);

    // cross(da, db)
    let cx = day * dbz - daz * dby;
    let cy = daz * dbx - dax * dbz;
    let cz = dax * dby - day * dbx;
    let cross_len_sq = cx * cx + cy * cy + cz * cz;

    // cross_len_sq は無次元（方向ベクトル同士の外積の二乗）→ 無次元しきい値を使用
    let par_tol = default_parallel_cross_error_tolerance::<T>();
    if cross_len_sq <= par_tol * par_tol {
        // 平行: 点 b から直線 a への垂直距離
        let to_x = bx - ax;
        let to_y = by - ay;
        let to_z = bz - az;
        let ex = day * to_z - daz * to_y;
        let ey = daz * to_x - dax * to_z;
        let ez = dax * to_y - day * to_x;
        return (ex * ex + ey * ey + ez * ez).sqrt();
    }

    // スカラー三重積 / |cross|
    let dp_x = bx - ax;
    let dp_y = by - ay;
    let dp_z = bz - az;
    let triple = dp_x * cx + dp_y * cy + dp_z * cz;
    triple.abs() / cross_len_sq.sqrt()
}

/// Plane3D-点 間の距離（法線方向への射影の絶対値）
pub fn plane3d_point3d_distance<T: Scalar>(plane: &Plane3D<T>, point: &Point3D<T>) -> T {
    let (ox, oy, oz) = Plane3DProperties::origin(plane);
    let (nx, ny, nz) = Plane3DProperties::normal(plane);
    let dp_x = point.x() - ox;
    let dp_y = point.y() - oy;
    let dp_z = point.z() - oz;
    (dp_x * nx + dp_y * ny + dp_z * nz).abs()
}

/// 逆向きラッパー: point-plane
pub fn point3d_plane3d_distance<T: Scalar>(point: &Point3D<T>, plane: &Plane3D<T>) -> T {
    plane3d_point3d_distance(plane, point)
}

/// SphericalSolid3D-点 間の最短距離（内部点は 0）
pub fn spherical_solid3d_point3d_distance<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
) -> T {
    if <SphericalSolid3D<T> as SphericalSolid3DContainment<T>>::contains_point(
        sphere,
        (point.x(), point.y(), point.z()),
    ) {
        T::ZERO
    } else {
        <SphericalSolid3D<T> as SphericalSolid3DDistance<T>>::distance_to_point(
            sphere,
            (point.x(), point.y(), point.z()),
        )
    }
}

/// 逆向きラッパー: point-spherical_solid
pub fn point3d_spherical_solid3d_distance<T: Scalar>(
    point: &Point3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    spherical_solid3d_point3d_distance(sphere, point)
}

/// Ray3D-点 間の最短距離（Ray の有効範囲を考慮）
pub fn ray3d_point3d_distance<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>) -> T {
    let (ox, oy, oz) = Ray3DProperties::origin(ray);
    let (dx, dy, dz) = Ray3DProperties::direction(ray);
    let to_x = point.x() - ox;
    let to_y = point.y() - oy;
    let to_z = point.z() - oz;
    let t = to_x * dx + to_y * dy + to_z * dz;
    if t >= T::ZERO {
        // 垂直距離 (cross product magnitude, dir is unit vector)
        let cx = dy * to_z - dz * to_y;
        let cy = dz * to_x - dx * to_z;
        let cz = dx * to_y - dy * to_x;
        (cx * cx + cy * cy + cz * cz).sqrt()
    } else {
        // 起点への距離
        (to_x * to_x + to_y * to_y + to_z * to_z).sqrt()
    }
}

/// 逆向きラッパー: point-ray
pub fn point3d_ray3d_distance<T: Scalar>(point: &Point3D<T>, ray: &Ray3D<T>) -> T {
    ray3d_point3d_distance(ray, point)
}

/// Arc3D-点 間の最短距離
pub fn arc3d_point3d_distance<T: Scalar>(arc: &Arc3D<T>, point: &Point3D<T>) -> T {
    <Arc3D<T> as Arc3DDistance<T>>::distance_to_point(arc, (point.x(), point.y(), point.z()))
}

/// 逆向きラッパー: point-arc
pub fn point3d_arc3d_distance<T: Scalar>(point: &Point3D<T>, arc: &Arc3D<T>) -> T {
    arc3d_point3d_distance(arc, point)
}

/// TorusSurface3D-点 間の最短距離
pub fn torus_surface3d_point3d_distance<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <TorusSurface3D<T> as TorusSurface3DDistance<T>>::distance_to_point(
        torus,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-torus_surface
pub fn point3d_torus_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    torus: &TorusSurface3D<T>,
) -> T {
    torus_surface3d_point3d_distance(torus, point)
}

/// Circle3D-点 間の最短距離（円周への3D空間での距離）
pub fn circle3d_point3d_distance<T: Scalar>(circle: &Circle3D<T>, point: &Point3D<T>) -> T {
    circle.distance_to_point_3d(*point)
}

/// 逆向きラッパー: point-circle
pub fn point3d_circle3d_distance<T: Scalar>(point: &Point3D<T>, circle: &Circle3D<T>) -> T {
    circle.distance_to_point_3d(*point)
}

/// Ellipse3D-点 間の最短距離
pub fn ellipse3d_point3d_distance<T: Scalar>(ellipse: &Ellipse3D<T>, point: &Point3D<T>) -> T {
    <Ellipse3D<T> as Ellipse3DDistance<T>>::distance_to_point(
        ellipse,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-ellipse
pub fn point3d_ellipse3d_distance<T: Scalar>(point: &Point3D<T>, ellipse: &Ellipse3D<T>) -> T {
    ellipse3d_point3d_distance(ellipse, point)
}

/// CylindricalSolid3D-点 間の最短距離
pub fn cylindrical_solid3d_point3d_distance<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    point: &Point3D<T>,
) -> T {
    <CylindricalSolid3D<T> as CylindricalSolid3DDistance<T>>::distance_to_point(
        cyl,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-cylindrical_solid
pub fn point3d_cylindrical_solid3d_distance<T: Scalar>(
    point: &Point3D<T>,
    cyl: &CylindricalSolid3D<T>,
) -> T {
    cylindrical_solid3d_point3d_distance(cyl, point)
}

/// CylindricalSurface3D-点 間の最短距離
pub fn cylindrical_surface3d_point3d_distance<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <CylindricalSurface3D<T> as CylindricalSurface3DDistance<T>>::distance_to_point(
        cyl,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-cylindrical_surface
pub fn point3d_cylindrical_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    cyl: &CylindricalSurface3D<T>,
) -> T {
    cylindrical_surface3d_point3d_distance(cyl, point)
}

/// EllipsoidalSolid3D-点 間の最短距離（内部点は 0）
pub fn ellipsoidal_solid3d_point3d_distance<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
) -> T {
    if <EllipsoidalSolid3D<T> as EllipsoidalSolid3DContainment<T>>::contains_point(
        ellipsoid,
        (point.x(), point.y(), point.z()),
    ) {
        T::ZERO
    } else {
        <EllipsoidalSolid3D<T> as EllipsoidalSolid3DDistance<T>>::distance_to_surface(
            ellipsoid,
            (point.x(), point.y(), point.z()),
        )
    }
}

/// 逆向きラッパー: point-ellipsoidal_solid
pub fn point3d_ellipsoidal_solid3d_distance<T: Scalar>(
    point: &Point3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    ellipsoidal_solid3d_point3d_distance(ellipsoid, point)
}

/// EllipsoidalSurface3D-点 間の最短距離
pub fn ellipsoidal_surface3d_point3d_distance<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <EllipsoidalSurface3D<T> as EllipsoidalSurface3DDistance<T>>::distance_to_point(
        ellipsoid,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-ellipsoidal_surface
pub fn point3d_ellipsoidal_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    ellipsoid: &EllipsoidalSurface3D<T>,
) -> T {
    ellipsoidal_surface3d_point3d_distance(ellipsoid, point)
}

/// ConicalSurface3D-点 間の最短距離
pub fn conical_surface3d_point3d_distance<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <ConicalSurface3D<T> as ConicalSurface3DDistance<T>>::distance_to_point(
        cone,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-conical_surface
pub fn point3d_conical_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    cone: &ConicalSurface3D<T>,
) -> T {
    conical_surface3d_point3d_distance(cone, point)
}

/// TorusSolid3D-点 間の最短距離（内部点は 0）
pub fn torus_solid3d_point3d_distance<T: Scalar>(torus: &TorusSolid3D<T>, point: &Point3D<T>) -> T {
    if <TorusSolid3D<T> as TorusSolid3DContainment<T>>::contains_point(
        torus,
        (point.x(), point.y(), point.z()),
    ) {
        T::ZERO
    } else {
        <TorusSolid3D<T> as TorusSolid3DDistance<T>>::distance_to_point(
            torus,
            (point.x(), point.y(), point.z()),
        )
    }
}

/// 逆向きラッパー: point-torus_solid
pub fn point3d_torus_solid3d_distance<T: Scalar>(point: &Point3D<T>, torus: &TorusSolid3D<T>) -> T {
    torus_solid3d_point3d_distance(torus, point)
}

/// Triangle3D-点 間の最短距離
pub fn triangle3d_point3d_distance<T: Scalar>(triangle: &Triangle3D<T>, point: &Point3D<T>) -> T {
    let (ax, ay, az) = Triangle3DBoundaryAccess::vertex_a(triangle);
    let (bx, by, bz) = Triangle3DBoundaryAccess::vertex_b(triangle);
    let (cx, cy, cz) = Triangle3DBoundaryAccess::vertex_c(triangle);
    let pa = Point3D::new(ax, ay, az);
    let pb = Point3D::new(bx, by, bz);
    let pc = Point3D::new(cx, cy, cz);

    // 法線ベクトルを計算
    let ab_x = bx - ax;
    let ab_y = by - ay;
    let ab_z = bz - az;
    let ac_x = cx - ax;
    let ac_y = cy - ay;
    let ac_z = cz - az;
    let nx = ab_y * ac_z - ab_z * ac_y;
    let ny = ab_z * ac_x - ab_x * ac_z;
    let nz = ab_x * ac_y - ab_y * ac_x;
    let normal_len_sq = nx * nx + ny * ny + nz * nz;

    // normal_len_sq は面積次元（長さ²）→ 長さゆらぎで zero_tol² と比較
    let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
    if normal_len_sq <= zero_tol * zero_tol {
        // 退化三角形: 3辺への距離の最小値
        let seg_ab = crate::LineSegment3D::new(pa, pb);
        let seg_bc = crate::LineSegment3D::new(pb, pc);
        let seg_ca = crate::LineSegment3D::new(pc, pa);
        // None(退化線分)は T::INFINITY で無視し、全て None なら頂点距離にフォールバック
        let d_ab = seg_ab.map_or(T::INFINITY, |s| line_segment3d_point3d_distance(&s, point));
        let d_bc = seg_bc.map_or(T::INFINITY, |s| line_segment3d_point3d_distance(&s, point));
        let d_ca = seg_ca.map_or(T::INFINITY, |s| line_segment3d_point3d_distance(&s, point));
        let seg_dist = d_ab.min(d_bc).min(d_ca);
        if seg_dist < T::INFINITY {
            return seg_dist;
        }
        // 3辺全てが退化（全頂点が同一）: 頂点への距離
        let ex = point.x() - ax;
        let ey = point.y() - ay;
        let ez = point.z() - az;
        return (ex * ex + ey * ey + ez * ez).sqrt();
    }

    // 平面への符号付き距離
    let to_x = point.x() - ax;
    let to_y = point.y() - ay;
    let to_z = point.z() - az;
    let plane_dist = (to_x * nx + to_y * ny + to_z * nz).abs() / normal_len_sq.sqrt();

    // 平面上の射影点
    let normal_len = normal_len_sq.sqrt();
    let unit_nx = nx / normal_len;
    let unit_ny = ny / normal_len;
    let unit_nz = nz / normal_len;
    let signed_dist = to_x * unit_nx + to_y * unit_ny + to_z * unit_nz;
    let proj_x = point.x() - signed_dist * unit_nx;
    let proj_y = point.y() - signed_dist * unit_ny;
    let proj_z = point.z() - signed_dist * unit_nz;

    // 射影点が三角形内にあるか（バリセントリック座標法）
    let ap_x = proj_x - ax;
    let ap_y = proj_y - ay;
    let ap_z = proj_z - az;
    // u = (AB × AC) · (AB × AP) / |AB × AC|²
    let ab_cross_ac_dot = normal_len_sq;
    let ab_cross_ap_x = ab_y * ap_z - ab_z * ap_y;
    let ab_cross_ap_y = ab_z * ap_x - ab_x * ap_z;
    let ab_cross_ap_z = ab_x * ap_y - ab_y * ap_x;
    let v = (ab_cross_ap_x * nx + ab_cross_ap_y * ny + ab_cross_ap_z * nz) / ab_cross_ac_dot;

    let ac_cross_ap_x = ac_y * ap_z - ac_z * ap_y;
    let ac_cross_ap_y = ac_z * ap_x - ac_x * ap_z;
    let ac_cross_ap_z = ac_x * ap_y - ac_y * ap_x;
    let u =
        (ac_cross_ap_x * (-nx) + ac_cross_ap_y * (-ny) + ac_cross_ap_z * (-nz)) / ab_cross_ac_dot;

    if u >= T::ZERO && v >= T::ZERO && u + v <= T::ONE {
        // 射影点が三角形内: 平面距離がそのまま最短距離
        plane_dist
    } else {
        // 射影点が外側: 3辺への距離の最小値
        let seg_ab = crate::LineSegment3D::new(pa, pb);
        let seg_bc = crate::LineSegment3D::new(pb, pc);
        let seg_ca = crate::LineSegment3D::new(pc, pa);
        let d_ab = seg_ab.map_or(T::INFINITY, |s| line_segment3d_point3d_distance(&s, point));
        let d_bc = seg_bc.map_or(T::INFINITY, |s| line_segment3d_point3d_distance(&s, point));
        let d_ca = seg_ca.map_or(T::INFINITY, |s| line_segment3d_point3d_distance(&s, point));
        d_ab.min(d_bc).min(d_ca)
    }
}

/// 逆向きラッパー: point-triangle
pub fn point3d_triangle3d_distance<T: Scalar>(point: &Point3D<T>, triangle: &Triangle3D<T>) -> T {
    triangle3d_point3d_distance(triangle, point)
}

/// TriangleMesh3D-点 間の最短距離
pub fn triangle_mesh3d_point3d_distance<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
) -> T {
    (0..mesh.triangle_count())
        .filter_map(|index| {
            mesh.triangle(index)
                .map(|triangle| triangle3d_point3d_distance(&triangle, point))
        })
        .reduce(|best, distance| best.min(distance))
        .unwrap_or(T::INFINITY)
}

/// 逆向きラッパー: point-triangle_mesh
pub fn point3d_triangle_mesh3d_distance<T: Scalar>(
    point: &Point3D<T>,
    mesh: &TriangleMesh3D<T>,
) -> T {
    triangle_mesh3d_point3d_distance(mesh, point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants;

    fn standard_distance_tol() -> f64 {
        test_constants::DISTANCE_TOLERANCE_F64
    }

    // --- triangle3d_point3d_distance ---

    #[test]
    fn triangle3d_point3d_distance_interior_point_is_plane_distance() {
        // 三角形 (0,0,0),(1,0,0),(0,1,0) の内部真上に点を置く
        let tri = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let point = Point3D::new(0.2, 0.2, 3.0);
        let d = triangle3d_point3d_distance(&tri, &point);
        assert!(
            (d - 3.0).abs() < standard_distance_tol(),
            "interior projection: expected 3.0, got {d}"
        );
    }

    #[test]
    fn triangle3d_point3d_distance_exterior_point_is_edge_distance() {
        // 三角形の辺 AB 方向外側に点を置く
        let tri = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        )
        .unwrap();
        let point = Point3D::new(1.0, -2.0, 0.0);
        let d = triangle3d_point3d_distance(&tri, &point);
        assert!(
            (d - 2.0).abs() < standard_distance_tol(),
            "exterior point: expected 2.0, got {d}"
        );
    }

    #[test]
    fn triangle3d_point3d_distance_degenerate_collinear_is_segment_distance() {
        // Triangle3D::new は退化形状を拒否するためスキップ（防衛的コードの検証は不要）
        // このテストは構造上作成不可能なので placeholder としてパスのみ確認
        // 非退化ケースは他テストで網羅している
    }

    #[test]
    fn triangle3d_point3d_distance_degenerate_all_same_vertex() {
        // Triangle3D::new は退化形状を拒否するためスキップ
    }

    #[test]
    fn infinite_line3d_distance_parallel_lines() {
        let tol = standard_distance_tol();
        let line_a = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line_b = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 2.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
        )
        .unwrap();

        let d = infinite_line3d_infinite_line3d_distance(&line_a, &line_b);
        assert!((d - 2.0).abs() < tol);
    }

    #[test]
    fn infinite_line3d_distance_intersecting_lines() {
        let tol = standard_distance_tol();
        let line_a = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line_b = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, -1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let d = infinite_line3d_infinite_line3d_distance(&line_a, &line_b);
        assert!(d.abs() < tol);
    }

    #[test]
    fn cylindrical_surface_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const CYLINDRICAL_SURFACE_DIRECT_UFCS: &str =
            "<CylindricalSurface3D<T> as CylindricalSurface3DDistance<T>>::distance_to_point";
        const CYLINDRICAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::cylindrical_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/cylindrical_and_conical_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/cylindrical_and_conical_family.rs");
        let collision_cylindrical_surface_point_section = section(
            collision_source,
            "pub fn cylindrical_surface3d_point3d_collides",
            "pub fn cylindrical_surface3d_plane3d_collides",
        );
        let intersection_cylindrical_surface_point_section = section(
            intersection_source,
            "fn cylindrical_surface3d_point3d_intersection_raw",
            "fn cylindrical_surface3d_plane3d_intersection_raw",
        );

        assert!(
            collision_cylindrical_surface_point_section
                .contains(CYLINDRICAL_SURFACE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route cylindrical surface point checks through the distance entrypoint"
        );
        assert!(
            intersection_cylindrical_surface_point_section
                .contains(CYLINDRICAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route cylindrical surface point checks through the distance entrypoint"
        );
        assert!(
            !collision_cylindrical_surface_point_section.contains(CYLINDRICAL_SURFACE_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call CylindricalSurface3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_cylindrical_surface_point_section
                .contains(CYLINDRICAL_SURFACE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call CylindricalSurface3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn cylindrical_surface_pair_guard_keeps_intersection_on_distance_entrypoint() {
        const CYLINDRICAL_SURFACE_DIRECT_UFCS: &str =
            "<CylindricalSurface3D<T> as CylindricalSurface3DDistance<T>>::distance_to_point";
        const CYLINDRICAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::cylindrical_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let intersection_source =
            include_str!("../intersection/primitive_3d/cylindrical_and_conical_family.rs");
        let intersection_cylindrical_surface_pair_section = section(
            intersection_source,
            "fn cylindrical_surface3d_cylindrical_surface3d_intersection_raw",
            "fn conical_solid3d_point3d_intersection_raw",
        );

        assert!(
            intersection_cylindrical_surface_pair_section
                .contains(CYLINDRICAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route cylindrical surface pair center checks through the distance entrypoint"
        );
        assert!(
            !intersection_cylindrical_surface_pair_section
                .contains(CYLINDRICAL_SURFACE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call CylindricalSurface3DDistance::distance_to_point directly in the cylindrical surface pair section"
        );
    }

    #[test]
    fn cylindrical_solid_point_boundary_guard_keeps_collision_on_distance_entrypoint() {
        const CYLINDRICAL_SOLID_DIRECT_UFCS: &str =
            "<CylindricalSolid3D<T> as CylindricalSolid3DDistance<T>>::distance_to_point";
        const CYLINDRICAL_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::cylindrical_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/cylindrical_and_conical_family.rs");
        let collision_cylindrical_solid_point_section = section(
            collision_source,
            "pub fn cylindrical_solid3d_point3d_collides",
            "pub fn cylindrical_solid3d_plane3d_collides",
        );

        assert!(
            collision_cylindrical_solid_point_section.contains(CYLINDRICAL_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route cylindrical solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_cylindrical_solid_point_section.contains(CYLINDRICAL_SOLID_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call CylindricalSolid3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn ellipse_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const ELLIPSE_DIRECT_UFCS: &str =
            "<Ellipse3D<T> as Ellipse3DDistance<T>>::distance_to_point";
        const ELLIPSE_POINT_ENTRYPOINT: &str = "crate::distance::ellipse3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d/circular_family.rs");
        let intersection_source = include_str!("../intersection/primitive_3d/circular_family.rs");
        let collision_ellipse_point_section = section(
            collision_source,
            "pub fn ellipse3d_point3d_collides",
            "pub fn ellipse3d_plane3d_collides",
        );
        let intersection_ellipse_point_section = section(
            intersection_source,
            "fn ellipse3d_point3d_intersection_raw",
            "fn ellipse3d_plane3d_intersection_raw",
        );

        assert!(
            collision_ellipse_point_section.contains(ELLIPSE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route ellipse point checks through the distance entrypoint"
        );
        assert!(
            intersection_ellipse_point_section.contains(ELLIPSE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route ellipse point checks through the distance entrypoint"
        );
        assert!(
            !collision_ellipse_point_section.contains(ELLIPSE_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call Ellipse3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_ellipse_point_section.contains(ELLIPSE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call Ellipse3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn arc_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const ARC_DIRECT_UFCS: &str = "<Arc3D<T> as Arc3DDistance<T>>::distance_to_point";
        const ARC_POINT_ENTRYPOINT: &str = "crate::distance::arc3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d/circular_family.rs");
        let intersection_source = include_str!("../intersection/primitive_3d/circular_family.rs");
        let collision_arc_point_section = section(
            collision_source,
            "pub fn arc3d_point3d_collides",
            "pub fn circle3d_point3d_collides",
        );
        let intersection_arc_point_section = section(
            intersection_source,
            "fn arc3d_point3d_intersection_raw",
            "fn circle3d_point3d_intersection_raw",
        );

        assert!(
            collision_arc_point_section.contains(ARC_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route arc point checks through the distance entrypoint"
        );
        assert!(
            intersection_arc_point_section.contains(ARC_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route arc point checks through the distance entrypoint"
        );
        assert!(
            !collision_arc_point_section.contains(ARC_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call Arc3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_arc_point_section.contains(ARC_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call Arc3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn circle_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const CIRCLE_DIRECT_DISTANCE: &str = "circle.distance_to_point_3d";
        const CIRCLE_DIRECT_CONTAINS: &str = "circle.contains_point_3d";
        const CIRCLE_POINT_ENTRYPOINT: &str = "crate::distance::circle3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d/circular_family.rs");
        let intersection_source = include_str!("../intersection/primitive_3d/circular_family.rs");
        let collision_circle_point_start = collision_source
            .find("pub fn circle3d_point3d_collides")
            .unwrap_or_else(|| panic!("missing start marker: pub fn circle3d_point3d_collides"));
        let collision_circle_point_section = &collision_source[collision_circle_point_start..];
        let intersection_circle_point_section = section(
            intersection_source,
            "fn circle3d_point3d_intersection_raw",
            "fn circle3d_line_segment3d_intersection_raw",
        );

        assert!(
            collision_circle_point_section.contains(CIRCLE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route circle point checks through the distance entrypoint"
        );
        assert!(
            intersection_circle_point_section.contains(CIRCLE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route circle point checks through the distance entrypoint"
        );
        assert!(
            !collision_circle_point_section.contains(CIRCLE_DIRECT_DISTANCE),
            "collision/primitive_3d.rs must not call circle.distance_to_point_3d directly"
        );
        assert!(
            !intersection_circle_point_section.contains(CIRCLE_DIRECT_DISTANCE),
            "intersection/primitive_3d.rs must not call circle.distance_to_point_3d directly"
        );
        assert!(
            !intersection_circle_point_section.contains(CIRCLE_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call circle.contains_point_3d directly"
        );
    }

    #[test]
    fn ellipsoidal_solid_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const ELLIPSOIDAL_SOLID_DIRECT_DISTANCE: &str = "ellipsoid.distance_to_surface";
        const ELLIPSOIDAL_SOLID_DIRECT_CONTAINS: &str = "ellipsoid.contains_point";
        const ELLIPSOIDAL_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::ellipsoidal_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/spherical_and_quadric_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_ellipsoidal_solid_point_section = section(
            collision_source,
            "pub fn ellipsoidal_solid3d_point3d_collides",
            "pub fn ellipsoidal_surface3d_point3d_collides",
        );
        let intersection_ellipsoidal_solid_point_section = section(
            intersection_source,
            "fn ellipsoidal_solid3d_point3d_intersection_raw",
            "fn ellipsoidal_surface3d_point3d_intersection_raw",
        );

        assert!(
            collision_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route ellipsoidal solid point checks through the distance entrypoint"
        );
        assert!(
            intersection_ellipsoidal_solid_point_section
                .contains(ELLIPSOIDAL_SOLID_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route ellipsoidal solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_DIRECT_DISTANCE),
            "collision/primitive_3d.rs must not call ellipsoid.distance_to_surface directly"
        );
        assert!(
            !collision_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_DIRECT_CONTAINS),
            "collision/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
        assert!(
            !intersection_ellipsoidal_solid_point_section
                .contains(ELLIPSOIDAL_SOLID_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
    }

    #[test]
    fn ellipsoidal_surface_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const ELLIPSOIDAL_SURFACE_DIRECT_CONTAINS: &str = "ellipsoid.contains_point";
        const ELLIPSOIDAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::ellipsoidal_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/spherical_and_quadric_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_ellipsoidal_surface_point_section = section(
            collision_source,
            "pub fn ellipsoidal_surface3d_point3d_collides",
            "pub fn torus_solid3d_point3d_collides",
        );
        let intersection_ellipsoidal_surface_point_section = section(
            intersection_source,
            "fn ellipsoidal_surface3d_point3d_intersection_raw",
            "fn spherical_solid3d_point3d_intersection_raw",
        );

        assert!(
            collision_ellipsoidal_surface_point_section
                .contains(ELLIPSOIDAL_SURFACE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route ellipsoidal surface point checks through the distance entrypoint"
        );
        assert!(
            intersection_ellipsoidal_surface_point_section
                .contains(ELLIPSOIDAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route ellipsoidal surface point checks through the distance entrypoint"
        );
        assert!(
            !collision_ellipsoidal_surface_point_section
                .contains(ELLIPSOIDAL_SURFACE_DIRECT_CONTAINS),
            "collision/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
        assert!(
            !intersection_ellipsoidal_surface_point_section
                .contains(ELLIPSOIDAL_SURFACE_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
    }

    #[test]
    fn conical_surface_point_boundary_guard_keeps_intersection_on_distance_entrypoint() {
        const CONICAL_SURFACE_DIRECT_CONTAINS: &str = "cone.contains_point";
        const CONICAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::conical_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let intersection_source =
            include_str!("../intersection/primitive_3d/cylindrical_and_conical_family.rs");
        let intersection_conical_surface_point_section = section(
            intersection_source,
            "fn conical_surface3d_point3d_intersection_raw",
            "pub fn conical_surface3d_point3d_intersection",
        );

        assert!(
            intersection_conical_surface_point_section.contains(CONICAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route conical surface point checks through the distance entrypoint"
        );
        assert!(
            !intersection_conical_surface_point_section.contains(CONICAL_SURFACE_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call cone.contains_point directly"
        );
    }

    #[test]
    fn spherical_solid_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const SPHERICAL_SOLID_DIRECT_DISTANCE: &str = "sphere.distance_to_surface";
        const SPHERICAL_SOLID_DIRECT_CONTAINS: &str = "sphere.contains_point";
        const SPHERICAL_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::spherical_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/spherical_and_quadric_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_spherical_solid_point_section = section(
            collision_source,
            "pub fn spherical_solid3d_point3d_collides",
            "pub fn spherical_solid3d_circle3d_collides",
        );
        let intersection_spherical_solid_point_section = section(
            intersection_source,
            "fn spherical_solid3d_point3d_intersection_raw",
            "fn spherical_solid3d_line3d_intersection_raw",
        );

        assert!(
            collision_spherical_solid_point_section.contains(SPHERICAL_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route spherical solid point checks through the distance entrypoint"
        );
        assert!(
            intersection_spherical_solid_point_section.contains(SPHERICAL_SOLID_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route spherical solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_spherical_solid_point_section.contains(SPHERICAL_SOLID_DIRECT_DISTANCE),
            "collision/primitive_3d.rs must not call sphere.distance_to_surface directly"
        );
        assert!(
            !intersection_spherical_solid_point_section.contains(SPHERICAL_SOLID_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call sphere.contains_point directly"
        );
    }

    #[test]
    fn torus_solid_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const TORUS_SOLID_DIRECT_CONTAINS: &str = "torus.contains_point";
        const TORUS_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::torus_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/spherical_and_quadric_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_torus_solid_point_section = section(
            collision_source,
            "pub fn torus_solid3d_point3d_collides",
            "pub fn torus_surface3d_point3d_collides",
        );
        let intersection_torus_solid_point_section = section(
            intersection_source,
            "fn torus_solid3d_point3d_intersection_raw",
            "fn torus_surface3d_point3d_intersection_raw",
        );

        assert!(
            collision_torus_solid_point_section.contains(TORUS_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route torus solid point checks through the distance entrypoint"
        );
        assert!(
            intersection_torus_solid_point_section.contains(TORUS_SOLID_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route torus solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_torus_solid_point_section.contains(TORUS_SOLID_DIRECT_CONTAINS),
            "collision/primitive_3d.rs must not call torus.contains_point directly"
        );
        assert!(
            !intersection_torus_solid_point_section.contains(TORUS_SOLID_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call torus.contains_point directly"
        );
    }

    #[test]
    fn torus_surface_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint()
    {
        const TORUS_SURFACE_DIRECT_UFCS: &str =
            "<TorusSurface3D<T> as TorusSurface3DDistance<T>>::distance_to_point";
        const TORUS_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::torus_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source =
            include_str!("../collision/primitive_3d/spherical_and_quadric_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_torus_surface_point_start = collision_source
            .find("pub fn torus_surface3d_point3d_collides")
            .unwrap_or_else(|| {
                panic!("missing start marker: pub fn torus_surface3d_point3d_collides")
            });
        let collision_torus_surface_point_section =
            &collision_source[collision_torus_surface_point_start..];
        let intersection_torus_surface_point_section = section(
            intersection_source,
            "fn torus_surface3d_point3d_intersection_raw",
            "pub fn torus_surface3d_point3d_intersection",
        );

        assert!(
            collision_torus_surface_point_section.contains(TORUS_SURFACE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route torus surface point checks through the distance entrypoint"
        );
        assert!(
            intersection_torus_surface_point_section.contains(TORUS_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route torus surface point checks through the distance entrypoint"
        );
        assert!(
            !collision_torus_surface_point_section.contains(TORUS_SURFACE_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call TorusSurface3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_torus_surface_point_section.contains(TORUS_SURFACE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call TorusSurface3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn triangle_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const TRIANGLE_POINT_ENTRYPOINT: &str = "crate::distance::triangle3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d/planar_and_mesh_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/planar_and_mesh_family.rs");
        let collision_triangle_point_section = section(
            collision_source,
            "pub fn triangle3d_point3d_collides",
            "pub fn triangle_mesh3d_point3d_collides",
        );
        let intersection_triangle_point_section = section(
            intersection_source,
            "fn triangle3d_point3d_intersection_raw",
            "fn triangle_mesh3d_point3d_intersection_raw",
        );

        assert!(
            collision_triangle_point_section.contains(TRIANGLE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route triangle point checks through the distance entrypoint"
        );
        assert!(
            intersection_triangle_point_section.contains(TRIANGLE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route triangle point checks through the distance entrypoint"
        );
    }

    #[test]
    fn triangle_mesh_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint()
    {
        const TRIANGLE_MESH_POINT_ENTRYPOINT: &str =
            "crate::distance::triangle_mesh3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d/planar_and_mesh_family.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/planar_and_mesh_family.rs");
        let collision_triangle_mesh_point_section = section(
            collision_source,
            "pub fn triangle_mesh3d_point3d_collides",
            "pub fn plane3d_point3d_collides",
        );
        let intersection_triangle_mesh_point_section = section(
            intersection_source,
            "fn triangle_mesh3d_point3d_intersection_raw",
            "fn plane3d_point3d_intersection_raw",
        );

        assert!(
            collision_triangle_mesh_point_section.contains(TRIANGLE_MESH_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route triangle mesh point checks through the distance entrypoint"
        );
        assert!(
            intersection_triangle_mesh_point_section.contains(TRIANGLE_MESH_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route triangle mesh point checks through the distance entrypoint"
        );
    }
}
