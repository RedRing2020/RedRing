//! 形状（三角形メッシュ・NURBS 曲面）の、リトルエンディアンのバイト列の読み書き
//!
//! ファイル全体の形（ヘッダ・ほかの項目）は使う側が決め、本モジュールは形状 1 つ分のバイト列だけを扱う。
//! 数値はすべてリトルエンディアンとする。
//!
//! - 三角形メッシュ: 頂点の数 `u32` と座標（`f64`×3）、三角形の数 `u32` と頂点の番号（`u32`×3）。頂点の法線は持たない
//! - NURBS 曲面: u・v の次数と制御点の数（`u32`×4）、u・v のノットの数 `u32` とノット（`f64`）、制御点の座標（u 方向の
//!   番号ごとに v 方向へ並べた `f64`×3）、重み（有無 `u8`（0: なし、1: あり）と、制御点と同じ並びの `f64`）
//! - NURBS 曲線: 次数と制御点の数（`u32`×2）、ノットの数 `u32` とノット（`f64`）、制御点の座標（`f64`×3）、重み（有無
//!   `u8` と、制御点の数の `f64`）
//! - 解析的な曲線・曲面（点・方向は `f64`×3、長さ・角度は `f64`。角度はラジアン）
//!   - 直線: 通る点・方向
//!   - 円: 中心・軸・角度 0 の方向・半径
//!   - 楕円: 中心・法線・長軸の方向・長半径・短半径
//!   - 平面: 原点・法線・u の軸
//!   - 円柱: 中心・軸・角度 0 の方向・半径
//!   - 円錐: 中心・軸・角度 0 の方向・半径・半頂角
//!   - 球: 中心・軸・角度 0 の方向・半径
//!   - トーラス: 中心・軸・角度 0 の方向・中心の円の半径・管の半径
//!
//! 読み込みは信用できない入力として扱う。個数から確保する前に、残りのバイトの数で足りることを確かめる。NURBS
//! 曲線と解析的な曲線・曲面の値は、有限の値でなければ、または形を作れなければ（大きさ 0 の方向・正でない半径など）
//! 誤りにする。

use std::fmt;

use geo_algorithms::{
    Angle, Circle3D, ConicalSurface3D, CylindricalSurface3D, Direction3D, Ellipse3D,
    InfiniteLine3D, NurbsCurve3D, NurbsSurface3D, NurbsSurface3DConstructor,
    NurbsSurface3DProperties, Plane3D, Point3D, SphericalSurface3D, TorusSurface3D, TriangleMesh3D,
    TriangleMesh3DConstructor, TriangleMesh3DProperties, Vector3D,
};
use geo_contracts::{
    Circle3DConstructor, Circle3DProperties, ConicalSurface3DConstructor,
    ConicalSurface3DProperties, CylindricalSurface3DConstructor, CylindricalSurface3DProperties,
    Ellipse3DConstructor, Ellipse3DProperties, InfiniteLine3DConstructor, InfiniteLine3DProperties,
    NurbsCurve3DConstructor, NurbsCurve3DProperties, Plane3DConstructor, Plane3DProperties,
    SphericalSurface3DConstructor, SphericalSurface3DProperties, TorusSurface3DConstructor,
    TorusSurface3DProperties,
};

#[cfg(test)]
mod tests;

/// 形状のバイト列の読み書きの誤り
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryGeometryError {
    /// バイト列が途中で終わっている
    UnexpectedEnd,
    /// 個数・長さが収まらない（書くときは `u32` に、読むときは確保できる大きさに。項目の名前）
    LengthOverflow(&'static str),
    /// 値が正しくない（読むとき。項目の名前）
    InvalidValue(&'static str),
}

impl fmt::Display for BinaryGeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "unexpected end of the geometry bytes"),
            Self::LengthOverflow(field) => write!(f, "length overflow: {field}"),
            Self::InvalidValue(field) => write!(f, "invalid geometry value: {field}"),
        }
    }
}

impl std::error::Error for BinaryGeometryError {}

