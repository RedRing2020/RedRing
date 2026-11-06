//! shape_converter - 幾何形状からGPU頂点データへの変換
//!
//! 線分・円・円弧・楕円弧・三角形を、GPU レンダリング用の頂点データに変換します。

use crate::mesh_converter::VertexData;
use geo_algorithms::{Arc3D, Circle3D, EllipseArc3D, LineSegment3D, Point3D, Triangle3D, Vector3D};
use geo_contracts::{
    Arc3DProperties, Circle3DProperties, EllipseArc3DProperties, LineSegment3DProperties,
    Triangle3DBoundaryAccess,
};
use std::f64::consts::PI;

/// テッセレーション品質パラメータ（曲線を線分に分ける数）
#[derive(Debug, Clone)]
pub struct TessellationQuality {
    /// 円弧・楕円弧の最小セグメント数
    pub min_segments: usize,

    /// 円形状のデフォルトセグメント数
    pub circle_segments: usize,
}

impl Default for TessellationQuality {
    fn default() -> Self {
        Self {
            min_segments: 8,
            circle_segments: 32,
        }
    }
}

/// LineSegment3D を GPU用頂点データに変換
///
/// 線分は始点と終点の2頂点で表現されます。
/// 法線はゼロベクトルとして設定されます（線には法線が定義されないため）。
pub fn line_segment_to_vertices(segment: &LineSegment3D<f64>) -> Vec<VertexData> {
    // 始点と終点を取得
    let start = segment.start_point();
    let end = segment.end_point();

    // 法線はゼロベクトル（線には法線が定義されない）
    let normal = [0.0f32, 0.0, 0.0];

    vec![
        VertexData::new(
            [start.x() as f32, start.y() as f32, start.z() as f32],
            normal,
        ),
        VertexData::new([end.x() as f32, end.y() as f32, end.z() as f32], normal),
    ]
}

/// Circle3D をワイヤーフレーム用LineList頂点データに変換
///
/// 円周を線分のリストとして表現します（LineListトポロジ用）。
/// 各セグメントは2頂点のペアとして生成されます。
pub fn circle_to_wireframe_line_segments(
    circle: &Circle3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    let segments = quality.circle_segments;
    let mut vertices = Vec::with_capacity(segments * 2); // 各線分に2頂点

    // 円の中心と半径を取得
    let center = circle.center();
    let radius = circle.radius();

    // 円の法線方向を取得
    let normal_vec = circle.axis();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // 円の平面上で2つの直交する基底ベクトルを計算
    let arbitrary = if normal_vec.x().abs() > 0.9 {
        Vector3D::new(0.0, 1.0, 0.0)
    } else {
        Vector3D::new(1.0, 0.0, 0.0)
    };

    let u = normal_vec.cross(arbitrary).normalize();
    let v = normal_vec.cross(u).normalize();

    // 円周を線分のペアとして生成
    for i in 0..segments {
        let angle1 = 2.0 * PI * (i as f64) / (segments as f64);
        let angle2 = 2.0 * PI * ((i + 1) as f64) / (segments as f64);

        // 始点
        let point1_x = center.x() + radius * (angle1.cos() * u.x() + angle1.sin() * v.x());
        let point1_y = center.y() + radius * (angle1.cos() * u.y() + angle1.sin() * v.y());
        let point1_z = center.z() + radius * (angle1.cos() * u.z() + angle1.sin() * v.z());

        // 終点
        let point2_x = center.x() + radius * (angle2.cos() * u.x() + angle2.sin() * v.x());
        let point2_y = center.y() + radius * (angle2.cos() * u.y() + angle2.sin() * v.y());
        let point2_z = center.z() + radius * (angle2.cos() * u.z() + angle2.sin() * v.z());

        // 線分として2頂点を追加
        vertices.push(VertexData::new(
            [point1_x as f32, point1_y as f32, point1_z as f32],
            normal,
        ));
        vertices.push(VertexData::new(
            [point2_x as f32, point2_y as f32, point2_z as f32],
            normal,
        ));
    }

    vertices
}

/// Triangle3D を GPU用頂点データに変換（ソリッド）
///
/// 三角形を塗りつぶして表示します。
/// CCW（反時計回り）順序で頂点を配置し、法線を計算します。
pub fn triangle_to_solid_vertices(triangle: &Triangle3D<f64>) -> Vec<VertexData> {
    // 3つの頂点を取得
    let va = triangle.vertex_a();
    let vb = triangle.vertex_b();
    let vc = triangle.vertex_c();

    // 法線を計算（CCW順序を前提）
    let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
    let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());
    let normal_vec = edge1.cross(edge2).normalize();
    let normal = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // 3頂点を配置
    vec![
        VertexData::new([va.x() as f32, va.y() as f32, va.z() as f32], normal),
        VertexData::new([vb.x() as f32, vb.y() as f32, vb.z() as f32], normal),
        VertexData::new([vc.x() as f32, vc.y() as f32, vc.z() as f32], normal),
    ]
}

