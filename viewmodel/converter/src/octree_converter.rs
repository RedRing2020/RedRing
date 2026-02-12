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
        VoxelState::Solid => [0.2, 0.4, 1.0],  // 青
        VoxelState::Mixed => [1.0, 0.8, 0.0],  // 黄
        VoxelState::Empty => [0.3, 0.3, 0.3],  // グレー
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
        &octree.bounds(),
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

    // VoxelOctreeの全ノードを走査
    traverse_voxel_octree(voxel_octree, 0, options, &mut vertices);

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
    let color = if options.color_by_state {
        state_to_color(VoxelState::Mixed) // デフォルト色
    } else if options.color_by_depth {
        depth_to_color(depth, options.max_depth)
    } else {
        [1.0, 1.0, 1.0]
    };

    let mut bbox_vertices = bbox_to_wireframe_vertices(&bounds, color);
    vertices.append(&mut bbox_vertices);
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
}
