//! 逆オフセット包絡面（CL 面）のデバッグ可視化データ変換
//!
//! `application::cam_inspection` の検査結果を、MeshStage で描画できる
//! シェーディングメッシュと重畳線分へ変換する。
//!
//! - 包絡面モード: 包絡面をシェーディング表示し、元形状を三角形エッジで重畳する
//! - 格子線モード: 元形状をシェーディング表示し、包絡面を一定ピッチの格子線で重畳する
//!
//! 両モードとも包絡面の接触境界（工具が形状に触れ得る範囲の外縁）を輪郭線で重畳する。
//! 境界は格子辺上の二分法境界点（model 層で算出）を marching squares で結んで求め、
//! 境界を跨ぐ格子セル・格子線は境界点まで切り詰めて表示する。

use std::collections::HashSet;

/// View 層が application へ直接依存しないよう、工具種別を再エクスポートする
pub use application::cam_inspection::InspectionToolKind;
use application::cam_inspection::{
    inspect_inverse_offset, InverseOffsetEnvelopeGrid, InverseOffsetInspection,
    InverseOffsetInspectionError, InverseOffsetInspectionRequest,
};
use geo_algorithms::nurbs_fixtures::create_sample_nurbs_surface_with_adaptive_params;

use crate::mesh_converter::VertexData;
use crate::octree_converter::WireframeVertex;

/// 包絡面の表示モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InverseOffsetDisplayMode {
    /// 包絡面をシェーディング表示（元形状は三角形エッジで重畳）
    EnvelopeSurface,
    /// 包絡面を一定ピッチの格子線で表示（元形状はシェーディング表示）
    EnvelopeGrid,
}

impl InverseOffsetDisplayMode {
    pub fn toggled(self) -> Self {
        match self {
            Self::EnvelopeSurface => Self::EnvelopeGrid,
            Self::EnvelopeGrid => Self::EnvelopeSurface,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::EnvelopeSurface => "envelope surface",
            Self::EnvelopeGrid => "envelope grid lines",
        }
    }
}

/// 逆オフセット包絡面デバッグ表示の設定（View 側から保持・注入する想定）
#[derive(Debug, Clone)]
pub struct InverseOffsetVisualizationSettings {
    /// 工具半径（サンプル曲面の座標系）
    pub tool_radius: f64,
    /// 包絡面の評価間隔
    pub sample_pitch: f64,
    /// 格子線を引く間隔（評価点何個ごとに 1 本か）。格子線ピッチ = sample_pitch × この値
    pub grid_line_interval: usize,
    /// 包絡面モードで包絡面をシェーディングする色（RGBA）
    pub envelope_surface_color: [f32; 4],
    /// 格子線モードで元形状をシェーディングする色（RGBA）
    pub source_surface_color: [f32; 4],
    /// 元形状の重畳線色
    pub source_line_color: [f32; 3],
    /// 包絡面格子線の色
    pub envelope_line_color: [f32; 3],
    /// 包絡面の接触境界（輪郭線）の色
    pub envelope_outline_color: [f32; 3],
}

impl Default for InverseOffsetVisualizationSettings {
    fn default() -> Self {
        Self {
            tool_radius: 0.1,
            sample_pitch: 0.01,
            grid_line_interval: 5,
            envelope_surface_color: [0.95, 0.55, 0.25, 1.0],
            source_surface_color: [0.55, 0.6, 0.7, 1.0],
            source_line_color: [0.6, 0.6, 0.6],
            envelope_line_color: [1.0, 0.6, 0.1],
            envelope_outline_color: [0.3, 0.9, 1.0],
        }
    }
}

impl InverseOffsetVisualizationSettings {
    /// 格子線ピッチ
    pub fn grid_line_pitch(&self) -> f64 {
        self.sample_pitch * self.grid_line_interval as f64
    }
}

/// 逆オフセット包絡面の描画データ
#[derive(Debug, Clone)]
pub struct InverseOffsetVisualization {
    pub mode: InverseOffsetDisplayMode,
    pub tool_kind: InspectionToolKind,
    /// シェーディング表示するメッシュ頂点（三角形ごとに独立した頂点・面法線）
    pub mesh_vertices: Vec<VertexData>,
    pub mesh_indices: Vec<u32>,
    /// シェーディング色（RGBA）
    pub mesh_color: [f32; 4],
    /// 重畳する線分（LineList: 2 頂点で 1 本）
    pub overlay_lines: Vec<WireframeVertex>,
    /// カメラ合わせ用の全表示点
    pub fit_positions: Vec<[f32; 3]>,
    /// 工具先端から包絡面基準点までの高さ
    pub reference_offset: f64,
    /// 包絡面の評価点数（列 × 行）
    pub envelope_samples: (usize, usize),
}

