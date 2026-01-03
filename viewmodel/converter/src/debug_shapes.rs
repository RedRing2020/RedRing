//! デバッグ用形状生成モジュール
//!
//! View層がModel層に直接依存しないよう、ViewModel層で形状を生成します。
//! テスト・デバッグ用の簡単な形状データを提供します。

use geo_primitives::{Arc3D, Circle3D, Direction3D, LineSegment3D, Point3D, Triangle3D, Vector3D};

use crate::mesh_converter::VertexData;
use crate::shape_converter::{
    arc_to_wireframe_line_segments, circle_to_wireframe_line_segments, line_segment_to_vertices,
    triangle_to_solid_vertices, TessellationQuality,
};

/// デバッグ用線分を生成（-2.0 to 2.0 の水平線）
pub fn create_debug_line_segment() -> Vec<VertexData> {
    let start = Point3D::<f64>::new(-2.0, 0.0, 0.0);
    let end = Point3D::<f64>::new(2.0, 0.0, 0.0);
    let line = LineSegment3D::new(start, end).unwrap();
    line_segment_to_vertices(&line)
}

/// デバッグ用円を生成（XY平面、半径5.0）
pub fn create_debug_circle() -> Vec<VertexData> {
    let center = Point3D::<f64>::origin();
    let normal = Direction3D::from_vector(Vector3D::<f64>::new(0.0, 0.0, 1.0))
        .expect("Valid normal direction");
    let radius = 5.0;
    let circle = Circle3D::new(center, normal, radius).unwrap();
    let quality = TessellationQuality::default();
    circle_to_wireframe_line_segments(&circle, &quality)
}

/// デバッグ用三角形を生成（XY平面、0.0-1.0範囲）
pub fn create_debug_triangle() -> Vec<VertexData> {
    let p0 = Point3D::<f64>::new(0.0, 0.0, 0.0);
    let p1 = Point3D::<f64>::new(1.0, 0.0, 0.0);
    let p2 = Point3D::<f64>::new(0.5, 1.0, 0.0);
    let triangle = Triangle3D::new(p0, p1, p2).unwrap();
    triangle_to_solid_vertices(&triangle)
}

/// デバッグ用円弧を生成（XY平面、半径1.5、90度）
pub fn create_debug_arc() -> Vec<VertexData> {
    let center = Point3D::<f64>::origin();
    let radius = 1.5;
    let normal =
        Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).expect("Valid normal direction");
    let start_dir =
        Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).expect("Valid start direction");
    let start_angle = geo_foundation::Angle::from_radians(0.0);
    let end_angle = geo_foundation::Angle::from_radians(std::f64::consts::FRAC_PI_2);

    let arc = Arc3D::new(center, radius, normal, start_dir, start_angle, end_angle).unwrap();

    let quality = TessellationQuality::default();
    arc_to_wireframe_line_segments(&arc, &quality)
}

/// デバッグ用クリップ空間座標の正方形（単位行列テスト用）
pub fn create_debug_clip_square() -> Vec<VertexData> {
    let default_normal = [0.0, 0.0, 1.0];
    vec![
        VertexData::new([-0.5, 0.5, 0.0], default_normal),
        VertexData::new([0.5, 0.5, 0.0], default_normal),
        VertexData::new([0.5, 0.5, 0.0], default_normal),
        VertexData::new([0.5, -0.5, 0.0], default_normal),
        VertexData::new([0.5, -0.5, 0.0], default_normal),
        VertexData::new([-0.5, -0.5, 0.0], default_normal),
        VertexData::new([-0.5, -0.5, 0.0], default_normal),
        VertexData::new([-0.5, 0.5, 0.0], default_normal),
    ]
}
