//! cam_sim_visualization_converter - CAMシミュレーション結果の可視化変換
//!
//! CAMシミュレーション（toolpath + tool + work octree）の実行と、
//! 可視化に必要な ViewModel データ（wireframe + snapshot）の結合を担当します。

use cam_core::{
    validate_toolpath_machine_constraints, CamTolerance, MachineConstraint, PathGeometry, Tool,
    ToolPath, ValidationError,
};
use cam_sim::{CuttingSimulator, SimulationError, SnapshotInterval};
use geo_algorithms::{
    octree::{VoxelOctree, VoxelState},
    Aabb3D, LineSegment3D, Point3D,
};
use std::collections::HashMap;
use std::f64::consts::TAU;

use crate::mesh_converter::VertexData;
use crate::octree_converter::{
    voxel_octree_to_wireframe, OctreeVisualizationSettings, VoxelVisualizationOptions,
    WireframeVertex,
};
use crate::snapshot_converter::{
    cam_snapshot_exports_to_inputs, cam_snapshot_inputs_to_domain_series,
    CamSimulationSnapshotInput, DomainSnapshotSeries,
};
use crate::toolpath_converter::{
    create_sample_toolpath, toolpath_to_vertices, ToolPathVisualizationSettings,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CamSimulationVisualizationError {
    Validation(ValidationError),
    Simulation(SimulationError),
}

impl std::fmt::Display for CamSimulationVisualizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(error) => write!(f, "{}", error),
            Self::Simulation(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for CamSimulationVisualizationError {}

impl From<ValidationError> for CamSimulationVisualizationError {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

impl From<SimulationError> for CamSimulationVisualizationError {
    fn from(value: SimulationError) -> Self {
        Self::Simulation(value)
    }
}

/// CAMシミュレーション可視化用の結合データ。
///
/// - `depth_levels`: 最終ワーク（VoxelOctree）の深さ別ワイヤーフレーム
/// - `snapshot_series`: 進捗表示・スクラブに使う時系列スナップショット
#[derive(Debug, Clone)]
pub struct CamSimulationVisualizationBundle {
    pub snapshot_wireframes: Vec<Vec<WireframeVertex>>,
    pub snapshot_solid_meshes: Vec<(Vec<VertexData>, Vec<u32>)>,
    pub snapshot_tool_wireframes: Vec<Vec<WireframeVertex>>,
    pub toolpath_wireframe: Vec<WireframeVertex>,
    pub snapshot_series: DomainSnapshotSeries<CamSimulationSnapshotInput>,
}

fn push_face(
    vertices: &mut Vec<VertexData>,
    indices: &mut Vec<u32>,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
    normal: [f32; 3],
) {
    let aa = a;
    let mut bb = b;
    let cc = c;
    let mut dd = d;

    let ab = [bb[0] - aa[0], bb[1] - aa[1], bb[2] - aa[2]];
    let ac = [cc[0] - aa[0], cc[1] - aa[1], cc[2] - aa[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let dot = cross[0] * normal[0] + cross[1] * normal[1] + cross[2] * normal[2];
    if dot < 0.0 {
        std::mem::swap(&mut bb, &mut dd);
    }

    let base = vertices.len() as u32;
    vertices.push(VertexData::new(aa, normal));
    vertices.push(VertexData::new(bb, normal));
    vertices.push(VertexData::new(cc, normal));
    vertices.push(VertexData::new(dd, normal));

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[derive(Clone, Copy)]
struct FaceQuad {
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
    normal: [f32; 3],
}

fn coord_bits(value: f32) -> u32 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn point_key(position: [f32; 3]) -> [u32; 3] {
    [
        coord_bits(position[0]),
        coord_bits(position[1]),
        coord_bits(position[2]),
    ]
}

fn face_key(face: &FaceQuad) -> [[u32; 3]; 4] {
    let mut points = [
        point_key(face.a),
        point_key(face.b),
        point_key(face.c),
        point_key(face.d),
    ];
    points.sort();
    points
}

fn build_solid_mesh_from_voxel_tree(
    voxel_tree: &VoxelOctree<f64>,
    max_depth: usize,
) -> (Vec<VertexData>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut face_map: HashMap<[[u32; 3]; 4], (FaceQuad, usize)> = HashMap::new();

    let mut register_face = |face: FaceQuad| {
        let key = face_key(&face);
        if let Some((_existing_face, count)) = face_map.get_mut(&key) {
            *count += 1;
        } else {
            face_map.insert(key, (face, 1));
        }
    };

    let solid_bounds = voxel_tree.collect_non_empty_voxel_bounds_up_to_depth(max_depth);
    for bounds in &solid_bounds {
        let min = bounds.min();
        let max = bounds.max();

        let v000 = [min.x() as f32, min.y() as f32, min.z() as f32];
        let v001 = [min.x() as f32, min.y() as f32, max.z() as f32];
        let v010 = [min.x() as f32, max.y() as f32, min.z() as f32];
        let v011 = [min.x() as f32, max.y() as f32, max.z() as f32];
        let v100 = [max.x() as f32, min.y() as f32, min.z() as f32];
        let v101 = [max.x() as f32, min.y() as f32, max.z() as f32];
        let v110 = [max.x() as f32, max.y() as f32, min.z() as f32];
        let v111 = [max.x() as f32, max.y() as f32, max.z() as f32];

        register_face(FaceQuad {
            a: v000,
            b: v100,
            c: v110,
            d: v010,
            normal: [0.0, 0.0, -1.0],
        });
        register_face(FaceQuad {
            a: v001,
            b: v011,
            c: v111,
            d: v101,
            normal: [0.0, 0.0, 1.0],
        });
        register_face(FaceQuad {
            a: v000,
            b: v001,
            c: v101,
            d: v100,
            normal: [0.0, -1.0, 0.0],
        });
        register_face(FaceQuad {
            a: v010,
            b: v110,
            c: v111,
            d: v011,
            normal: [0.0, 1.0, 0.0],
        });
        register_face(FaceQuad {
            a: v000,
            b: v010,
            c: v011,
            d: v001,
            normal: [-1.0, 0.0, 0.0],
        });
        register_face(FaceQuad {
            a: v100,
            b: v101,
            c: v111,
            d: v110,
            normal: [1.0, 0.0, 0.0],
        });
    }

    for (_key, (face, count)) in face_map {
        if count == 1 {
            push_face(
                &mut vertices,
                &mut indices,
                face.a,
                face.b,
                face.c,
                face.d,
                face.normal,
            );
        }
    }

    (vertices, indices)
}

fn collect_line_segments_with_flags(
    toolpath: &ToolPath<f64>,
) -> Result<Vec<(LineSegment3D<f64>, bool)>, SimulationError> {
    let mut out = Vec::new();

    for segment in &toolpath.approach_segments {
        match segment.geometry {
            PathGeometry::Line { end } => {
                if let Some(line_segment) = LineSegment3D::new(segment.start, end) {
                    out.push((line_segment, false));
                }
            }
            PathGeometry::Arc { .. } => return Err(SimulationError::UnsupportedGeometry),
        }
    }

    for contour in &toolpath.contour_levels {
        for segment in &contour.segments {
            match segment.geometry {
                PathGeometry::Line { end } => {
                    if let Some(line_segment) = LineSegment3D::new(segment.start, end) {
                        out.push((line_segment, segment.is_cutting()));
                    }
                }
                PathGeometry::Arc { .. } => return Err(SimulationError::UnsupportedGeometry),
            }
        }
    }

    for segment in &toolpath.retract_segments {
        match segment.geometry {
            PathGeometry::Line { end } => {
                if let Some(line_segment) = LineSegment3D::new(segment.start, end) {
                    out.push((line_segment, false));
                }
            }
            PathGeometry::Arc { .. } => return Err(SimulationError::UnsupportedGeometry),
        }
    }

    Ok(out)
}

fn compute_work_bounds_from_toolpath(
    segments: &[(LineSegment3D<f64>, bool)],
    tool_radius: f64,
) -> Aabb3D<f64> {
    if segments.is_empty() {
        return Aabb3D::new(
            Point3D::new(-60.0, -60.0, -20.0),
            Point3D::new(60.0, 60.0, 5.0),
        );
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut max_z = f64::NEG_INFINITY;

    let mut cutting_min_z = f64::INFINITY;
    let mut cutting_max_z = f64::NEG_INFINITY;

    for (segment, is_cutting) in segments {
        let points = [segment.start(), segment.end()];
        for point in points {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            min_z = min_z.min(point.z());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
            max_z = max_z.max(point.z());

            if *is_cutting {
                cutting_min_z = cutting_min_z.min(point.z());
                cutting_max_z = cutting_max_z.max(point.z());
            }
        }
    }

    if !cutting_min_z.is_finite() || !cutting_max_z.is_finite() {
        cutting_min_z = min_z;
        cutting_max_z = max_z;
    }

    let xy_margin = (tool_radius * 3.0).max(10.0);
    let z_top_margin = (tool_radius * 0.2).max(0.5);
    let z_bottom_margin = (tool_radius * 4.0).max(10.0);

    Aabb3D::new(
        Point3D::new(
            min_x - xy_margin,
            min_y - xy_margin,
            cutting_min_z - z_bottom_margin,
        ),
        Point3D::new(
            max_x + xy_margin,
            max_y + xy_margin,
            cutting_max_z + z_top_margin,
        ),
    )
}

fn count_non_cutting_interference_segments(
    segments: &[(LineSegment3D<f64>, bool)],
    work_bounds: &Aabb3D<f64>,
    tool_radius: f64,
) -> usize {
    let stock_min = work_bounds.min();
    let stock_max = work_bounds.max();

    segments
        .iter()
        .filter(|(_, is_cutting)| !*is_cutting)
        .filter(|(segment, _)| {
            let seg_min_x = segment.start().x().min(segment.end().x()) - tool_radius;
            let seg_min_y = segment.start().y().min(segment.end().y()) - tool_radius;
            let seg_min_z = segment.start().z().min(segment.end().z());

            let seg_max_x = segment.start().x().max(segment.end().x()) + tool_radius;
            let seg_max_y = segment.start().y().max(segment.end().y()) + tool_radius;
            let seg_max_z = segment.start().z().max(segment.end().z()) + 0.0001;

            !(seg_max_x < stock_min.x()
                || seg_min_x > stock_max.x()
                || seg_max_y < stock_min.y()
                || seg_min_y > stock_max.y()
                || seg_max_z < stock_min.z()
                || seg_min_z > stock_max.z())
        })
        .count()
}

fn lerp_point_on_segment(segment: &LineSegment3D<f64>, t: f64) -> Point3D<f64> {
    let tt = t.clamp(0.0, 1.0);
    let sx = segment.start().x();
    let sy = segment.start().y();
    let sz = segment.start().z();
    let ex = segment.end().x();
    let ey = segment.end().y();
    let ez = segment.end().z();

    Point3D::new(
        sx + (ex - sx) * tt,
        sy + (ey - sy) * tt,
        sz + (ez - sz) * tt,
    )
}

fn tool_tip_position(
    segments: &[(LineSegment3D<f64>, bool)],
    segment_index: usize,
    segment_t: f64,
) -> Option<Point3D<f64>> {
    let (segment, _) = segments.get(segment_index)?;
    Some(lerp_point_on_segment(segment, segment_t))
}

fn create_toolpath_wireframe_vertices(toolpath: &ToolPath<f64>) -> Vec<WireframeVertex> {
    let settings = ToolPathVisualizationSettings::default();
    let toolpath_vertices = toolpath_to_vertices(toolpath, &settings);

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

fn create_flat_end_mill_wireframe(
    tip: Point3D<f64>,
    radius: f64,
    cutting_length: f64,
) -> Vec<WireframeVertex> {
    let mut vertices = Vec::new();
    let circle_divisions = 24usize;
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

fn apply_cutting_progress(
    voxel_tree: &mut VoxelOctree<f64>,
    segments: &[(LineSegment3D<f64>, bool)],
    from_segment_index: usize,
    from_t: f64,
    to_segment_index: usize,
    to_t: f64,
    tool_radius: f64,
) {
    if segments.is_empty() {
        return;
    }

    let last_index = segments.len().saturating_sub(1);
    let start_index = from_segment_index.min(last_index);
    let end_index = to_segment_index.min(last_index);
    if end_index < start_index {
        return;
    }

    let blend_half = (tool_radius * 0.25).max(0.5);

    for (index, (segment, is_cutting)) in segments
        .iter()
        .enumerate()
        .take(end_index + 1)
        .skip(start_index)
    {
        if !*is_cutting {
            continue;
        }

        let start_t = if index == start_index { from_t } else { 0.0 }.clamp(0.0, 1.0);
        let end_t = if index == end_index { to_t } else { 1.0 }.clamp(0.0, 1.0);
        if end_t <= start_t {
            continue;
        }

        let start_point = lerp_point_on_segment(segment, start_t);
        let end_point = lerp_point_on_segment(segment, end_t);
        if let Some(cut_segment) = LineSegment3D::new(start_point, end_point) {
            voxel_tree.remove_material_swept_cylinder(&cut_segment, tool_radius);

            let sx = start_point.x();
            let sy = start_point.y();
            let sz = start_point.z();
            let ex = end_point.x();
            let ey = end_point.y();
            let ez = end_point.z();

            let corner_axes = [
                (
                    Point3D::new(sx - blend_half, sy, sz),
                    Point3D::new(sx + blend_half, sy, sz),
                ),
                (
                    Point3D::new(sx, sy - blend_half, sz),
                    Point3D::new(sx, sy + blend_half, sz),
                ),
                (
                    Point3D::new(ex - blend_half, ey, ez),
                    Point3D::new(ex + blend_half, ey, ez),
                ),
                (
                    Point3D::new(ex, ey - blend_half, ez),
                    Point3D::new(ex, ey + blend_half, ez),
                ),
            ];

            for (a, b) in corner_axes {
                if let Some(blend_segment) = LineSegment3D::new(a, b) {
                    voxel_tree.remove_material_swept_cylinder(&blend_segment, tool_radius);
                }
            }
        }
    }
}

/// デバッグ用：テストToolPath + Tool + ワークOctreeで切削シミュレーションを実行し、
/// 可視化に必要な深さ別ワイヤーフレームとスナップショット系列を返す。
pub fn create_sample_cam_simulation_visualization_bundle_with_settings(
    settings: &OctreeVisualizationSettings,
) -> Result<CamSimulationVisualizationBundle, CamSimulationVisualizationError> {
    let toolpath = create_sample_toolpath();
    let tool = Tool::flat_end_mill("endmill_3mm".to_string(), 10.0, 50.0);
    let tolerance = CamTolerance::default();
    let machine_constraint = MachineConstraint::empty();
    validate_toolpath_machine_constraints(&toolpath, &machine_constraint, &tolerance)?;

    let segments = collect_line_segments_with_flags(&toolpath)?;

    let work_bounds = compute_work_bounds_from_toolpath(&segments, tool.radius());
    let voxel_tree = VoxelOctree::new(work_bounds, settings.max_depth);

    let non_cutting_interference_count =
        count_non_cutting_interference_segments(&segments, &work_bounds, tool.radius());
    if non_cutting_interference_count > 0 {
        tracing::warn!(
            "非切削セグメントとワークの干渉候補を検出: {} 件（現在は干渉回避計算未実装）",
            non_cutting_interference_count
        );
    }

    let mut simulator = CuttingSimulator::new(voxel_tree, SnapshotInterval::default());
    simulator.simulate(&toolpath, &tool)?;

    let exports = simulator.snapshot_exports_f64();
    let snapshot_inputs = cam_snapshot_exports_to_inputs(&exports);
    let snapshot_series = cam_snapshot_inputs_to_domain_series("cam_sim", &snapshot_inputs);

    let toolpath_wireframe = create_toolpath_wireframe_vertices(&toolpath);

    let mut replay_tree = VoxelOctree::new(work_bounds, settings.max_depth);
    let mut replay_segment_index = 0usize;
    let mut replay_t = 0.0f64;

    let max_depth = settings.max_depth;
    let mut snapshot_wireframes = Vec::new();
    let mut snapshot_solid_meshes = Vec::new();
    let mut snapshot_tool_wireframes = Vec::new();

    for snapshot in &snapshot_series.frames {
        let payload = snapshot.payload;

        apply_cutting_progress(
            &mut replay_tree,
            &segments,
            replay_segment_index,
            replay_t,
            payload.segment_index,
            payload.segment_t,
            tool.radius(),
        );

        replay_segment_index = payload.segment_index;
        replay_t = payload.segment_t;

        let options = VoxelVisualizationOptions {
            depth_range: 0..(max_depth + 1),
            show_states: vec![VoxelState::Solid],
            color_by_state: false,
            color_by_depth: true,
            max_depth: max_depth.max(1),
        };

        let mut frame_vertices = voxel_octree_to_wireframe(&replay_tree, &options);
        frame_vertices.extend(toolpath_wireframe.iter().copied());

        let mut tool_wireframe = Vec::new();

        if let Some(tool_tip) =
            tool_tip_position(&segments, payload.segment_index, payload.segment_t)
        {
            let tool_wire =
                create_flat_end_mill_wireframe(tool_tip, tool.radius(), tool.cutting_length);
            tool_wireframe = tool_wire;
            frame_vertices.extend(tool_wireframe.iter().copied());
            snapshot_solid_meshes.push(build_solid_mesh_from_voxel_tree(&replay_tree, max_depth));
        } else {
            snapshot_solid_meshes.push(build_solid_mesh_from_voxel_tree(&replay_tree, max_depth));
        }

        snapshot_tool_wireframes.push(tool_wireframe);
        snapshot_wireframes.push(frame_vertices);
    }

    Ok(CamSimulationVisualizationBundle {
        snapshot_wireframes,
        snapshot_solid_meshes,
        snapshot_tool_wireframes,
        toolpath_wireframe,
        snapshot_series,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_algorithms::octree::OctreeTolerance;

    #[test]
    fn test_create_sample_cam_simulation_visualization_bundle_with_settings() {
        let settings = OctreeVisualizationSettings {
            max_depth: 3,
            gradient_start: [0.2, 1.0, 1.0],
            gradient_end: [1.0, 0.4, 0.4],
            octree_tolerance: OctreeTolerance::default(),
        };

        let bundle = create_sample_cam_simulation_visualization_bundle_with_settings(&settings)
            .expect("cam simulation visualization should be created");

        assert!(!bundle.snapshot_series.frames.is_empty());
        assert_eq!(
            bundle.snapshot_wireframes.len(),
            bundle.snapshot_series.frames.len()
        );
        assert_eq!(
            bundle.snapshot_solid_meshes.len(),
            bundle.snapshot_series.frames.len()
        );
        assert!(bundle
            .snapshot_wireframes
            .iter()
            .any(|vertices| !vertices.is_empty()));
        assert!(bundle
            .snapshot_solid_meshes
            .iter()
            .any(|(vertices, indices)| !vertices.is_empty() && !indices.is_empty()));
    }
}
