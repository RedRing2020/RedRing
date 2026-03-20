//! SVG形状ローダー（ViewModel層）
//!
//! geo_ioのSVGパーサーを使用して形状データを読み込み、
//! GPU用のVertexDataに変換します。

use crate::mesh_converter::VertexData;
use crate::shape_converter::{
    arc_to_wireframe_line_segments, circle_to_wireframe_line_segments, line_segment_to_vertices,
    triangle_to_solid_vertices, TessellationQuality,
};
// geo_algorithms は geo_primitives を再エクスポート（アーキテクチャルール上許可）
use geo_algorithms::{
    Arc3D, Circle3D, Direction3D, LineSegment3D, NurbsCurve3D, Point3D, Triangle3D, Vector3D,
};
use geo_contracts::{Angle, NurbsCurve3DConstructor, NurbsCurve3DMeasure};
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
        let center = Point3D::<f64>::new(
            circle_data.center.0,
            circle_data.center.1,
            circle_data.center.2,
        );
        let normal = Direction3D::from_vector(Vector3D::new(
            circle_data.normal.0,
            circle_data.normal.1,
            circle_data.normal.2,
        ))
        .ok_or_else(|| SvgLoaderError::InvalidData("Invalid circle normal vector".to_string()))?;

        let circle = Circle3D::new(center, normal, circle_data.radius).ok_or_else(|| {
            SvgLoaderError::ConstructionError("Failed to create Circle3D".to_string())
        })?;

        vertices.extend(circle_to_wireframe_line_segments(&circle, &quality));
    }

    // 線分を変換
    for line_data in &svg_data.lines {
        let start = Point3D::<f64>::new(line_data.start.0, line_data.start.1, line_data.start.2);
        let end = Point3D::<f64>::new(line_data.end.0, line_data.end.1, line_data.end.2);

        let line = LineSegment3D::new(start, end).ok_or_else(|| {
            SvgLoaderError::ConstructionError("Failed to create LineSegment3D".to_string())
        })?;

        vertices.extend(line_segment_to_vertices(&line));
    }

    // 三角形を変換
    for tri_data in &svg_data.triangles {
        let p0 = Point3D::<f64>::new(tri_data.p0.0, tri_data.p0.1, tri_data.p0.2);
        let p1 = Point3D::<f64>::new(tri_data.p1.0, tri_data.p1.1, tri_data.p1.2);
        let p2 = Point3D::<f64>::new(tri_data.p2.0, tri_data.p2.1, tri_data.p2.2);

        let triangle = Triangle3D::new(p0, p1, p2).ok_or_else(|| {
            SvgLoaderError::ConstructionError("Failed to create Triangle3D".to_string())
        })?;

        vertices.extend(triangle_to_solid_vertices(&triangle));
    }

    // 円弧を変換（TODO: SVGパーサーでarc抽出実装後）
    for arc_data in &svg_data.arcs {
        let center = Point3D::<f64>::new(arc_data.center.0, arc_data.center.1, arc_data.center.2);
        let normal = Direction3D::from_vector(Vector3D::new(
            arc_data.normal.0,
            arc_data.normal.1,
            arc_data.normal.2,
        ))
        .ok_or_else(|| SvgLoaderError::InvalidData("Invalid arc normal vector".to_string()))?;

        let start_dir = Direction3D::from_vector(Vector3D::new(
            arc_data.start_direction.0,
            arc_data.start_direction.1,
            arc_data.start_direction.2,
        ))
        .ok_or_else(|| SvgLoaderError::InvalidData("Invalid arc start direction".to_string()))?;

        let start_angle = Angle::from_radians(arc_data.start_angle);
        let end_angle = Angle::from_radians(arc_data.end_angle);

        let arc = Arc3D::new(
            center,
            arc_data.radius,
            normal,
            start_dir,
            start_angle,
            end_angle,
        )
        .ok_or_else(|| SvgLoaderError::ConstructionError("Failed to create Arc3D".to_string()))?;

        vertices.extend(arc_to_wireframe_line_segments(&arc, &quality));
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
        nurbs_data.control_points.clone(),
        nurbs_data.weights.clone(),
    )
    .map_err(|e| {
        SvgLoaderError::ConstructionError(format!("Failed to create NurbsCurve3D: {:?}", e))
    })?;

    // パラメータ範囲を取得
    let (t_min, t_max) = curve.parameter_domain();

    // 曲線の全長を計算
    let arc_length = curve.arc_length_total(tolerance);

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
        let point = curve.evaluate_at(t);

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
}