/// Arc3D をワイヤーフレーム用LineList頂点データに変換
///
/// 円弧を線分のリストとして表現します（LineListトポロジ用）。
/// 各セグメントは2頂点のペアとして生成されます。
pub fn arc_to_wireframe_line_segments(
    arc: &Arc3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    // 円弧の角度範囲を取得
    let start_angle: f64 = Arc3DProperties::start_angle(arc).to_radians();
    let end_angle: f64 = Arc3DProperties::end_angle(arc).to_radians();

    // 角度範囲を計算（CCW方向）
    let mut angle_range: f64 = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += 2.0 * PI;
    }

    // 角度範囲に基づいてセグメント数を決定
    let segments = ((angle_range / (2.0 * PI)) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;

    let mut vertices = Vec::with_capacity(segments * 2); // 各線分に2頂点

    // 円弧の中心と半径を取得
    let center = Arc3DProperties::center(arc);
    let radius: f64 = Arc3DProperties::radius(arc);

    // 円弧の法線方向と開始方向を取得
    let start_dir = arc.start_direction();

    // Direction3Dから Vector3Dへ変換
    let normal_vec = arc.normal().as_vector();
    let start_dir_vec = start_dir.as_vector();

    let normal_f32 = [
        normal_vec.x() as f32,
        normal_vec.y() as f32,
        normal_vec.z() as f32,
    ];

    // V方向 = normal × start_dir
    let v = normal_vec.cross(start_dir_vec).normalize();

    // 円弧を線分のペアとして生成
    for i in 0..segments {
        let t1 = (i as f64) / (segments as f64);
        let t2 = ((i + 1) as f64) / (segments as f64);

        let angle1 = start_angle + t1 * angle_range;
        let angle2 = start_angle + t2 * angle_range;

        // 始点
        let cos_a1 = angle1.cos();
        let sin_a1 = angle1.sin();
        let point1_x = center.x() + radius * (cos_a1 * start_dir_vec.x() + sin_a1 * v.x());
        let point1_y = center.y() + radius * (cos_a1 * start_dir_vec.y() + sin_a1 * v.y());
        let point1_z = center.z() + radius * (cos_a1 * start_dir_vec.z() + sin_a1 * v.z());

        // 終点
        let cos_a2 = angle2.cos();
        let sin_a2 = angle2.sin();
        let point2_x = center.x() + radius * (cos_a2 * start_dir_vec.x() + sin_a2 * v.x());
        let point2_y = center.y() + radius * (cos_a2 * start_dir_vec.y() + sin_a2 * v.y());
        let point2_z = center.z() + radius * (cos_a2 * start_dir_vec.z() + sin_a2 * v.z());

        // 線分として2頂点を追加
        vertices.push(VertexData::new(
            [point1_x as f32, point1_y as f32, point1_z as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [point2_x as f32, point2_y as f32, point2_z as f32],
            normal_f32,
        ));
    }

    vertices
}

/// EllipseArc3D をワイヤーフレーム用LineList頂点データに変換
///
/// 楕円弧を線分のリストとして表現します（LineListトポロジ用）。
pub fn ellipse_arc_to_wireframe_line_segments(
    arc: &EllipseArc3D<f64>,
    quality: &TessellationQuality,
) -> Vec<VertexData> {
    // 角度範囲を取得
    let start_angle: f64 = EllipseArc3DProperties::start_angle(arc).to_radians();
    let end_angle: f64 = EllipseArc3DProperties::end_angle(arc).to_radians();

    let mut angle_range: f64 = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += 2.0 * PI;
    }

    let segments = ((angle_range / (2.0 * PI)) * (quality.circle_segments as f64))
        .ceil()
        .max(quality.min_segments as f64) as usize;

    let mut vertices = Vec::with_capacity(segments * 2);

    let center = EllipseArc3DProperties::center(arc);
    let semi_major = EllipseArc3DProperties::semi_major_axis(arc);
    let semi_minor = EllipseArc3DProperties::semi_minor_axis(arc);
    let normal = EllipseArc3DProperties::normal(arc).as_vector();
    let major_dir = EllipseArc3DProperties::major_axis_direction(arc).as_vector();
    let minor_dir = EllipseArc3DProperties::minor_axis_direction(arc).as_vector();

    let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

    // 楕円弧を線分として生成
    for i in 0..segments {
        let t1 = (i as f64) / (segments as f64);
        let t2 = ((i + 1) as f64) / (segments as f64);

        let angle1 = start_angle + t1 * angle_range;
        let angle2 = start_angle + t2 * angle_range;

        // 始点
        let point1 = Point3D::new(
            center.x()
                + semi_major * angle1.cos() * major_dir.x()
                + semi_minor * angle1.sin() * minor_dir.x(),
            center.y()
                + semi_major * angle1.cos() * major_dir.y()
                + semi_minor * angle1.sin() * minor_dir.y(),
            center.z()
                + semi_major * angle1.cos() * major_dir.z()
                + semi_minor * angle1.sin() * minor_dir.z(),
        );

        // 終点
        let point2 = Point3D::new(
            center.x()
                + semi_major * angle2.cos() * major_dir.x()
                + semi_minor * angle2.sin() * minor_dir.x(),
            center.y()
                + semi_major * angle2.cos() * major_dir.y()
                + semi_minor * angle2.sin() * minor_dir.y(),
            center.z()
                + semi_major * angle2.cos() * major_dir.z()
                + semi_minor * angle2.sin() * minor_dir.z(),
        );

        vertices.push(VertexData::new(
            [point1.x() as f32, point1.y() as f32, point1.z() as f32],
            normal_f32,
        ));
        vertices.push(VertexData::new(
            [point2.x() as f32, point2.y() as f32, point2.z() as f32],
            normal_f32,
        ));
    }

    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    use geo_contracts::{LineSegment3DConstructor, Triangle3DConstructor};

    #[test]
    fn test_line_segment_conversion() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(1.0, 1.0, 1.0);
        let segment = LineSegment3D::new(start, end).unwrap();

        let vertices = line_segment_to_vertices(&segment);

        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_triangle_solid_conversion() {
        // XY平面上の三角形
        let va = Point3D::new(0.0, 0.0, 0.0);
        let vb = Point3D::new(1.0, 0.0, 0.0);
        let vc = Point3D::new(0.0, 1.0, 0.0);
        let triangle = Triangle3D::new(va, vb, vc).unwrap();

        let vertices = triangle_to_solid_vertices(&triangle);

        // 3頂点
        assert_eq!(vertices.len(), 3);

        // 頂点位置の検証
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(vertices[2].position, [0.0, 1.0, 0.0]);

        // 全ての法線がZ方向（0, 0, 1）であることを確認
        for vertex in &vertices {
            assert!((vertex.normal[0] - 0.0).abs() < 0.001);
            assert!((vertex.normal[1] - 0.0).abs() < 0.001);
            assert!((vertex.normal[2] - 1.0).abs() < 0.001);
        }
    }

    /// 線の頂点の、原点からの距離
    fn distances(vertices: &[VertexData]) -> Vec<f32> {
        vertices
            .iter()
            .map(|vertex| {
                let [x, y, z] = vertex.position;
                (x * x + y * y + z * z).sqrt()
            })
            .collect()
    }

    #[test]
    fn circle_lines_lie_on_the_circle() {
        use geo_algorithms::Direction3D;
        use geo_contracts::Circle3DConstructor;
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let circle = Circle3D::new(Point3D::new(0.0, 0.0, 0.0), normal, 2.0).unwrap();
        let quality = TessellationQuality {
            circle_segments: 16,
            ..Default::default()
        };
        let vertices = circle_to_wireframe_line_segments(&circle, &quality);
        // 16 本の線分の両端
        assert_eq!(vertices.len(), 32);
        assert!(distances(&vertices).iter().all(|d| (d - 2.0).abs() < 1e-3));
    }

    #[test]
    fn arc_lines_lie_on_the_arc_from_the_start_to_the_end() {
        use geo_algorithms::Angle;
        use geo_contracts::Arc3DConstructor;
        let arc = Arc3D::xy_arc(
            Point3D::new(0.0, 0.0, 0.0),
            3.0,
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::FRAC_PI_2),
        )
        .unwrap();
        let vertices = arc_to_wireframe_line_segments(&arc, &TessellationQuality::default());
        assert!(!vertices.is_empty() && vertices.len().is_multiple_of(2));
        assert!(distances(&vertices).iter().all(|d| (d - 3.0).abs() < 1e-3));
        // 始点 (3, 0, 0) から終点 (0, 3, 0) まで
        let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(p, q)| (p - q).abs() < 1e-4);
        assert!(close(vertices[0].position, [3.0, 0.0, 0.0]));
        assert!(close(
            vertices[vertices.len() - 1].position,
            [0.0, 3.0, 0.0]
        ));
    }
}
