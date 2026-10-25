//! cam_sim_visualization_converter - 切削シミュレーションの表示用の線の変換
//!
//! 工具の先端の位置、工具の形（フラット・ボールエンドミル）とツールパスの線を、描画用の線の頂点にする。

use cam_core::ToolPath;
use geo_algorithms::{LineSegment3D, Point3D};
use geo_contracts::LineSegment3DProperties;
use std::f64::consts::TAU;

use crate::octree_converter::WireframeVertex;
use crate::toolpath_converter::{toolpath_to_vertices, ToolPathVisualizationSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolWireframeVisualizationSettings {
    pub flat_circle_divisions: usize,
    pub ball_circle_divisions: usize,
    pub ball_hemisphere_divisions: usize,
    pub ball_meridian_count: usize,
}

impl Default for ToolWireframeVisualizationSettings {
    fn default() -> Self {
        Self {
            flat_circle_divisions: 24,
            ball_circle_divisions: 48,
            ball_hemisphere_divisions: 8,
            ball_meridian_count: 12,
        }
    }
}

fn lerp_point_on_segment(segment: &LineSegment3D<f64>, t: f64) -> Point3D<f64> {
    let tt = t.clamp(0.0, 1.0);
    let sx = segment.start_point().x();
    let sy = segment.start_point().y();
    let sz = segment.start_point().z();
    let ex = segment.end_point().x();
    let ey = segment.end_point().y();
    let ez = segment.end_point().z();

    Point3D::new(
        sx + (ex - sx) * tt,
        sy + (ey - sy) * tt,
        sz + (ez - sz) * tt,
    )
}

pub(crate) fn tool_tip_position(
    segments: &[(LineSegment3D<f64>, bool)],
    segment_index: usize,
    segment_t: f64,
) -> Option<Point3D<f64>> {
    let (segment, _) = segments.get(segment_index)?;
    Some(lerp_point_on_segment(segment, segment_t))
}

pub(crate) fn create_toolpath_wireframe_vertices(
    toolpath: &ToolPath<f64>,
    settings: &ToolPathVisualizationSettings,
) -> Vec<WireframeVertex> {
    let toolpath_vertices = toolpath_to_vertices(toolpath, settings);

    toolpath_vertices
        .vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let color = toolpath_vertices
                .colors
                .get(index / 2)
                .copied()
                .unwrap_or([1.0, 1.0, 1.0, 1.0]);
            WireframeVertex::new(vertex.position, [color[0], color[1], color[2]])
        })
        .collect()
}

pub(crate) fn create_flat_end_mill_wireframe(
    tip: Point3D<f64>,
    radius: f64,
    cutting_length: f64,
    settings: &ToolWireframeVisualizationSettings,
) -> Vec<WireframeVertex> {
    let mut vertices = Vec::new();
    let circle_divisions = settings.flat_circle_divisions.max(6);
    let tool_color = [0.9, 0.95, 1.0];

    let z0 = tip.z();
    let z1 = z0 + cutting_length.max(1.0);
    let cx = tip.x();
    let cy = tip.y();

    for i in 0..circle_divisions {
        let theta0 = TAU * (i as f64) / (circle_divisions as f64);
        let theta1 = TAU * ((i + 1) as f64) / (circle_divisions as f64);

        let x0 = cx + radius * theta0.cos();
        let y0 = cy + radius * theta0.sin();
        let x1 = cx + radius * theta1.cos();
        let y1 = cy + radius * theta1.sin();

        let bottom0 = [x0 as f32, y0 as f32, z0 as f32];
        let bottom1 = [x1 as f32, y1 as f32, z0 as f32];
        let top0 = [x0 as f32, y0 as f32, z1 as f32];
        let top1 = [x1 as f32, y1 as f32, z1 as f32];

        vertices.push(WireframeVertex::new(bottom0, tool_color));
        vertices.push(WireframeVertex::new(bottom1, tool_color));

        vertices.push(WireframeVertex::new(top0, tool_color));
        vertices.push(WireframeVertex::new(top1, tool_color));

        if i % 3 == 0 {
            vertices.push(WireframeVertex::new(bottom0, tool_color));
            vertices.push(WireframeVertex::new(top0, tool_color));
        }
    }

    vertices
}