/// 三角形メッシュを書く（頂点の法線は書かない）
pub fn write_triangle_mesh(
    out: &mut Vec<u8>,
    mesh: &TriangleMesh3D<f64>,
) -> Result<(), BinaryGeometryError> {
    write_count(out, mesh.vertices().len(), "mesh.vertices")?;
    for &vertex in mesh.vertices() {
        write_point(out, vertex);
    }
    write_count(out, mesh.indices().len(), "mesh.triangles")?;
    for triangle in mesh.indices() {
        for &index in triangle {
            write_count(out, index, "mesh.index")?;
        }
    }
    Ok(())
}

/// 三角形メッシュを読み、`cursor` を読んだ後ろへ進める
pub fn read_triangle_mesh(cursor: &mut &[u8]) -> Result<TriangleMesh3D<f64>, BinaryGeometryError> {
    let vertex_count = read_count(cursor, 24, "mesh.vertices")?;
    let mut vertices = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        vertices.push(read_point(cursor)?);
    }
    let triangle_count = read_count(cursor, 12, "mesh.triangles")?;
    let mut indices = Vec::with_capacity(triangle_count);
    for _ in 0..triangle_count {
        let mut triangle = [0_usize; 3];
        for index in &mut triangle {
            *index = read_u32(cursor)? as usize;
        }
        indices.push(triangle);
    }
    TriangleMesh3D::new(vertices, indices).map_err(|_| BinaryGeometryError::InvalidValue("mesh"))
}

/// NURBS 曲面を書く
pub fn write_nurbs_surface(
    out: &mut Vec<u8>,
    surface: &NurbsSurface3D<f64>,
) -> Result<(), BinaryGeometryError> {
    let (u_count, v_count) = (surface.u_count(), surface.v_count());
    for (value, field) in [
        (surface.u_degree(), "nurbs.u_degree"),
        (surface.v_degree(), "nurbs.v_degree"),
        (u_count, "nurbs.u_count"),
        (v_count, "nurbs.v_count"),
    ] {
        write_count(out, value, field)?;
    }
    for (knots, field) in [
        (surface.u_knots(), "nurbs.u_knots"),
        (surface.v_knots(), "nurbs.v_knots"),
    ] {
        write_count(out, knots.len(), field)?;
        for &knot in knots {
            write_f64(out, knot);
        }
    }
    let grid = || (0..u_count).flat_map(move |u| (0..v_count).map(move |v| (u, v)));
    for (u, v) in grid() {
        let point = surface
            .control_point(u, v)
            .ok_or(BinaryGeometryError::InvalidValue("nurbs.control_point"))?;
        write_point(out, point);
    }
    if surface.weights().is_some() {
        out.push(1);
        for (u, v) in grid() {
            let weight = surface
                .weight(u, v)
                .ok_or(BinaryGeometryError::InvalidValue("nurbs.weight"))?;
            write_f64(out, weight);
        }
    } else {
        out.push(0);
    }
    Ok(())
}

/// NURBS 曲面 1 つの最小のバイトの数（次数と制御点の数 `u32`×4・ノットの数 `u32`×2・重みの有無 `u8`）
pub const NURBS_SURFACE_MIN_LEN: usize = 4 * 4 + 4 * 2 + 1;

