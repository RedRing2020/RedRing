//! octree_converter - Octree から GPU用ワイヤーフレームデータへの変換
//!
//! MVVMアーキテクチャにおけるViewModel層の責務として、
//! geo_algorithms の Octree を GPU レンダリング用のワイヤーフレーム頂点データに変換します。

use geo_algorithms::octree::{Octree, VoxelOctree, VoxelState};
use geo_core::Aabb3D;
use geo_foundation::Scalar;
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

    // VoxelOctreeの全ノードを走査
    traverse_voxel_octree(voxel_octree, 0, options, &mut vertices);

    tracing::debug!("voxel_octree_to_wireframe: {} vertices", vertices.len());

    vertices
}

/// VoxelOctreeを再帰的に走査してワイヤーフレーム頂点を生成
fn traverse_voxel_octree<T: Scalar>(
    voxel_octree: &VoxelOctree<T>,
    depth: usize,
    options: &VoxelVisualizationOptions,
    vertices: &mut Vec<WireframeVertex>,
) {
    // 深さフィルタリング
    if !options.depth_range.contains(&depth) {
        return;
    }

    // VoxelOctreeを走査してボクセルノードを取得
    // 注: 現在のVoxelOctree実装では全ノードへのアクセスAPIが限定的なため、
    // この実装は簡略化されています。実際にはVoxelOctreeにノードイテレータが必要です。

    // ルートノードの境界ボックスを表示（プレースホルダー）
    let bounds = voxel_octree.bounds();

    tracing::debug!(
        "traverse_voxel_octree: depth={}, bounds=[{:.1},{:.1},{:.1}] - [{:.1},{:.1},{:.1}]",
        depth,
        bounds.min().x().to_f32(),
        bounds.min().y().to_f32(),
        bounds.min().z().to_f32(),
        bounds.max().x().to_f32(),
        bounds.max().y().to_f32(),
        bounds.max().z().to_f32()
    );

    let color = if options.color_by_state {
        state_to_color(VoxelState::Mixed) // デフォルト色
    } else if options.color_by_depth {
        depth_to_color(depth, options.max_depth)
    } else {
        [1.0, 1.0, 1.0]
    };

    let mut bbox_vertices = bbox_to_wireframe_vertices(bounds, color);
    vertices.append(&mut bbox_vertices);
}

/// デバッグ/教育用：サンプルVoxelOctreeワイヤーフレームデータを生成
///
/// 100x100x50mmのワークピースに簡単な切削例を作成し、
/// ワイヤーフレーム頂点データを返します。
///
/// # 生成される形状
/// - ワークピース: 100x100x50mm
/// - 外縁10mm除去
/// - 中央にポケット加工（直径10mm、深さ30mm）
///
/// # Returns
/// ワイヤーフレーム頂点の位置データ（[[f32; 3]]）
pub fn create_sample_voxel_octree_wireframe() -> Vec<[f32; 3]> {
    use geo_core::Point3D;

    tracing::info!("create_sample_voxel_octree_wireframe: 開始");

    // ワークピース設定（100x100x50mm）
    let work_bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 50.0),
    );

    let mut voxel_tree = VoxelOctree::new(work_bounds, 6);

    tracing::info!(
        "初期VoxelOctree: 体積={:.1} mm³",
        voxel_tree.remaining_volume()
    );

    // 簡単な切削例：外縁10mm除去
    let outline_region = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 10.0),
    );
    voxel_tree.remove_material_box(&outline_region);

    tracing::info!(
        "外縁除去後: 体積={:.1} mm³",
        voxel_tree.remaining_volume()
    );

    // 中央にポケット加工
    voxel_tree.remove_material_z_axis(50.0, 50.0, 10.0, 40.0, 10.0);

    tracing::info!(
        "ポケット加工後: 体積={:.1} mm³, Solidボクセル={}",
        voxel_tree.remaining_volume(),
        voxel_tree.solid_voxel_count()
    );

    // ViewModel で変換（ワイヤーフレーム頂点に）
    let options = VoxelVisualizationOptions {
        depth_range: 0..8,
        show_states: vec![VoxelState::Solid],
        color_by_state: true,
        color_by_depth: false,
        max_depth: 6,
    };

    let wireframe_vertices = voxel_octree_to_wireframe(&voxel_tree, &options);

    tracing::info!(
        "voxel_octree_to_wireframe: {} 頂点生成",
        wireframe_vertices.len()
    );

    // 頂点を [[f32; 3]] 配列に変換
    let positions: Vec<[f32; 3]> = wireframe_vertices.iter().map(|v| v.position).collect();

    // 最初の頂点をデバッグ出力
    if let Some(first) = positions.first() {
        tracing::info!("First vertex: [{:.1}, {:.1}, {:.1}]", first[0], first[1], first[2]);
    }

    tracing::info!("create_sample_voxel_octree_wireframe: 完了 ({} positions)", positions.len());

    positions
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let bbox = Aabb3D::new(
            geo_core::Point3D::new(0.0, 0.0, 0.0),
            geo_core::Point3D::new(1.0, 1.0, 1.0),
        );
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
        assert_eq!(options.color_by_depth, true);
        assert_eq!(options.filter_empty, true);
        assert_eq!(options.max_depth, 8);
    }

    #[test]
    fn test_voxel_visualization_options_default() {
        let options = VoxelVisualizationOptions::default();
        assert_eq!(options.depth_range, 0..8);
        assert_eq!(options.show_states.len(), 2);
        assert_eq!(options.color_by_state, true);
        assert_eq!(options.max_depth, 8);
    }

    #[test]
    fn test_create_sample_voxel_octree_wireframe() {
        let positions = create_sample_voxel_octree_wireframe();

        // サンプルデータが生成されること
        assert!(positions.len() > 0, "サンプルデータが生成されるべき");

        // LineList形式（2頂点 = 1辺）なので偶数であること
        assert_eq!(
            positions.len() % 2,
            0,
            "LineList形式では頂点数は偶数であるべき"
        );

        // 全頂点が有効な座標値を持つこと
        for pos in &positions {
            assert!(
                pos[0].is_finite() && pos[1].is_finite() && pos[2].is_finite(),
                "全座標が有効な値であるべき"
            );
        }
    }
}
