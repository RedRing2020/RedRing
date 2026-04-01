//! octree_converter - Octree から GPU用ワイヤーフレームデータへの変換
//!
//! MVVMアーキテクチャにおけるViewModel層の責務として、
//! geo_algorithms の Octree を GPU レンダリング用のワイヤーフレーム頂点データに変換します。

use geo_algorithms::{
    octree::{Octree, OctreeTolerance, VoxelOctree, VoxelState},
    Aabb3D,
};
use geo_contracts::Scalar;
use std::collections::HashSet;
use std::ops::Range;

/// GPU用ワイヤーフレーム頂点データ（renderクレートのVertex3Dと同じ構造）
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireframeVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl WireframeVertex {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}

/// Octree可視化オプション
#[derive(Clone, Debug)]
pub struct OctreeVisualizationOptions {
    /// 表示する深さ範囲（例: 0..3 は深さ0,1,2を表示）
    pub depth_range: Range<usize>,

    /// 深さによる色分けを有効化
    pub color_by_depth: bool,

    /// 空ノードをフィルタリング
    pub filter_empty: bool,

    /// 最大深さ（カラーマップ正規化用）
    pub max_depth: usize,
}

impl Default for OctreeVisualizationOptions {
    fn default() -> Self {
        Self {
            depth_range: 0..8,
            color_by_depth: true,
            filter_empty: true,
            max_depth: 8,
        }
    }
}

/// VoxelOctree可視化オプション
#[derive(Clone, Debug)]
pub struct VoxelVisualizationOptions {
    /// 表示する深さ範囲
    pub depth_range: Range<usize>,

    /// 表示する状態（Solid, Mixed, Empty）
    pub show_states: Vec<VoxelState>,

    /// 状態による色分けを有効化
    pub color_by_state: bool,

    /// 深さによる色分けを有効化（状態色分けが無効の場合）
    pub color_by_depth: bool,

    /// 最大深さ（カラーマップ正規化用）
    pub max_depth: usize,
}

/// Octreeデバッグ可視化設定（View側から保持・注入する想定）
#[derive(Clone, Debug)]
pub struct OctreeVisualizationSettings {
    /// 可視化する最大深さ
    pub max_depth: usize,

    /// 深さグラデーション開始色（浅い）
    pub gradient_start: [f32; 3],

    /// 深さグラデーション終了色（深い）
    pub gradient_end: [f32; 3],

    /// Octreeトレランス（将来のUI設定連携用）
    pub octree_tolerance: OctreeTolerance<f64>,
}

impl Default for OctreeVisualizationSettings {
    fn default() -> Self {
        Self {
            max_depth: 4,
            gradient_start: [0.2, 1.0, 1.0],
            gradient_end: [1.0, 0.4, 0.4],
            octree_tolerance: OctreeTolerance::default(),
        }
    }
}

impl Default for VoxelVisualizationOptions {
    fn default() -> Self {
        Self {
            depth_range: 0..8,
            show_states: vec![VoxelState::Solid, VoxelState::Mixed],
            color_by_state: true,
            color_by_depth: false,
            max_depth: 8,
        }
    }
}

/// 深さに基づいた色を計算（青→シアン→緑→黄→赤）
fn depth_to_color(depth: usize, max_depth: usize) -> [f32; 3] {
    let t = if max_depth == 0 {
        0.0
    } else {
        (depth as f32) / (max_depth as f32)
    };

    // HSVライクなカラーマップ
    // t=0.0: 青 (240°), t=1.0: 赤 (0°)
    let hue = (1.0 - t) * 240.0; // 240° → 0°

    // HSV to RGB変換（簡易版）
    let h = hue / 60.0;
    let i = h.floor() as i32;
    let f = h - i as f32;

    let v = 1.0; // Value = 1.0
    let s = 0.8; // Saturation = 0.8

    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t_val = v * (1.0 - s * (1.0 - f));

    match i {
        0 => [v, t_val, p],
        1 => [q, v, p],
        2 => [p, v, t_val],
        3 => [p, q, v],
        4 => [t_val, p, v],
        _ => [v, p, q],
    }
}

/// VoxelStateに基づいた色を計算
fn state_to_color(state: VoxelState) -> [f32; 3] {
    match state {
        VoxelState::Solid => [0.2, 0.4, 1.0], // 青
        VoxelState::Mixed => [1.0, 0.8, 0.0], // 黄
        VoxelState::Empty => [0.3, 0.3, 0.3], // グレー
    }
}