/// NURBS 曲面を読み、`cursor` を読んだ後ろへ進める
pub fn read_nurbs_surface(cursor: &mut &[u8]) -> Result<NurbsSurface3D<f64>, BinaryGeometryError> {
    let u_degree = read_u32(cursor)? as usize;
    let v_degree = read_u32(cursor)? as usize;
    let u_count = read_u32(cursor)? as usize;
    let v_count = read_u32(cursor)? as usize;
    let u_knots = read_f64_list(cursor, "nurbs.u_knots")?;
    let v_knots = read_f64_list(cursor, "nurbs.v_knots")?;
    // 片方の数が 0 だと、もう片方の数が大きくても制御点の数が 0 になり、残りのバイトの確かめを抜けてしまう
    if u_count == 0 || v_count == 0 {
        return Err(BinaryGeometryError::InvalidValue("nurbs.control_points"));
    }
    let point_count = u_count
        .checked_mul(v_count)
        .ok_or(BinaryGeometryError::LengthOverflow("nurbs.control_points"))?;
    ensure_remaining(cursor, point_count, 24, "nurbs.control_points")?;
    let mut control_points = Vec::with_capacity(u_count);
    for _ in 0..u_count {
        let mut row = Vec::with_capacity(v_count);
        for _ in 0..v_count {
            row.push(read_point(cursor)?);
        }
        control_points.push(row);
    }
    let weights = match read_u8(cursor)? {
        0 => None,
        1 => {
            ensure_remaining(cursor, point_count, 8, "nurbs.weights")?;
            let mut grid = Vec::with_capacity(u_count);
            for _ in 0..u_count {
                let mut row = Vec::with_capacity(v_count);
                for _ in 0..v_count {
                    row.push(read_f64(cursor)?);
                }
                grid.push(row);
            }
            Some(grid)
        }
        _ => return Err(BinaryGeometryError::InvalidValue("nurbs.weights")),
    };
    NurbsSurface3D::new(
        u_degree,
        v_degree,
        u_knots,
        v_knots,
        &control_points,
        weights,
    )
    .map_err(|_| BinaryGeometryError::InvalidValue("nurbs"))
}

/// NURBS 曲線を書く
pub fn write_nurbs_curve(
    out: &mut Vec<u8>,
    curve: &NurbsCurve3D<f64>,
) -> Result<(), BinaryGeometryError> {
    let count = curve.control_point_count();
    write_count(out, curve.degree(), "nurbs_curve.degree")?;
    write_count(out, count, "nurbs_curve.count")?;
    let knots = <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::knot_vector(curve);
    write_count(out, knots.len(), "nurbs_curve.knots")?;
    for &knot in knots {
        write_f64(out, knot);
    }
    for index in 0..count {
        let point = curve
            .control_point(index)
            .ok_or(BinaryGeometryError::InvalidValue(
                "nurbs_curve.control_point",
            ))?;
        write_point(out, point);
    }
    match <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::weights(curve) {
        Some(weights) => {
            out.push(1);
            for &weight in weights {
                write_f64(out, weight);
            }
        }
        None => out.push(0),
    }
    Ok(())
}

/// NURBS 曲線を読み、`cursor` を読んだ後ろへ進める
pub fn read_nurbs_curve(cursor: &mut &[u8]) -> Result<NurbsCurve3D<f64>, BinaryGeometryError> {
    let degree = read_u32(cursor)? as usize;
    let count = read_u32(cursor)? as usize;
    let knots = read_f64_list(cursor, "nurbs_curve.knots")?;
    if knots.iter().any(|knot| !knot.is_finite()) {
        return Err(BinaryGeometryError::InvalidValue("nurbs_curve.knots"));
    }
    if count == 0 {
        return Err(BinaryGeometryError::InvalidValue(
            "nurbs_curve.control_points",
        ));
    }
    ensure_remaining(cursor, count, 24, "nurbs_curve.control_points")?;
    let mut control_points = Vec::with_capacity(count);
    for _ in 0..count {
        control_points.push(read_finite_point(cursor, "nurbs_curve.control_points")?);
    }
    let weights = match read_u8(cursor)? {
        0 => None,
        1 => {
            ensure_remaining(cursor, count, 8, "nurbs_curve.weights")?;
            let mut weights = Vec::with_capacity(count);
            for _ in 0..count {
                weights.push(read_finite(cursor, "nurbs_curve.weights")?);
            }
            Some(weights)
        }
        _ => return Err(BinaryGeometryError::InvalidValue("nurbs_curve.weights")),
    };
    <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        degree,
        knots,
        &control_points,
        weights,
    )
    .map_err(|_| BinaryGeometryError::InvalidValue("nurbs_curve"))
}

/// 直線を書く
pub fn write_line(out: &mut Vec<u8>, line: &InfiniteLine3D<f64>) {
    write_point(out, line.point());
    write_direction(out, line.direction());
}

