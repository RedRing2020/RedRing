//! SVG形状ローダー（ViewModel層）
//!
//! geo_ioのSVGパーサーを使用して形状データを読み込み、
//! GPU用のVertexDataに変換します。

use crate::mesh_converter::VertexData;
use crate::shape_converter::{
    arc_to_wireframe_line_segments, circle_to_wireframe_line_segments,
    ellipse_arc_to_wireframe_line_segments, line_segment_to_vertices, triangle_to_solid_vertices,
    TessellationQuality,
};
use geo_contracts::{
    Arc3DConstructor, Circle3DConstructor, EllipseArc3DConstructor, LineSegment3DConstructor,
    NurbsCurve3DConstructor, NurbsCurve3DDerived, NurbsCurve3DEvaluation, NurbsCurve3DProperties,
    Triangle3DConstructor,
};
// geo_algorithms は geo_primitives を再エクスポート（アーキテクチャルール上許可）
use geo_algorithms::{Arc3D, Circle3D, EllipseArc3D, LineSegment3D, NurbsCurve3D, Triangle3D};
use geo_io::svg::{parse_svg_file, NurbsCurveData, SvgError, SvgShapeData};
use std::path::Path;
use thiserror::Error;

/// SVGローダーエラー
#[derive(Error, Debug)]
pub enum SvgLoaderError {
    #[error("SVG parsing error: {0}")]
    SvgError(#[from] SvgError),

    #[error("Shape construction error: {0}")]
    ConstructionError(String),

    #[error("Invalid shape data: {0}")]
    InvalidData(String),
}

/// SVGファイルを読み込み、GPU用頂点データに変換
///
/// # Arguments
/// * `path` - SVGファイルパス
/// * `tolerance` - テッセレーショントレランス（ミリメートル単位）
pub fn load_svg_shapes(path: &Path, tolerance: f64) -> Result<Vec<VertexData>, SvgLoaderError> {
    // 1. geo_ioでSVG解析
    let svg_data = parse_svg_file(path)?;

    // 2. 各形状をVertexDataに変換
    convert_svg_to_vertices(&svg_data, tolerance)
}

/// SvgShapeDataをVertexDataに変換
fn convert_svg_to_vertices(
    svg_data: &SvgShapeData,
    tolerance: f64,
) -> Result<Vec<VertexData>, SvgLoaderError> {
    let mut vertices = Vec::new();
    let quality = TessellationQuality::default();

    // 円形状を変換
    for circle_data in &svg_data.circles {
        let circle = Circle3D::new(circle_data.center, circle_data.normal, circle_data.radius)
            .ok_or_else(|| {
                SvgLoaderError::ConstructionError("Failed to create Circle3D".to_string())
            })?;

        vertices.extend(circle_to_wireframe_line_segments(&circle, &quality));
    }

    // 線分を変換
    for line_data in &svg_data.lines {
        let line = LineSegment3D::new(line_data.start, line_data.end).ok_or_else(|| {
            SvgLoaderError::ConstructionError("Failed to create LineSegment3D".to_string())
        })?;

        vertices.extend(line_segment_to_vertices(&line));
    }

    // 三角形を変換
    for tri_data in &svg_data.triangles {
        let triangle = Triangle3D::new(tri_data.p0, tri_data.p1, tri_data.p2).ok_or_else(|| {
            SvgLoaderError::ConstructionError("Failed to create Triangle3D".to_string())
        })?;

        vertices.extend(triangle_to_solid_vertices(&triangle));
    }

    // 円弧を変換
    for arc_data in &svg_data.arcs {
        let arc = Arc3D::new(
            arc_data.center,
            arc_data.radius,
            arc_data.normal,
            arc_data.start_direction,
            arc_data.start_angle,
            arc_data.end_angle,
        )
        .ok_or_else(|| SvgLoaderError::ConstructionError("Failed to create Arc3D".to_string()))?;

        vertices.extend(arc_to_wireframe_line_segments(&arc, &quality));
    }

    // 楕円弧を変換
    for arc_data in &svg_data.ellipse_arcs {
        let arc = EllipseArc3D::new(
            arc_data.center,
            arc_data.semi_major,
            arc_data.semi_minor,
            arc_data.normal,
            arc_data.major_axis_direction,
            arc_data.start_angle,
            arc_data.end_angle,
        )
        .ok_or_else(|| {
            SvgLoaderError::ConstructionError("Failed to create EllipseArc3D".to_string())
        })?;

        vertices.extend(ellipse_arc_to_wireframe_line_segments(&arc, &quality));
    }

    // NURBS曲線を変換
    for nurbs_data in &svg_data.nurbs_curves {
        vertices.extend(nurbs_curve_to_vertices(nurbs_data, tolerance)?);
    }

    Ok(vertices)
}

/// NURBS曲線をテッセレーション（線分分割）してVertexDataに変換
///
/// # Arguments
/// * `nurbs_data` - SVGから読み込んだNURBSデータ
/// * `tolerance` - 許容誤差（ミリメートル単位）
///
/// # テッセレーション戦略
/// - 曲線の全長を計算
/// - 全長÷トレランスで分割数を決定
/// - 均等パラメータ分割で頂点生成
fn nurbs_curve_to_vertices(
    nurbs_data: &NurbsCurveData,
    tolerance: f64,
) -> Result<Vec<VertexData>, SvgLoaderError> {
    // NURBS曲線を生成
    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        nurbs_data.degree,
        nurbs_data.knots.clone(),
        &nurbs_data.control_points,
        nurbs_data.weights.clone(),
    )
    .map_err(|e| {
        SvgLoaderError::ConstructionError(format!("Failed to create NurbsCurve3D: {:?}", e))
    })?;

