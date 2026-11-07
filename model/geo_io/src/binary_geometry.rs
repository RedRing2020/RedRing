//! 形状（三角形メッシュ・NURBS 曲面）の、リトルエンディアンのバイト列の読み書き
//!
//! ファイル全体の形（ヘッダ・ほかの項目）は使う側が決め、本モジュールは形状 1 つ分のバイト列だけを扱う。
//! 数値はすべてリトルエンディアンとする。
//!
//! - 三角形メッシュ: 頂点の数 `u32` と座標（`f64`×3）、三角形の数 `u32` と頂点の番号（`u32`×3）。頂点の法線は持たない
//! - NURBS 曲面: u・v の次数と制御点の数（`u32`×4）、u・v のノットの数 `u32` とノット（`f64`）、制御点の座標（u 方向の
//!   番号ごとに v 方向へ並べた `f64`×3）、重み（有無 `u8`（0: なし、1: あり）と、制御点と同じ並びの `f64`）
//!
//! 読み込みは信用できない入力として扱う。個数から確保する前に、残りのバイトの数で足りることを確かめる。

use std::fmt;

use geo_algorithms::{
    NurbsSurface3D, NurbsSurface3DConstructor, NurbsSurface3DProperties, Point3D, TriangleMesh3D,
    TriangleMesh3DConstructor, TriangleMesh3DProperties,
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