/// 直線を読み、`cursor` を読んだ後ろへ進める
pub fn read_line(cursor: &mut &[u8]) -> Result<InfiniteLine3D<f64>, BinaryGeometryError> {
    let point = read_finite_point(cursor, "line.point")?;
    let direction = read_vector(cursor, "line.direction")?;
    <InfiniteLine3D<f64> as InfiniteLine3DConstructor<f64>>::new(point, direction)
        .ok_or(BinaryGeometryError::InvalidValue("line"))
}

/// 円を書く
pub fn write_circle(out: &mut Vec<u8>, circle: &Circle3D<f64>) {
    write_point(out, circle.center());
    write_direction(out, circle.axis());
    write_direction(out, circle.ref_direction());
    write_f64(out, circle.radius());
}

/// 円を読み、`cursor` を読んだ後ろへ進める
pub fn read_circle(cursor: &mut &[u8]) -> Result<Circle3D<f64>, BinaryGeometryError> {
    let center = read_finite_point(cursor, "circle.center")?;
    let axis = read_direction(cursor, "circle.axis")?;
    let reference = read_direction(cursor, "circle.ref_direction")?;
    let radius = read_finite(cursor, "circle.radius")?;
    <Circle3D<f64> as Circle3DConstructor<f64>>::from_frame(center, axis, reference, radius)
        .ok_or(BinaryGeometryError::InvalidValue("circle"))
}

/// 楕円を書く
pub fn write_ellipse(out: &mut Vec<u8>, ellipse: &Ellipse3D<f64>) {
    write_point(out, ellipse.center());
    write_direction(out, ellipse.normal());
    write_direction(out, ellipse.major_axis_direction());
    write_f64(out, ellipse.semi_major_axis());
    write_f64(out, ellipse.semi_minor_axis());
}

/// 楕円を読み、`cursor` を読んだ後ろへ進める
pub fn read_ellipse(cursor: &mut &[u8]) -> Result<Ellipse3D<f64>, BinaryGeometryError> {
    let center = read_finite_point(cursor, "ellipse.center")?;
    let normal = read_direction(cursor, "ellipse.normal")?;
    let major = read_direction(cursor, "ellipse.major_axis")?;
    let semi_major = read_finite(cursor, "ellipse.semi_major_axis")?;
    let semi_minor = read_finite(cursor, "ellipse.semi_minor_axis")?;
    <Ellipse3D<f64> as Ellipse3DConstructor<f64>>::new(
        center, semi_major, semi_minor, normal, major,
    )
    .ok_or(BinaryGeometryError::InvalidValue("ellipse"))
}

/// 平面を書く
pub fn write_plane(out: &mut Vec<u8>, plane: &Plane3D<f64>) {
    write_point(out, plane.origin());
    write_direction(out, plane.normal());
    write_direction(out, plane.u_axis());
}

/// 平面を読み、`cursor` を読んだ後ろへ進める
pub fn read_plane(cursor: &mut &[u8]) -> Result<Plane3D<f64>, BinaryGeometryError> {
    let origin = read_finite_point(cursor, "plane.origin")?;
    let normal = read_direction(cursor, "plane.normal")?;
    let u_axis = read_direction(cursor, "plane.u_axis")?;
    <Plane3D<f64> as Plane3DConstructor<f64>>::from_origin_and_axes(origin, normal, u_axis)
        .ok_or(BinaryGeometryError::InvalidValue("plane"))
}

/// 円柱を書く
pub fn write_cylinder(out: &mut Vec<u8>, cylinder: &CylindricalSurface3D<f64>) {
    write_point(out, cylinder.center());
    write_direction(out, cylinder.axis());
    write_direction(out, cylinder.ref_direction());
    write_f64(out, cylinder.radius());
}