    // パラメータ範囲を取得
    let (t_min, t_max) = curve.parameter_domain();

    // 曲線の全長を計算
    let arc_length = curve.length();

    // 分割数を決定（曲線長÷トレランス、最小10、最大10000）
    let num_segments = ((arc_length / tolerance).ceil() as usize).clamp(10, 10000);

    tracing::debug!(
        "NURBS tessellation: arc_length={:.3}, tolerance={:.3}, segments={}",
        arc_length,
        tolerance,
        num_segments
    );

    // 均等パラメータ分割で頂点生成
    let mut vertices = Vec::with_capacity(num_segments + 1);
    for i in 0..=num_segments {
        let t = t_min + (t_max - t_min) * (i as f64 / num_segments as f64);
        let point = curve
            .point_at_parameter(t)
            .expect("sample parameter should be within the curve domain");

        vertices.push(VertexData {
            position: [point.x() as f32, point.y() as f32, point.z() as f32],
            normal: [0.0, 0.0, 1.0], // Z軸正方向（SVGは2D平面）
        });
    }

    Ok(vertices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_svg_circle() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <circle cx="0" cy="0" r="5" />
        </svg>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(svg_content.as_bytes()).unwrap();

        let vertices = load_svg_shapes(temp_file.path(), 0.01).unwrap();
        assert!(!vertices.is_empty(), "Should have vertices for circle");
    }

    #[test]
    fn test_load_svg_line() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <line x1="-2" y1="0" x2="2" y2="0" />
        </svg>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(svg_content.as_bytes()).unwrap();

        let vertices = load_svg_shapes(temp_file.path(), 0.01).unwrap();
        assert_eq!(vertices.len(), 2, "Line should have 2 vertices");
    }

    #[test]
    fn test_load_svg_triangle() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <polygon points="0,0 1,0 0.5,1" />
        </svg>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(svg_content.as_bytes()).unwrap();

        let vertices = load_svg_shapes(temp_file.path(), 0.01).unwrap();
        assert_eq!(vertices.len(), 3, "Triangle should have 3 vertices");
    }

    #[test]
    fn test_load_svg_multiple_shapes() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <circle cx="0" cy="0" r="5" />
            <line x1="0" y1="0" x2="10" y2="10" />
            <polygon points="0,0 1,0 0.5,1" />
        </svg>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(svg_content.as_bytes()).unwrap();

        let vertices = load_svg_shapes(temp_file.path(), 0.01).unwrap();
        assert!(
            !vertices.is_empty(),
            "Should have vertices for multiple shapes"
        );
    }

    #[test]
    fn test_load_svg_nurbs_curve() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <path 
                data-nurbs="true"
                data-degree="3"
                data-control-points="0,0,0; 3,0,0; 3,3,0; 0,3,0"
                data-knots="0,0,0,0,1,1,1,1"
                data-weights="1,1,1,1" />
        </svg>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(svg_content.as_bytes()).unwrap();

        let vertices = load_svg_shapes(temp_file.path(), 0.01).unwrap();
        assert!(
            vertices.len() > 10,
            "NURBS curve should have many vertices from tessellation"
        );
    }

    #[test]
    fn test_load_fixture_svgs_with_path_arcs_and_lines() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/shapes");
        // arc.svg は円弧と楕円弧、clip_square.svg は直線の path
        for name in ["arc.svg", "clip_square.svg"] {
            let vertices = load_svg_shapes(&fixtures.join(name), 0.01).unwrap();
            assert!(!vertices.is_empty(), "{name} should have vertices");
        }
    }
}
