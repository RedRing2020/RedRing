//! ジョブの入力のファイル（ツールパス生成の入力・切削シミュレーションの入力。設計: JOB_MANAGER_STORAGE_DESIGN.md 5.2）
//!
//! 共通のヘッダ（種別 `ArtifactKind::CamSolverInput`・`ArtifactKind::CuttingSimulationInput`・版・単位・座標系）は
//! 成果物と同じ形式とし、中身は本モジュールが定義する。数値はすべてリトルエンディアン、文字列は長さ `u32` と
//! UTF-8 のバイト列、有無は `u8`（0: なし、1: あり）で表す。
//!
//! ツールパス生成の入力の中身: 加工の ID（文字列）、工具、加工の種類 `u8`（1: 走査線）と条件（ピックフィード・送り
//! `f64`、工具の移動範囲 `u8`、送り方 `u8`、走査線の角度 `f64`）、弦の許容誤差 `f64`、三角形に分ける上限（`u32`×2・
//! `u64`）、加工範囲（有無と長方形の `f64`×4）、形状。単位・座標系はヘッダに持つ。
//!
//! 切削シミュレーションの入力の中身: 素材の直方体（最小・最大の点の `f64`×6）、工具、素材の精度 `f64`、スナップショットの
//! 記録の間隔（種類 `u8` と値）、ツールパス生成のトレランス `f64`、製品形状。
//!
//! 工具: 種類 `u8`（1: フラット、2: ラジアス、3: ボール）、ID（文字列）、直径・角の半径 `f64`、工具長（有無と `f64`）。
//! 形状: 種類 `u8`（1: 三角形メッシュ、2: NURBS 曲面の集まり）。三角形メッシュは頂点の数 `u32` と座標（`f64`×3）、
//! 三角形の数 `u32` と頂点の番号（`u32`×3）。頂点の法線は持たない（ツールパス生成・切削シミュレーションで使わない）。NURBS 曲面の集まりは曲面の数 `u32` と、曲面ごとに u・v の次数と
//! 制御点の数（`u32`×4）、u・v のノットの数 `u32` とノット（`f64`）、制御点の座標（u 方向の番号ごとに v 方向へ並べた
//! `f64`×3）、重み（有無と、制御点と同じ並びの `f64`）。
//!
//! 読み込みは信用できない入力として扱う（`ARTIFACT_BINARY_IO_CONTRACT_DESIGN.md` の読み込みの決まり）。個数から確保する
//! 前に、残りのバイトの数で足りることを確かめる。三角形に分ける上限は、既定の値（`TessellationLimits::default`）を
//! 超える値を拒む（上限を外して、計算の量・メモリを際限なく増やさせないため）。

#[cfg(test)]
mod tests;

use std::io::{Read, Write};

use cam_algorithms::{CamSolverInput, SolverGeometry};
use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, BinaryFormatError, FORMAT_VERSION_MINOR_V1,
    MachiningBoundary, OperationSpec, RectangleBoundary, ScanlineCutDirection, ScanlineParams,
    ScanlineToolRange, Tool, ToolType, read_artifact_v1,
};
use geo_algorithms::{
    Aabb3D, NurbsSurface3D, NurbsSurface3DConstructor, NurbsSurface3DProperties, Point3D,
    TessellationLimits, TriangleMesh3D, TriangleMesh3DConstructor, TriangleMesh3DProperties,
};

use crate::binary_codec::{read_f64, read_u8, read_u32, read_u64, write_count};
use crate::{CuttingSimulationInput, SnapshotInterval};

/// 読み込める中身の版（共通のヘッダの minor）
const SUPPORTED_MINORS: &[u16] = &[FORMAT_VERSION_MINOR_V1];

const GEOMETRY_TRIANGLE_MESH: u8 = 1;
const GEOMETRY_NURBS_SURFACE_SET: u8 = 2;