/// 円柱を読み、`cursor` を読んだ後ろへ進める
pub fn read_cylinder(cursor: &mut &[u8]) -> Result<CylindricalSurface3D<f64>, BinaryGeometryError> {
    let (center, axis, reference) = read_frame(
        cursor,
        ["cylinder.center", "cylinder.axis", "cylinder.ref_direction"],
    )?;
    let radius = read_finite(cursor, "cylinder.radius")?;
    <CylindricalSurface3D<f64> as CylindricalSurface3DConstructor<f64>>::new(
        center, axis, reference, radius,
    )
    .ok_or(BinaryGeometryError::InvalidValue("cylinder"))
}

/// 円錐を書く
pub fn write_cone(out: &mut Vec<u8>, cone: &ConicalSurface3D<f64>) {
    write_point(out, cone.center());
    write_direction(out, cone.axis());
    write_direction(out, cone.ref_direction());
    write_f64(out, cone.radius());
    write_f64(out, cone.semi_angle().to_radians());
}

/// 円錐を読み、`cursor` を読んだ後ろへ進める
pub fn read_cone(cursor: &mut &[u8]) -> Result<ConicalSurface3D<f64>, BinaryGeometryError> {
    let (center, axis, reference) =
        read_frame(cursor, ["cone.center", "cone.axis", "cone.ref_direction"])?;
    let radius = read_finite(cursor, "cone.radius")?;
    let semi_angle = read_finite(cursor, "cone.semi_angle")?;
    <ConicalSurface3D<f64> as ConicalSurface3DConstructor<f64>>::new(
        center,
        axis,
        reference,
        radius,
        Angle::from_radians(semi_angle),
    )
    .ok_or(BinaryGeometryError::InvalidValue("cone"))
}

/// 球を書く
pub fn write_sphere(out: &mut Vec<u8>, sphere: &SphericalSurface3D<f64>) {
    write_point(out, sphere.center());
    write_direction(out, sphere.axis());
    write_direction(out, sphere.ref_direction());
    write_f64(out, sphere.radius());
}

/// 球を読み、`cursor` を読んだ後ろへ進める
pub fn read_sphere(cursor: &mut &[u8]) -> Result<SphericalSurface3D<f64>, BinaryGeometryError> {
    let (center, axis, reference) = read_frame(
        cursor,
        ["sphere.center", "sphere.axis", "sphere.ref_direction"],
    )?;
    let radius = read_finite(cursor, "sphere.radius")?;
    <SphericalSurface3D<f64> as SphericalSurface3DConstructor<f64>>::new(
        center, axis, reference, radius,
    )
    .ok_or(BinaryGeometryError::InvalidValue("sphere"))
}

/// トーラスを書く
pub fn write_torus(out: &mut Vec<u8>, torus: &TorusSurface3D<f64>) {
    write_point(out, torus.center());
    write_direction(out, torus.axis());
    write_direction(out, torus.ref_direction());
    write_f64(out, torus.major_radius());
    write_f64(out, torus.minor_radius());
}

/// トーラスを読み、`cursor` を読んだ後ろへ進める
pub fn read_torus(cursor: &mut &[u8]) -> Result<TorusSurface3D<f64>, BinaryGeometryError> {
    let (center, axis, reference) = read_frame(
        cursor,
        ["torus.center", "torus.axis", "torus.ref_direction"],
    )?;
    let major = read_finite(cursor, "torus.major_radius")?;
    let minor = read_finite(cursor, "torus.minor_radius")?;
    <TorusSurface3D<f64> as TorusSurface3DConstructor<f64>>::new(
        center, axis, reference, major, minor,
    )
    .ok_or(BinaryGeometryError::InvalidValue("torus"))
}

/// 中心・軸・角度 0 の方向
type Frame = (Point3D<f64>, Direction3D<f64>, Direction3D<f64>);