/// 境界ボックスをワイヤーフレーム頂点に変換（12辺 = 24頂点）
fn bbox_to_wireframe_vertices<T: Scalar>(
    bbox: &Aabb3D<T>,
    color: [f32; 3],
) -> Vec<WireframeVertex> {
    let min = bbox.min();
    let max = bbox.max();

    // 8頂点
    let v000 = [min.x().to_f32(), min.y().to_f32(), min.z().to_f32()];
    let v001 = [min.x().to_f32(), min.y().to_f32(), max.z().to_f32()];
    let v010 = [min.x().to_f32(), max.y().to_f32(), min.z().to_f32()];
    let v011 = [min.x().to_f32(), max.y().to_f32(), max.z().to_f32()];
    let v100 = [max.x().to_f32(), min.y().to_f32(), min.z().to_f32()];
    let v101 = [max.x().to_f32(), min.y().to_f32(), max.z().to_f32()];
    let v110 = [max.x().to_f32(), max.y().to_f32(), min.z().to_f32()];
    let v111 = [max.x().to_f32(), max.y().to_f32(), max.z().to_f32()];

    // 12辺をLineList形式で返す（連続する2頂点で1辺）
    vec![
        // 底面 (z = min)
        WireframeVertex::new(v000, color),
        WireframeVertex::new(v100, color),
        WireframeVertex::new(v100, color),
        WireframeVertex::new(v110, color),
        WireframeVertex::new(v110, color),
        WireframeVertex::new(v010, color),
        WireframeVertex::new(v010, color),
        WireframeVertex::new(v000, color),
        // 上面 (z = max)
        WireframeVertex::new(v001, color),
        WireframeVertex::new(v101, color),
        WireframeVertex::new(v101, color),
        WireframeVertex::new(v111, color),
        WireframeVertex::new(v111, color),
        WireframeVertex::new(v011, color),
        WireframeVertex::new(v011, color),
        WireframeVertex::new(v001, color),
        // 垂直辺（4本）
        WireframeVertex::new(v000, color),
        WireframeVertex::new(v001, color),
        WireframeVertex::new(v100, color),
        WireframeVertex::new(v101, color),
        WireframeVertex::new(v110, color),
        WireframeVertex::new(v111, color),
        WireframeVertex::new(v010, color),
        WireframeVertex::new(v011, color),
    ]
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

fn canonical_edge_key(a: [f32; 3], b: [f32; 3]) -> ([u32; 3], [u32; 3]) {
    let pa = point_key(a);
    let pb = point_key(b);
    if pa <= pb {
        (pa, pb)
    } else {
        (pb, pa)
    }
}

fn deduplicate_exact_edges(vertices: Vec<WireframeVertex>) -> Vec<WireframeVertex> {
    let mut unique_edges: HashSet<([u32; 3], [u32; 3])> = HashSet::new();
    let mut deduped = Vec::with_capacity(vertices.len());

    for edge in vertices.chunks_exact(2) {
        let a = edge[0];
        let b = edge[1];
        let key = canonical_edge_key(a.position, b.position);

        if unique_edges.insert(key) {
            deduped.push(a);
            deduped.push(b);
        }
    }

    deduped
}

/// OctreeをGPU用ワイヤーフレーム頂点に変換
///
/// # Arguments
/// * `octree` - 変換するOctree
/// * `options` - 可視化オプション
///
/// # Returns
/// LineList形式の頂点配列（連続する2頂点で1辺）
pub fn octree_to_wireframe<T: Scalar, D: Clone>(
    octree: &Octree<T, D>,
    options: &OctreeVisualizationOptions,
) -> Vec<WireframeVertex> {
    let mut vertices = Vec::new();

    // ルートノードから再帰的に走査
    traverse_octree(
        octree,
        0, // depth
        octree.bounds(),
        options,
        &mut vertices,
    );

    vertices
}

/// Octreeを再帰的に走査してワイヤーフレーム頂点を生成
fn traverse_octree<T: Scalar, D: Clone>(
    _octree: &Octree<T, D>,
    depth: usize,
    bounds: &Aabb3D<T>,
    options: &OctreeVisualizationOptions,
    vertices: &mut Vec<WireframeVertex>,
) {
    // 深さフィルタリング
    if !options.depth_range.contains(&depth) {
        return;
    }

    // 色を計算
    let color = if options.color_by_depth {
        depth_to_color(depth, options.max_depth)
    } else {
        [1.0, 1.0, 1.0] // 白
    };

    // ノードの境界ボックスをワイヤーフレームに変換
    let mut bbox_vertices = bbox_to_wireframe_vertices(bounds, color);
    vertices.append(&mut bbox_vertices);

    // 子ノードを走査（Octreeの内部実装に依存）
    // 注: 現在のOctree実装では子ノードへの直接アクセスAPIが無いため、
    // この実装は簡略化されています。実際にはOctreeに子ノードイテレータが必要です。
}

/// VoxelOctreeをGPU用ワイヤーフレーム頂点に変換
///
/// # Arguments
/// * `voxel_octree` - 変換するVoxelOctree
/// * `options` - 可視化オプション
///
/// # Returns
/// LineList形式の頂点配列（連続する2頂点で1辺）
pub fn voxel_octree_to_wireframe<T: Scalar>(
    voxel_octree: &VoxelOctree<T>,
    options: &VoxelVisualizationOptions,
) -> Vec<WireframeVertex> {
    let mut vertices = Vec::new();

    tracing::debug!("voxel_octree_to_wireframe: 開始");

    // 可視深さまでの非Emptyボクセル境界ボックスを取得
    let visible_depth = options
        .depth_range
        .end
        .saturating_sub(1)
        .min(voxel_octree.max_depth());
    let solid_bounds = voxel_octree.collect_non_empty_voxel_bounds_up_to_depth(visible_depth);

    tracing::info!(
        "voxel_octree_to_wireframe: {} Solidボクセル検出",
        solid_bounds.len()
    );

    // 各ボクセル境界ボックスをワイヤーフレーム化
    let depth_hint = visible_depth;
    let color = if options.color_by_state {
        state_to_color(VoxelState::Solid)
    } else if options.color_by_depth {
        depth_to_color(depth_hint, options.max_depth.max(1))
    } else {
        [1.0, 1.0, 1.0]
    };

    for bounds in solid_bounds {
        let mut bbox_vertices = bbox_to_wireframe_vertices(&bounds, color);
        vertices.append(&mut bbox_vertices);
    }

    let before = vertices.len();
    let deduped = deduplicate_exact_edges(vertices);
    let removed_edges = (before.saturating_sub(deduped.len())) / 2;

    tracing::info!(
        "voxel_octree_to_wireframe: {} 頂点 → {} 頂点（重複辺 {} 本削除）",
        before,
        deduped.len(),
        removed_edges
    );

    deduped
}

pub fn voxel_octree_to_wireframe_positions<T: Scalar>(
    voxel_octree: &VoxelOctree<T>,
    options: &VoxelVisualizationOptions,
) -> Vec<[f32; 3]> {
    voxel_octree_to_wireframe(voxel_octree, options)
        .iter()
        .map(|vertex| vertex.position)
        .collect()
}

pub fn voxel_octree_to_wireframe_colored_levels_with_settings<T: Scalar>(
    voxel_octree: &VoxelOctree<T>,
    settings: &OctreeVisualizationSettings,
) -> Vec<Vec<WireframeVertex>> {
    let mut levels = Vec::new();
    let max_depth = settings.max_depth;

    let depth_color = |depth: usize, max_depth: usize| {
        if max_depth == 0 {
            return settings.gradient_start;
        }
        let t = (depth as f32 / max_depth as f32).clamp(0.0, 1.0);
        [
            settings.gradient_start[0]
                + (settings.gradient_end[0] - settings.gradient_start[0]) * t,
            settings.gradient_start[1]
                + (settings.gradient_end[1] - settings.gradient_start[1]) * t,
            settings.gradient_start[2]
                + (settings.gradient_end[2] - settings.gradient_start[2]) * t,
        ]
    };

    for depth in 0..=max_depth {
        let options = VoxelVisualizationOptions {
            depth_range: 0..(depth + 1),
            show_states: vec![VoxelState::Solid],
            color_by_state: false,
            color_by_depth: true,
            max_depth: max_depth.max(1),
        };

        let mut wireframe_vertices = voxel_octree_to_wireframe(voxel_octree, &options);
        let color = depth_color(depth, max_depth.max(1));
        for vertex in &mut wireframe_vertices {
            vertex.color = color;
        }
        levels.push(wireframe_vertices);
    }

    tracing::info!(
        "create_sample_voxel_octree_wireframe_colored_levels_with_settings: {} レベル生成（0..{}）",
        levels.len(),
        max_depth
    );

    levels
}

/// デバッグ/教育用：サンプルVoxelOctreeワイヤーフレームデータを生成
pub fn create_sample_voxel_octree_wireframe() -> Vec<[f32; 3]> {
    tracing::info!("create_sample_voxel_octree_wireframe: 開始");

    let voxel_tree = geo_algorithms::octree::fixtures::create_sample_voxel_octree(3, 0.0);
    let options = VoxelVisualizationOptions {
        depth_range: 0..8,
        show_states: vec![VoxelState::Solid],
        color_by_state: true,
        color_by_depth: false,
        max_depth: 6,
    };
    let positions = voxel_octree_to_wireframe_positions(&voxel_tree, &options);

    tracing::info!(
        "create_sample_voxel_octree_wireframe: 完了 ({} positions)",
        positions.len()
    );

    positions
}

/// デバッグ用：深さごとのサンプルVoxelOctreeワイヤーフレームを生成
pub fn create_sample_voxel_octree_wireframe_levels(max_depth: usize) -> Vec<Vec<[f32; 3]>> {
    let colored_levels = create_sample_voxel_octree_wireframe_colored_levels(max_depth);
    colored_levels
        .iter()
        .map(|vertices| vertices.iter().map(|v| v.position).collect())
        .collect()
}

/// デバッグ用：深さごとのサンプルVoxelOctreeワイヤーフレーム（色付き）を生成
pub fn create_sample_voxel_octree_wireframe_colored_levels(
    max_depth: usize,
) -> Vec<Vec<WireframeVertex>> {
    let settings = OctreeVisualizationSettings {
        max_depth,
        ..OctreeVisualizationSettings::default()
    };
    create_sample_voxel_octree_wireframe_colored_levels_with_settings(&settings)
}

/// デバッグ用：設定付きで深さごとのサンプルVoxelOctreeワイヤーフレーム（色付き）を生成
pub fn create_sample_voxel_octree_wireframe_colored_levels_with_settings(
    settings: &OctreeVisualizationSettings,
) -> Vec<Vec<WireframeVertex>> {
    let voxel_tree = geo_algorithms::octree::fixtures::create_sample_voxel_octree(
        settings.max_depth,
        settings.octree_tolerance.query_expand,
    );
    voxel_octree_to_wireframe_colored_levels_with_settings(&voxel_tree, settings)
}

/// デバッグ用：掃引円柱除去サンプルの深さ別ワイヤーフレームを生成
pub fn create_sample_swept_cylinder_wireframe_colored_levels_with_settings(
    settings: &OctreeVisualizationSettings,
) -> Vec<Vec<WireframeVertex>> {
    let voxel_tree = geo_algorithms::octree::fixtures::create_sample_swept_cylinder_voxel_octree(
        settings.max_depth,
    );
    voxel_octree_to_wireframe_colored_levels_with_settings(&voxel_tree, settings)
}

/// デバッグ用 facade: 掃引円柱除去サンプルの深さ別ワイヤーフレームを生成
pub fn load_demo_swept_cylinder_wireframe_colored_levels_with_settings(
    settings: &OctreeVisualizationSettings,
) -> Vec<Vec<WireframeVertex>> {
    create_sample_swept_cylinder_wireframe_colored_levels_with_settings(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_algorithms::octree::fixtures::{
        create_sample_swept_cylinder_voxel_octree, create_sample_voxel_octree,
    };

    #[test]
    fn test_depth_to_color() {
        // 深さ0は青系
        let color0 = depth_to_color(0, 8);
        assert!(color0[2] > 0.5); // 青成分が大きい

        // 深さ最大は赤系
        let color_max = depth_to_color(8, 8);
        assert!(color_max[0] > 0.5); // 赤成分が大きい
    }

    #[test]
    fn test_state_to_color() {
        let solid_color = state_to_color(VoxelState::Solid);
        assert_eq!(solid_color, [0.2, 0.4, 1.0]); // 青

        let mixed_color = state_to_color(VoxelState::Mixed);
        assert_eq!(mixed_color, [1.0, 0.8, 0.0]); // 黄

        let empty_color = state_to_color(VoxelState::Empty);
        assert_eq!(empty_color, [0.3, 0.3, 0.3]); // グレー
    }

    #[test]
    fn test_bbox_to_wireframe_vertices() {
        use geo_algorithms::Point3D;

        let bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0));
        let color = [1.0, 0.0, 0.0];

        let vertices = bbox_to_wireframe_vertices(&bbox, color);

        // 12辺 × 2頂点 = 24頂点
        assert_eq!(vertices.len(), 24);

        // 全頂点が同じ色
        for vertex in &vertices {
            assert_eq!(vertex.color, color);
        }
    }

    #[test]
    fn test_octree_visualization_options_default() {
        let options = OctreeVisualizationOptions::default();
        assert_eq!(options.depth_range, 0..8);
        assert!(options.color_by_depth);
        assert!(options.filter_empty);
        assert_eq!(options.max_depth, 8);
    }

    #[test]
    fn test_voxel_visualization_options_default() {
        let options = VoxelVisualizationOptions::default();
        assert_eq!(options.depth_range, 0..8);
        assert_eq!(options.show_states.len(), 2);
        assert!(options.color_by_state);
        assert_eq!(options.max_depth, 8);
    }

    #[test]
    fn test_voxel_octree_to_wireframe_positions() {
        let voxel_tree = create_sample_voxel_octree(3, 0.0);
        let options = VoxelVisualizationOptions {
            depth_range: 0..8,
            show_states: vec![VoxelState::Solid],
            color_by_state: true,
            color_by_depth: false,
            max_depth: 6,
        };
        let positions = voxel_octree_to_wireframe_positions(&voxel_tree, &options);

        // サンプルデータが生成されること
        assert!(!positions.is_empty(), "サンプルデータが生成されるべき");

        // LineList形式（2頂点 = 1辺）なので偶数であること
        assert_eq!(
            positions.len() % 2,
            0,
            "LineList形式では頂点数は偶数であるべき"
        );

        // 単一ボックス(12辺=24頂点)ではなく、複数ボクセルが可視化されること
        assert!(
            positions.len() > 24,
            "複数ボクセル可視化のため、24頂点を超えるべき（actual={})",
            positions.len()
        );

        // 全頂点が有効な座標値を持つこと
        for pos in &positions {
            assert!(
                pos[0].is_finite() && pos[1].is_finite() && pos[2].is_finite(),
                "全座標が有効な値であるべき"
            );
        }
    }

    #[test]
    fn test_voxel_octree_to_wireframe_colored_levels_with_settings() {
        let settings = OctreeVisualizationSettings {
            max_depth: 3,
            ..OctreeVisualizationSettings::default()
        };
        let voxel_tree = create_sample_voxel_octree(3, settings.octree_tolerance.query_expand);
        let levels = voxel_octree_to_wireframe_colored_levels_with_settings(&voxel_tree, &settings);
        assert_eq!(levels.len(), 4);
        assert!(levels.iter().any(|vertices| !vertices.is_empty()));
    }

    #[test]
    fn test_voxel_octree_to_wireframe_colored_levels_with_swept_cylinder_fixture() {
        let settings = OctreeVisualizationSettings {
            max_depth: 3,
            ..OctreeVisualizationSettings::default()
        };

        let voxel_tree = create_sample_swept_cylinder_voxel_octree(settings.max_depth);
        let levels = voxel_octree_to_wireframe_colored_levels_with_settings(&voxel_tree, &settings);
        assert_eq!(levels.len(), 4);
        assert!(levels.iter().any(|vertices| !vertices.is_empty()));
    }

    #[test]
    fn test_octree_debug_visualization_settings_default() {
        let settings = OctreeVisualizationSettings::default();
        assert_eq!(settings.max_depth, 4);
        assert!(settings.gradient_start[1] > settings.gradient_start[0]);
        assert!(settings.gradient_end[0] > settings.gradient_end[1]);
    }

    #[test]
    fn test_deduplicate_exact_edges_removes_reversed_and_identical_duplicates() {
        let color = [1.0, 1.0, 0.0];
        let vertices = vec![
            WireframeVertex::new([0.0, 0.0, 0.0], color),
            WireframeVertex::new([1.0, 0.0, 0.0], color),
            WireframeVertex::new([1.0, 0.0, 0.0], color),
            WireframeVertex::new([0.0, 0.0, 0.0], color),
            WireframeVertex::new([0.0, 0.0, 0.0], color),
            WireframeVertex::new([1.0, 0.0, 0.0], color),
        ];

        let deduped = deduplicate_exact_edges(vertices);

        assert_eq!(deduped.len(), 2, "同一辺は1本分のみ残るべき");
    }
}