/// ツールパス生成の入力を、ジョブの入力のファイルの形式で書く
pub fn write_cam_solver_input_v1<W: Write>(
    writer: &mut W,
    input: &CamSolverInput,
) -> Result<(), BinaryFormatError> {
    let mut payload = Vec::new();
    write_string(&mut payload, &input.operation_id, "operation_id")?;
    write_tool(&mut payload, &input.tool)?;
    let OperationSpec::Scanline(params) = input.operation;
    payload.push(1);
    write_f64(&mut payload, params.stepover);
    write_f64(&mut payload, params.feed_rate);
    payload.push(match params.tool_range {
        ScanlineToolRange::ExtendByToolRadius => 1,
        ScanlineToolRange::GeometryExtent => 2,
    });
    payload.push(match params.cut_direction {
        ScanlineCutDirection::DownCut => 1,
        ScanlineCutDirection::UpCut => 2,
        ScanlineCutDirection::ZigZag => 3,
    });
    write_f64(&mut payload, params.angle_degrees);
    write_f64(&mut payload, input.chord_tolerance);
    let limits = input.tessellation_limits;
    payload.extend_from_slice(&limits.max_subdivisions.to_le_bytes());
    payload.extend_from_slice(&limits.max_refinement_iterations.to_le_bytes());
    payload.extend_from_slice(&(limits.max_vertices_per_surface as u64).to_le_bytes());
    match input.boundary {
        None => payload.push(0),
        Some(MachiningBoundary::Rectangle(rectangle)) => {
            payload.push(1);
            for value in [
                rectangle.rect_min[0],
                rectangle.rect_min[1],
                rectangle.rect_max[0],
                rectangle.rect_max[1],
            ] {
                write_f64(&mut payload, value);
            }
        }
    }
    write_geometry(&mut payload, &input.geometry)?;

    let mut header = ArtifactHeaderV1::new(ArtifactKind::CamSolverInput, payload.len() as u64);
    header.unit = input.units;
    header.frame = input.coordinate_frame;
    header.write_to(writer)?;
    writer.write_all(&payload)?;
    Ok(())
}

/// ジョブの入力のファイルから、ツールパス生成の入力を読む
///
/// 条件の値の検証（`CamSolverInput::validate`）は、計算の入口で行う。
pub fn read_cam_solver_input_v1<R: Read>(
    reader: &mut R,
) -> Result<CamSolverInput, BinaryFormatError> {
    let (header, bytes) = read_input_payload(reader, ArtifactKind::CamSolverInput)?;
    let mut cursor = bytes.as_slice();
    let operation_id = read_string(&mut cursor, "operation_id")?;
    let tool = read_tool(&mut cursor)?;
    if read_u8(&mut cursor)? != 1 {
        return Err(BinaryFormatError::InvalidPayload("operation.kind"));
    }
    let stepover = read_f64(&mut cursor)?;
    let feed_rate = read_f64(&mut cursor)?;
    let tool_range = match read_u8(&mut cursor)? {
        1 => ScanlineToolRange::ExtendByToolRadius,
        2 => ScanlineToolRange::GeometryExtent,
        _ => return Err(BinaryFormatError::InvalidPayload("operation.tool_range")),
    };
    let cut_direction = match read_u8(&mut cursor)? {
        1 => ScanlineCutDirection::DownCut,
        2 => ScanlineCutDirection::UpCut,
        3 => ScanlineCutDirection::ZigZag,
        _ => return Err(BinaryFormatError::InvalidPayload("operation.cut_direction")),
    };
    let angle_degrees = read_f64(&mut cursor)?;
    let chord_tolerance = read_f64(&mut cursor)?;
    let tessellation_limits = read_tessellation_limits(&mut cursor)?;
    let boundary = match read_u8(&mut cursor)? {
        0 => None,
        1 => Some(MachiningBoundary::Rectangle(RectangleBoundary {
            rect_min: [read_f64(&mut cursor)?, read_f64(&mut cursor)?],
            rect_max: [read_f64(&mut cursor)?, read_f64(&mut cursor)?],
        })),
        _ => return Err(BinaryFormatError::InvalidPayload("boundary")),
    };
    let geometry = read_geometry(&mut cursor)?;
    ensure_consumed(cursor)?;
    Ok(CamSolverInput {
        operation_id,
        tool,
        geometry,
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover,
            feed_rate,
            tool_range,
            cut_direction,
            angle_degrees,
        }),
        units: header.unit,
        coordinate_frame: header.frame,
        chord_tolerance,
        tessellation_limits,
        boundary,
    })
}