/// 中心・軸・角度 0 の方向を読む（`fields` は、それぞれの項目の名前）
fn read_frame(
    cursor: &mut &[u8],
    [center, axis, reference]: [&'static str; 3],
) -> Result<Frame, BinaryGeometryError> {
    Ok((
        read_finite_point(cursor, center)?,
        read_direction(cursor, axis)?,
        read_direction(cursor, reference)?,
    ))
}

fn write_direction(out: &mut Vec<u8>, direction: Direction3D<f64>) {
    for value in [direction.x(), direction.y(), direction.z()] {
        write_f64(out, value);
    }
}

/// 有限の値の点を読む
fn read_finite_point(
    cursor: &mut &[u8],
    field: &'static str,
) -> Result<Point3D<f64>, BinaryGeometryError> {
    let point = read_point(cursor)?;
    if point.x().is_finite() && point.y().is_finite() && point.z().is_finite() {
        Ok(point)
    } else {
        Err(BinaryGeometryError::InvalidValue(field))
    }
}

/// 有限の値のベクトルを読む
fn read_vector(
    cursor: &mut &[u8],
    field: &'static str,
) -> Result<Vector3D<f64>, BinaryGeometryError> {
    let point = read_finite_point(cursor, field)?;
    Ok(Vector3D::new(point.x(), point.y(), point.z()))
}

/// 方向を読む（大きさ 0 なら誤り）
fn read_direction(
    cursor: &mut &[u8],
    field: &'static str,
) -> Result<Direction3D<f64>, BinaryGeometryError> {
    Direction3D::from_vector(read_vector(cursor, field)?)
        .ok_or(BinaryGeometryError::InvalidValue(field))
}

/// 有限の値を読む
fn read_finite(cursor: &mut &[u8], field: &'static str) -> Result<f64, BinaryGeometryError> {
    let value = read_f64(cursor)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(BinaryGeometryError::InvalidValue(field))
    }
}

fn read_f64_list(cursor: &mut &[u8], field: &'static str) -> Result<Vec<f64>, BinaryGeometryError> {
    let count = read_count(cursor, 8, field)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_f64(cursor)?);
    }
    Ok(values)
}

fn write_point(out: &mut Vec<u8>, point: Point3D<f64>) {
    for value in [point.x(), point.y(), point.z()] {
        write_f64(out, value);
    }
}

fn read_point(cursor: &mut &[u8]) -> Result<Point3D<f64>, BinaryGeometryError> {
    Ok(Point3D::new(
        read_f64(cursor)?,
        read_f64(cursor)?,
        read_f64(cursor)?,
    ))
}

fn write_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_count(
    out: &mut Vec<u8>,
    count: usize,
    field: &'static str,
) -> Result<(), BinaryGeometryError> {
    let count = u32::try_from(count).map_err(|_| BinaryGeometryError::LengthOverflow(field))?;
    out.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

/// 個数 `u32` を読み、1 つ `element_len` バイト以上の要素がその数だけ残りのバイトに収まることを確かめる
fn read_count(
    cursor: &mut &[u8],
    element_len: usize,
    field: &'static str,
) -> Result<usize, BinaryGeometryError> {
    let count = read_u32(cursor)? as usize;
    ensure_remaining(cursor, count, element_len, field)?;
    Ok(count)
}

fn ensure_remaining(
    cursor: &[u8],
    count: usize,
    element_len: usize,
    field: &'static str,
) -> Result<(), BinaryGeometryError> {
    match count.checked_mul(element_len) {
        Some(length) if length <= cursor.len() => Ok(()),
        _ => Err(BinaryGeometryError::InvalidValue(field)),
    }
}

fn take<const N: usize>(cursor: &mut &[u8]) -> Result<[u8; N], BinaryGeometryError> {
    let (bytes, rest) = cursor
        .split_first_chunk::<N>()
        .ok_or(BinaryGeometryError::UnexpectedEnd)?;
    *cursor = rest;
    Ok(*bytes)
}

fn read_u8(cursor: &mut &[u8]) -> Result<u8, BinaryGeometryError> {
    Ok(take::<1>(cursor)?[0])
}

fn read_u32(cursor: &mut &[u8]) -> Result<u32, BinaryGeometryError> {
    Ok(u32::from_le_bytes(take(cursor)?))
}

fn read_f64(cursor: &mut &[u8]) -> Result<f64, BinaryGeometryError> {
    Ok(f64::from_le_bytes(take(cursor)?))
}