/// 可視化データ生成の失敗
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InverseOffsetVisualizationError {
    #[error("sample surface construction failed: {0}")]
    SampleSurface(String),
    #[error("inverse offset inspection failed: {0}")]
    Inspection(#[from] InverseOffsetInspectionError),
    #[error("grid_line_interval must be at least 1")]
    InvalidGridLineInterval,
}

/// サンプル NURBS 曲面（`m` キー表示と同一）の逆オフセット包絡面を可視化データへ変換する。
pub fn build_sample_inverse_offset_visualization(
    tool_kind: InspectionToolKind,
    mode: InverseOffsetDisplayMode,
    chord_tolerance: f64,
    settings: &InverseOffsetVisualizationSettings,
) -> Result<InverseOffsetVisualization, InverseOffsetVisualizationError> {
    let (surface, _) = create_sample_nurbs_surface_with_adaptive_params(chord_tolerance)
        .map_err(InverseOffsetVisualizationError::SampleSurface)?;
    let inspection = inspect_inverse_offset(&InverseOffsetInspectionRequest {
        surfaces: vec![surface],
        tool_kind,
        tool_radius: settings.tool_radius,
        chord_tolerance,
        grid_pitch: settings.sample_pitch,
    })?;
    inverse_offset_to_visualization(&inspection, tool_kind, mode, settings)
}

/// 検査結果を表示モードに応じた描画データへ変換する。
pub fn inverse_offset_to_visualization(
    inspection: &InverseOffsetInspection,
    tool_kind: InspectionToolKind,
    mode: InverseOffsetDisplayMode,
    settings: &InverseOffsetVisualizationSettings,
) -> Result<InverseOffsetVisualization, InverseOffsetVisualizationError> {
    if settings.grid_line_interval == 0 {
        return Err(InverseOffsetVisualizationError::InvalidGridLineInterval);
    }

    let source_triangles: Vec<[[f64; 3]; 3]> = inspection
        .source_triangles
        .iter()
        .map(|tri| tri.map(|vi| inspection.source_vertices[vi]))
        .collect();
    let envelope_triangles = envelope_triangles(&inspection.envelope);

    let (mesh_triangles, mesh_color, mut overlay_lines) = match mode {
        InverseOffsetDisplayMode::EnvelopeSurface => (
            envelope_triangles.clone(),
            settings.envelope_surface_color,
            triangle_edge_lines(
                &inspection.source_vertices,
                &inspection.source_triangles,
                settings.source_line_color,
            ),
        ),
        InverseOffsetDisplayMode::EnvelopeGrid => (
            source_triangles.clone(),
            settings.source_surface_color,
            envelope_grid_lines(
                &inspection.envelope,
                settings.grid_line_interval,
                settings.envelope_line_color,
            ),
        ),
    };

    overlay_lines.extend(envelope_outline_lines(
        &inspection.envelope,
        settings.envelope_outline_color,
    ));

    let (mesh_vertices, mesh_indices) = flat_shaded_mesh(&mesh_triangles);
    let fit_positions = source_triangles
        .iter()
        .chain(envelope_triangles.iter())
        .flatten()
        .map(|p| to_f32(*p))
        .collect();

    Ok(InverseOffsetVisualization {
        mode,
        tool_kind,
        mesh_vertices,
        mesh_indices,
        mesh_color,
        overlay_lines,
        fit_positions,
        reference_offset: inspection.reference_offset,
        envelope_samples: (inspection.envelope.columns, inspection.envelope.rows),
    })
}

/// 格子セルの 4 隅と 4 辺の境界点を、セル外周を一周する順に並べた要素
///
/// 順序: 隅(c,r) → 下辺 → 隅(c+1,r) → 右辺 → 隅(c+1,r+1) → 上辺 → 隅(c,r+1) → 左辺
struct CellRing {
    corners: [Option<[f64; 3]>; 4],
    edges: [Option<[f64; 3]>; 4],
}

impl CellRing {
    fn new(grid: &InverseOffsetEnvelopeGrid, column: usize, row: usize) -> Self {
        let corner = |c: usize, r: usize| {
            grid.height(c, r).map(|z| {
                let [x, y] = grid.xy(c, r);
                [x, y, z]
            })
        };
        Self {
            corners: [
                corner(column, row),
                corner(column + 1, row),
                corner(column + 1, row + 1),
                corner(column, row + 1),
            ],
            edges: [
                grid.x_edge_boundary(column, row),
                grid.y_edge_boundary(column + 1, row),
                grid.x_edge_boundary(column, row + 1),
                grid.y_edge_boundary(column, row),
            ],
        }
    }

    fn crossing_count(&self) -> usize {
        self.edges.iter().flatten().count()
    }

    /// セル内の接触領域を三角形化する（境界を跨ぐセルは境界点で切り詰める）
    fn triangles(&self) -> Vec<[[f64; 3]; 3]> {
        match self.crossing_count() {
            0 => match self.corners {
                [Some(p0), Some(p1), Some(p2), Some(p3)] => vec![[p0, p1, p2], [p0, p2, p3]],
                _ => Vec::new(),
            },
            // 鞍点セル: 接触している隅ごとに、隣接 2 辺の境界点との三角形を作る
            4 => (0..4)
                .filter_map(|i| {
                    let corner = self.corners[i]?;
                    let before = self.edges[(i + 3) % 4]?;
                    let after = self.edges[i]?;
                    Some([corner, after, before])
                })
                .collect(),
            // 境界が 1 本の直線で横切るセル: 接触側は凸多角形なので扇形分割する
            _ => {
                let polygon: Vec<[f64; 3]> = (0..4)
                    .flat_map(|i| [self.corners[i], self.edges[i]])
                    .flatten()
                    .collect();
                (1..polygon.len().saturating_sub(1))
                    .map(|i| [polygon[0], polygon[i], polygon[i + 1]])
                    .collect()
            }
        }
    }

    /// セル内の接触境界線分
    fn outline_segments(&self) -> Vec<([f64; 3], [f64; 3])> {
        match self.crossing_count() {
            2 => {
                let crossings: Vec<[f64; 3]> = self.edges.iter().flatten().copied().collect();
                vec![(crossings[0], crossings[1])]
            }
            4 => (0..4)
                .filter_map(|i| {
                    self.corners[i]?;
                    Some((self.edges[(i + 3) % 4]?, self.edges[i]?))
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}

/// 包絡面の接触領域を三角形化する。
fn envelope_triangles(grid: &InverseOffsetEnvelopeGrid) -> Vec<[[f64; 3]; 3]> {
    let mut triangles = Vec::new();
    for row in 0..grid.rows.saturating_sub(1) {
        for column in 0..grid.columns.saturating_sub(1) {
            triangles.extend(CellRing::new(grid, column, row).triangles());
        }
    }
    triangles
}

/// 格子線上の 1 区間を、接触側の点から相手点（接触あり）または境界点まで結ぶ。
fn clipped_segment(
    a: Option<[f64; 3]>,
    b: Option<[f64; 3]>,
    boundary: Option<[f64; 3]>,
) -> Option<([f64; 3], [f64; 3])> {
    match (a, b) {
        (Some(a), Some(b)) => Some((a, b)),
        (Some(a), None) => boundary.map(|p| (a, p)),
        (None, Some(b)) => boundary.map(|p| (p, b)),
        (None, None) => None,
    }
}

fn grid_point(grid: &InverseOffsetEnvelopeGrid, column: usize, row: usize) -> Option<[f64; 3]> {
    grid.height(column, row).map(|z| {
        let [x, y] = grid.xy(column, row);
        [x, y, z]
    })
}

fn push_line(lines: &mut Vec<WireframeVertex>, (a, b): ([f64; 3], [f64; 3]), color: [f32; 3]) {
    lines.push(WireframeVertex::new(to_f32(a), color));
    lines.push(WireframeVertex::new(to_f32(b), color));
}

/// 包絡面の格子線（X/Y 各方向に `interval` 評価点ごと、外周を含む）。
/// 接触境界を跨ぐ区間は境界点まで延ばす。
fn envelope_grid_lines(
    grid: &InverseOffsetEnvelopeGrid,
    interval: usize,
    color: [f32; 3],
) -> Vec<WireframeVertex> {
    let is_grid_line =
        |index: usize, count: usize| index.is_multiple_of(interval) || index + 1 == count;

    let mut lines = Vec::new();
    for row in (0..grid.rows).filter(|&row| is_grid_line(row, grid.rows)) {
        for column in 0..grid.columns.saturating_sub(1) {
            if let Some(segment) = clipped_segment(
                grid_point(grid, column, row),
                grid_point(grid, column + 1, row),
                grid.x_edge_boundary(column, row),
            ) {
                push_line(&mut lines, segment, color);
            }
        }
    }
    for column in (0..grid.columns).filter(|&column| is_grid_line(column, grid.columns)) {
        for row in 0..grid.rows.saturating_sub(1) {
            if let Some(segment) = clipped_segment(
                grid_point(grid, column, row),
                grid_point(grid, column, row + 1),
                grid.y_edge_boundary(column, row),
            ) {
                push_line(&mut lines, segment, color);
            }
        }
    }
    lines
}

/// 包絡面の接触境界（輪郭線）。
///
/// セル内部の境界は marching squares で、評価範囲の外周に達する接触領域は外周線で閉じる。
fn envelope_outline_lines(
    grid: &InverseOffsetEnvelopeGrid,
    color: [f32; 3],
) -> Vec<WireframeVertex> {
    let mut lines = Vec::new();
    for row in 0..grid.rows.saturating_sub(1) {
        for column in 0..grid.columns.saturating_sub(1) {
            for segment in CellRing::new(grid, column, row).outline_segments() {
                push_line(&mut lines, segment, color);
            }
        }
    }

    let last_row = grid.rows.saturating_sub(1);
    let last_column = grid.columns.saturating_sub(1);
    for row in [0, last_row] {
        for column in 0..last_column {
            if let Some(segment) = clipped_segment(
                grid_point(grid, column, row),
                grid_point(grid, column + 1, row),
                grid.x_edge_boundary(column, row),
            ) {
                push_line(&mut lines, segment, color);
            }
        }
    }
    for column in [0, last_column] {
        for row in 0..last_row {
            if let Some(segment) = clipped_segment(
                grid_point(grid, column, row),
                grid_point(grid, column, row + 1),
                grid.y_edge_boundary(column, row),
            ) {
                push_line(&mut lines, segment, color);
            }
        }
    }
    lines
}

/// 三角形メッシュの一意な辺を線分化する。
fn triangle_edge_lines(
    vertices: &[[f64; 3]],
    triangles: &[[usize; 3]],
    color: [f32; 3],
) -> Vec<WireframeVertex> {
    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    for tri in triangles {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            if seen.insert((a.min(b), a.max(b))) {
                lines.push(WireframeVertex::new(to_f32(vertices[a]), color));
                lines.push(WireframeVertex::new(to_f32(vertices[b]), color));
            }
        }
    }
    lines
}

/// 三角形ごとに面法線（上向き）を持つ独立頂点メッシュを作る。
fn flat_shaded_mesh(triangles: &[[[f64; 3]; 3]]) -> (Vec<VertexData>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(triangles.len() * 3);
    for [a, b, c] in triangles {
        let normal = upward_normal(*a, *b, *c);
        for p in [a, b, c] {
            vertices.push(VertexData::new(to_f32(*p), normal));
        }
    }
    let indices = (0..vertices.len() as u32).collect();
    (vertices, indices)
}

fn upward_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if length == 0.0 {
        return [0.0, 0.0, 1.0];
    }
    let sign = if n[2] < 0.0 { -1.0 } else { 1.0 };
    to_f32([
        sign * n[0] / length,
        sign * n[1] / length,
        sign * n[2] / length,
    ])
}

fn to_f32(p: [f64; 3]) -> [f32; 3] {
    [p[0] as f32, p[1] as f32, p[2] as f32]
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHORD_TOLERANCE: f64 = 0.001;

    fn build(
        tool_kind: InspectionToolKind,
        mode: InverseOffsetDisplayMode,
    ) -> InverseOffsetVisualization {
        build_sample_inverse_offset_visualization(
            tool_kind,
            mode,
            CHORD_TOLERANCE,
            &InverseOffsetVisualizationSettings::default(),
        )
        .unwrap()
    }

    #[test]
    fn envelope_surface_mode_shades_envelope_and_overlays_source_edges() {
        let view = build(
            InspectionToolKind::BallEndMill,
            InverseOffsetDisplayMode::EnvelopeSurface,
        );
        assert!(!view.mesh_vertices.is_empty());
        assert_eq!(view.mesh_vertices.len(), view.mesh_indices.len());
        assert_eq!(view.overlay_lines.len() % 2, 0);
        // 包絡面はボール中心（先端 + r）で表す
        assert_eq!(view.reference_offset, 0.1);
        // 面法線は上向き
        assert!(view.mesh_vertices.iter().all(|v| v.normal[2] >= 0.0));
    }

    #[test]
    fn envelope_grid_mode_draws_lines_at_fixed_pitch() {
        let settings = InverseOffsetVisualizationSettings::default();
        let view = build(
            InspectionToolKind::FlatEndMill,
            InverseOffsetDisplayMode::EnvelopeGrid,
        );
        let (columns, rows) = view.envelope_samples;

        // 格子線は X/Y 方向とも interval ごと（外周含む）に引かれる
        let lines_per_axis = |count: usize| {
            (0..count)
                .filter(|&i| i.is_multiple_of(settings.grid_line_interval) || i + 1 == count)
                .count()
        };
        let y_values: HashSet<i64> = view
            .overlay_lines
            .chunks(2)
            .filter(|segment| segment[0].color == settings.envelope_line_color)
            .filter(|segment| segment[0].position[1] == segment[1].position[1])
            .map(|segment| (segment[0].position[1] as f64 / settings.sample_pitch).round() as i64)
            .collect();
        assert!(!y_values.is_empty());
        assert_eq!(y_values.len(), lines_per_axis(rows));
        assert!(columns > settings.grid_line_interval);
        assert!(view.overlay_lines.iter().all(|v| {
            v.color == settings.envelope_line_color || v.color == settings.envelope_outline_color
        }));
    }

    /// 格子の角領域（x < 0, y < 0）にある線分端点
    fn corner_points(view: &InverseOffsetVisualization, color: [f32; 3]) -> Vec<[f32; 3]> {
        view.overlay_lines
            .iter()
            .filter(|v| v.color == color && v.position[0] < 0.0 && v.position[1] < 0.0)
            .map(|v| v.position)
            .collect()
    }

    #[test]
    fn outline_and_grid_lines_reach_rounded_corner_boundary() {
        // サンプル曲面の角 (0,0) 外側では、包絡面の境界は角から半径 r の円弧になる
        let settings = InverseOffsetVisualizationSettings::default();
        let r = settings.tool_radius as f32;
        let tolerance = settings.sample_pitch as f32;
        for kind in [
            InspectionToolKind::BallEndMill,
            InspectionToolKind::FlatEndMill,
        ] {
            let view = build(kind, InverseOffsetDisplayMode::EnvelopeGrid);

            let outline = corner_points(&view, settings.envelope_outline_color);
            assert!(!outline.is_empty(), "{kind:?}: corner outline is missing");
            for [x, y, _] in &outline {
                let distance = (x * x + y * y).sqrt();
                assert!(
                    (distance - r).abs() <= tolerance,
                    "{kind:?}: outline point ({x}, {y}) is {distance} from corner"
                );
            }

            // 格子線は円弧まで延びる（角領域内の格子線端点のうち最も遠い点が r 付近に達する）
            let grid_far = corner_points(&view, settings.envelope_line_color)
                .iter()
                .map(|[x, y, _]| (x * x + y * y).sqrt())
                .fold(0.0_f32, f32::max);
            assert!(
                (grid_far - r).abs() <= tolerance,
                "{kind:?}: grid lines stop at {grid_far}"
            );
        }
    }

    #[test]
    fn envelope_surface_covers_rounded_corner_up_to_boundary() {
        let settings = InverseOffsetVisualizationSettings::default();
        let r = settings.tool_radius as f32;
        let view = build(
            InspectionToolKind::FlatEndMill,
            InverseOffsetDisplayMode::EnvelopeSurface,
        );
        let corner_vertices: Vec<f32> = view
            .mesh_vertices
            .iter()
            .filter(|v| v.position[0] < 0.0 && v.position[1] < 0.0)
            .map(|v| (v.position[0].powi(2) + v.position[1].powi(2)).sqrt())
            .collect();
        assert!(!corner_vertices.is_empty());
        // 角領域の包絡面は円弧の内側に収まり、円弧まで達する
        let tolerance = settings.sample_pitch as f32;
        assert!(corner_vertices.iter().all(|&d| d <= r + tolerance));
        let farthest = corner_vertices.iter().copied().fold(0.0_f32, f32::max);
        assert!(
            (farthest - r).abs() <= tolerance,
            "surface reaches {farthest}"
        );
    }

    #[test]
    fn zero_grid_line_interval_is_rejected() {
        let settings = InverseOffsetVisualizationSettings {
            grid_line_interval: 0,
            ..Default::default()
        };
        let error = build_sample_inverse_offset_visualization(
            InspectionToolKind::BallEndMill,
            InverseOffsetDisplayMode::EnvelopeGrid,
            CHORD_TOLERANCE,
            &settings,
        )
        .unwrap_err();
        assert_eq!(
            error,
            InverseOffsetVisualizationError::InvalidGridLineInterval
        );
    }
}