/// 切削シミュレーションの入力を、ジョブの入力のファイルの形式で書く
pub fn write_cutting_simulation_input_v1<W: Write>(
    writer: &mut W,
    input: &CuttingSimulationInput,
) -> Result<(), BinaryFormatError> {
    let mut payload = Vec::new();
    for point in [input.stock.min(), input.stock.max()] {
        write_point(&mut payload, point);
    }
    write_tool(&mut payload, &input.tool)?;
    write_f64(&mut payload, input.precision);
    match input.snapshot_interval {
        SnapshotInterval::ByToolAndGrid => payload.push(1),
        SnapshotInterval::ByDistance {
            interval_mm,
            include_segment_endpoints,
        } => {
            payload.push(2);
            write_f64(&mut payload, interval_mm);
            payload.push(u8::from(include_segment_endpoints));
        }
        SnapshotInterval::ByAutoDistance {
            target_count,
            min_interval_mm,
            include_segment_endpoints,
        } => {
            payload.push(3);
            payload.extend_from_slice(&(target_count as u64).to_le_bytes());
            write_f64(&mut payload, min_interval_mm);
            payload.push(u8::from(include_segment_endpoints));
        }
    }
    write_f64(&mut payload, input.tolerance);
    write_geometry(&mut payload, &input.product)?;

    let header = ArtifactHeaderV1::new(ArtifactKind::CuttingSimulationInput, payload.len() as u64);
    header.write_to(writer)?;
    writer.write_all(&payload)?;
    Ok(())
}

/// ジョブの入力のファイルから、切削シミュレーションの入力を読む
///
/// 条件の値の検証（`cutting_simulation_input_errors` 等）は、計算の入口で行う。
pub fn read_cutting_simulation_input_v1<R: Read>(
    reader: &mut R,
) -> Result<CuttingSimulationInput, BinaryFormatError> {
    let (_, bytes) = read_input_payload(reader, ArtifactKind::CuttingSimulationInput)?;
    let mut cursor = bytes.as_slice();
    let stock = Aabb3D::new(read_point(&mut cursor)?, read_point(&mut cursor)?);
    let tool = read_tool(&mut cursor)?;
    let precision = read_f64(&mut cursor)?;
    let snapshot_interval = match read_u8(&mut cursor)? {
        1 => SnapshotInterval::ByToolAndGrid,
        2 => SnapshotInterval::ByDistance {
            interval_mm: read_f64(&mut cursor)?,
            include_segment_endpoints: read_bool(&mut cursor, "snapshot_interval")?,
        },
        3 => SnapshotInterval::ByAutoDistance {
            target_count: usize::try_from(read_u64(&mut cursor)?)
                .map_err(|_| BinaryFormatError::LengthOverflow("snapshot_interval.target_count"))?,
            min_interval_mm: read_f64(&mut cursor)?,
            include_segment_endpoints: read_bool(&mut cursor, "snapshot_interval")?,
        },
        _ => return Err(BinaryFormatError::InvalidPayload("snapshot_interval")),
    };
    let tolerance = read_f64(&mut cursor)?;
    let product = read_geometry(&mut cursor)?;
    ensure_consumed(cursor)?;
    Ok(CuttingSimulationInput {
        stock,
        tool,
        precision,
        snapshot_interval,
        product,
        tolerance,
    })
}

/// 三角形に分ける上限を読み、既定の値を超えないことを確かめる
fn read_tessellation_limits(cursor: &mut &[u8]) -> Result<TessellationLimits, BinaryFormatError> {
    let maximum = TessellationLimits::default();
    let max_subdivisions = read_u32(cursor)?;
    let max_refinement_iterations = read_u32(cursor)?;
    let max_vertices_per_surface = read_u64(cursor)?;
    if max_subdivisions > maximum.max_subdivisions
        || max_refinement_iterations > maximum.max_refinement_iterations
        || max_vertices_per_surface > maximum.max_vertices_per_surface as u64
    {
        return Err(BinaryFormatError::InvalidPayload("tessellation_limits"));
    }
    Ok(TessellationLimits {
        max_subdivisions,
        max_refinement_iterations,
        max_vertices_per_surface: max_vertices_per_surface as usize,
    })
}

/// 共通のヘッダを読み、種別 `kind` と版を確かめて、中身のバイト列を返す
fn read_input_payload<R: Read>(
    reader: &mut R,
    kind: ArtifactKind,
) -> Result<(ArtifactHeaderV1, Vec<u8>), BinaryFormatError> {
    let (header, payload) = read_artifact_v1(reader)?;
    let bytes = match (kind, payload) {
        (ArtifactKind::CamSolverInput, ArtifactPayload::CamSolverInput(bytes))
        | (ArtifactKind::CuttingSimulationInput, ArtifactPayload::CuttingSimulationInput(bytes)) => {
            bytes
        }
        _ => {
            return Err(BinaryFormatError::ArtifactKindMismatch {
                expected: kind,
                found: header.kind,
            });
        }
    };
    if !SUPPORTED_MINORS.contains(&header.version_minor) {
        return Err(BinaryFormatError::UnsupportedMinorVersion {
            major: header.version_major,
            found_minor: header.version_minor,
            supported_minors: SUPPORTED_MINORS,
        });
    }
    Ok((header, bytes))
}