pub(crate) fn create_ball_end_mill_wireframe(
    tip: Point3D<f64>,
    radius: f64,
    cutting_length: f64,
    settings: &ToolWireframeVisualizationSettings,
) -> Vec<WireframeVertex> {
    let mut vertices = Vec::new();
    let circle_divisions = settings.ball_circle_divisions.max(8);
    let hemisphere_divisions = settings.ball_hemisphere_divisions.max(2);
    let meridian_count = settings.ball_meridian_count.max(4);
    let tool_color = [0.9, 0.95, 1.0];

    let center_z = tip.z() + radius;
    let z1 = center_z + cutting_length.max(1.0);
    let cx = tip.x();
    let cy = tip.y();

    // 円筒部（外周リング + 上端リング + 軸方向補助線）
    for i in 0..circle_divisions {
        let theta0 = TAU * (i as f64) / (circle_divisions as f64);
        let theta1 = TAU * ((i + 1) as f64) / (circle_divisions as f64);

        let x0 = cx + radius * theta0.cos();
        let y0 = cy + radius * theta0.sin();
        let x1 = cx + radius * theta1.cos();
        let y1 = cy + radius * theta1.sin();

        let equator0 = [x0 as f32, y0 as f32, center_z as f32];
        let equator1 = [x1 as f32, y1 as f32, center_z as f32];
        let top0 = [x0 as f32, y0 as f32, z1 as f32];
        let top1 = [x1 as f32, y1 as f32, z1 as f32];

        vertices.push(WireframeVertex::new(equator0, tool_color));
        vertices.push(WireframeVertex::new(equator1, tool_color));

        vertices.push(WireframeVertex::new(top0, tool_color));
        vertices.push(WireframeVertex::new(top1, tool_color));

        if i % 3 == 0 {
            vertices.push(WireframeVertex::new(equator0, tool_color));
            vertices.push(WireframeVertex::new(top0, tool_color));
        }
    }

    // 半球部（緯線）
    for lat in 1..hemisphere_divisions {
        let phi = (lat as f64) * (std::f64::consts::FRAC_PI_2 / hemisphere_divisions as f64);
        let ring_radius = radius * phi.cos();
        let ring_z = center_z - radius * phi.sin();

        for i in 0..circle_divisions {
            let theta0 = TAU * (i as f64) / (circle_divisions as f64);
            let theta1 = TAU * ((i + 1) as f64) / (circle_divisions as f64);

            let p0 = [
                (cx + ring_radius * theta0.cos()) as f32,
                (cy + ring_radius * theta0.sin()) as f32,
                ring_z as f32,
            ];
            let p1 = [
                (cx + ring_radius * theta1.cos()) as f32,
                (cy + ring_radius * theta1.sin()) as f32,
                ring_z as f32,
            ];

            vertices.push(WireframeVertex::new(p0, tool_color));
            vertices.push(WireframeVertex::new(p1, tool_color));
        }
    }

    // 半球部（経線）
    for meridian in 0..meridian_count {
        let theta = TAU * (meridian as f64) / (meridian_count as f64);
        let mut prev = [cx as f32, cy as f32, tip.z() as f32];

        for lat in (0..=hemisphere_divisions).rev() {
            let phi = (lat as f64) * (std::f64::consts::FRAC_PI_2 / hemisphere_divisions as f64);
            let ring_radius = radius * phi.cos();
            let ring_z = center_z - radius * phi.sin();

            let curr = [
                (cx + ring_radius * theta.cos()) as f32,
                (cy + ring_radius * theta.sin()) as f32,
                ring_z as f32,
            ];

            vertices.push(WireframeVertex::new(prev, tool_color));
            vertices.push(WireframeVertex::new(curr, tool_color));
            prev = curr;
        }
    }

    vertices
}