fn ensure_consumed(cursor: &[u8]) -> Result<(), BinaryFormatError> {
    if cursor.is_empty() {
        Ok(())
    } else {
        Err(BinaryFormatError::InvalidPayload("trailing bytes"))
    }
}

fn write_tool(payload: &mut Vec<u8>, tool: &Tool<f64>) -> Result<(), BinaryFormatError> {
    payload.push(match tool.tool_type {
        ToolType::FlatEndMill => 1,
        ToolType::RadiusEndMill => 2,
        ToolType::BallEndMill => 3,
    });
    write_string(payload, &tool.id, "tool.id")?;
    write_f64(payload, tool.radius() * 2.0);
    write_f64(payload, tool.corner_radius());
    match tool.cutting_length {
        None => payload.push(0),
        Some(length) => {
            payload.push(1);
            write_f64(payload, length);
        }
    }
    Ok(())
}

fn read_tool(cursor: &mut &[u8]) -> Result<Tool<f64>, BinaryFormatError> {
    let tool_type = match read_u8(cursor)? {
        1 => ToolType::FlatEndMill,
        2 => ToolType::RadiusEndMill,
        3 => ToolType::BallEndMill,
        _ => return Err(BinaryFormatError::InvalidPayload("tool.type")),
    };
    let id = read_string(cursor, "tool.id")?;
    let diameter = read_f64(cursor)?;
    let corner_radius = read_f64(cursor)?;
    let cutting_length = match read_u8(cursor)? {
        0 => None,
        1 => Some(read_f64(cursor)?),
        _ => return Err(BinaryFormatError::InvalidPayload("tool.cutting_length")),
    };
    let mut tool = Tool::new(id, tool_type, diameter, corner_radius, 0.0);
    tool.cutting_length = cutting_length;
    Ok(tool)
}

fn write_geometry(
    payload: &mut Vec<u8>,
    geometry: &SolverGeometry,
) -> Result<(), BinaryFormatError> {
    match geometry {
        SolverGeometry::TriangleMesh(mesh) => {
            payload.push(GEOMETRY_TRIANGLE_MESH);
            write_count(payload, mesh.vertices().len(), "mesh.vertices")?;
            for &vertex in mesh.vertices() {
                write_point(payload, vertex);
            }
            write_count(payload, mesh.indices().len(), "mesh.triangles")?;
            for triangle in mesh.indices() {
                for &index in triangle {
                    write_count(payload, index, "mesh.index")?;
                }
            }
        }
        SolverGeometry::NurbsSurfaceSet(surfaces) => {
            payload.push(GEOMETRY_NURBS_SURFACE_SET);
            write_count(payload, surfaces.len(), "nurbs.surfaces")?;
            for surface in surfaces {
                write_nurbs_surface(payload, surface)?;
            }
        }
    }
    Ok(())
}

fn read_geometry(cursor: &mut &[u8]) -> Result<SolverGeometry, BinaryFormatError> {
    match read_u8(cursor)? {
        GEOMETRY_TRIANGLE_MESH => {
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
            TriangleMesh3D::new(vertices, indices)
                .map(SolverGeometry::TriangleMesh)
                .map_err(|_| BinaryFormatError::InvalidPayload("mesh"))
        }
        GEOMETRY_NURBS_SURFACE_SET => {
            // 曲面 1 つは、次数・制御点の数・ノットの数と重みの有無だけで 25 バイト以上
            let surface_count = read_count(cursor, 25, "nurbs.surfaces")?;
            let mut surfaces = Vec::with_capacity(surface_count);
            for _ in 0..surface_count {
                surfaces.push(read_nurbs_surface(cursor)?);
            }
            Ok(SolverGeometry::NurbsSurfaceSet(surfaces))
        }
        value => Err(BinaryFormatError::UnknownGeometryType(value)),
    }
}

fn write_nurbs_surface(
    payload: &mut Vec<u8>,
    surface: &NurbsSurface3D<f64>,
) -> Result<(), BinaryFormatError> {
    let (u_count, v_count) = (surface.u_count(), surface.v_count());
    for (value, field) in [
        (surface.u_degree(), "nurbs.u_degree"),
        (surface.v_degree(), "nurbs.v_degree"),
        (u_count, "nurbs.u_count"),
        (v_count, "nurbs.v_count"),
    ] {
        write_count(payload, value, field)?;
    }
    for (knots, field) in [
        (surface.u_knots(), "nurbs.u_knots"),
        (surface.v_knots(), "nurbs.v_knots"),
    ] {
        write_count(payload, knots.len(), field)?;
        for &knot in knots {
            write_f64(payload, knot);
        }
    }
    let grid = || (0..u_count).flat_map(move |u| (0..v_count).map(move |v| (u, v)));
    for (u, v) in grid() {
        let point = surface
            .control_point(u, v)
            .ok_or(BinaryFormatError::InvalidPayload("nurbs.control_point"))?;
        write_point(payload, point);
    }
    if surface.weights().is_some() {
        payload.push(1);
        for (u, v) in grid() {
            let weight = surface
                .weight(u, v)
                .ok_or(BinaryFormatError::InvalidPayload("nurbs.weight"))?;
            write_f64(payload, weight);
        }
    } else {
        payload.push(0);
    }
    Ok(())
}

fn read_nurbs_surface(cursor: &mut &[u8]) -> Result<NurbsSurface3D<f64>, BinaryFormatError> {
    let u_degree = read_u32(cursor)? as usize;
    let v_degree = read_u32(cursor)? as usize;
    let u_count = read_u32(cursor)? as usize;
    let v_count = read_u32(cursor)? as usize;
    let u_knots = read_f64_list(cursor, "nurbs.u_knots")?;
    let v_knots = read_f64_list(cursor, "nurbs.v_knots")?;
    let point_count = u_count
        .checked_mul(v_count)
        .ok_or(BinaryFormatError::LengthOverflow("nurbs.control_points"))?;
    ensure_remaining(cursor, point_count, 24, "nurbs.control_points")?;
    let mut control_points = Vec::with_capacity(u_count);
    for _ in 0..u_count {
        let mut row = Vec::with_capacity(v_count);
        for _ in 0..v_count {
            row.push(read_point(cursor)?);
        }
        control_points.push(row);
    }
    let weights = if read_bool(cursor, "nurbs.weights")? {
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
    } else {
        None
    };
    NurbsSurface3D::new(
        u_degree,
        v_degree,
        u_knots,
        v_knots,
        &control_points,
        weights,
    )
    .map_err(|_| BinaryFormatError::InvalidPayload("nurbs"))
}

fn read_f64_list(cursor: &mut &[u8], field: &'static str) -> Result<Vec<f64>, BinaryFormatError> {
    let count = read_count(cursor, 8, field)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_f64(cursor)?);
    }
    Ok(values)
}

fn write_point(payload: &mut Vec<u8>, point: Point3D<f64>) {
    for value in [point.x(), point.y(), point.z()] {
        write_f64(payload, value);
    }
}

fn read_point(cursor: &mut &[u8]) -> Result<Point3D<f64>, BinaryFormatError> {
    Ok(Point3D::new(
        read_f64(cursor)?,
        read_f64(cursor)?,
        read_f64(cursor)?,
    ))
}

fn write_f64(payload: &mut Vec<u8>, value: f64) {
    payload.extend_from_slice(&value.to_le_bytes());
}

/// 個数 `u32` を読み、1 つ `element_len` バイト以上の要素がその数だけ残りのバイトに収まることを確かめる
fn read_count(
    cursor: &mut &[u8],
    element_len: usize,
    field: &'static str,
) -> Result<usize, BinaryFormatError> {
    let count = read_u32(cursor)? as usize;
    ensure_remaining(cursor, count, element_len, field)?;
    Ok(count)
}

fn ensure_remaining(
    cursor: &[u8],
    count: usize,
    element_len: usize,
    field: &'static str,
) -> Result<(), BinaryFormatError> {
    match count.checked_mul(element_len) {
        Some(length) if length <= cursor.len() => Ok(()),
        _ => Err(BinaryFormatError::InvalidPayload(field)),
    }
}

fn write_string(
    payload: &mut Vec<u8>,
    value: &str,
    field: &'static str,
) -> Result<(), BinaryFormatError> {
    write_count(payload, value.len(), field)?;
    payload.extend_from_slice(value.as_bytes());
    Ok(())
}

fn read_string(cursor: &mut &[u8], field: &'static str) -> Result<String, BinaryFormatError> {
    let length = read_count(cursor, 1, field)?;
    let (bytes, rest) = cursor.split_at(length);
    *cursor = rest;
    String::from_utf8(bytes.to_vec()).map_err(|_| BinaryFormatError::InvalidPayload(field))
}

fn read_bool(cursor: &mut &[u8], field: &'static str) -> Result<bool, BinaryFormatError> {
    match read_u8(cursor)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(BinaryFormatError::InvalidPayload(field)),
    }
}
