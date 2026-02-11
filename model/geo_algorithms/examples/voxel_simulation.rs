//! VoxelOctree切削シミュレーション例
//!
//! このサンプルは、VoxelOctreeによる材料除去シミュレーションを実演します。
//!
//! ## 実行方法
//!
//! ```bash
//! cargo run -p geo_algorithms --example voxel_simulation
//! ```

use geo_algorithms::octree::voxel::VoxelOctree;
use geo_core::{Aabb3D, Point3D};
use geo_primitives::LineSegment3D;

fn main() {
    println!("=== VoxelOctree切削シミュレーション例 ===\n");

    // ========== 1. ワークピース設定 ==========
    println!("1. ワークピース設定");
    let work_bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 50.0), // 100x100x50mm
    );
    
    let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    
    let initial_volume = voxel_tree.remaining_volume();
    println!("   サイズ: 100 x 100 x 50 mm");
    println!("   初期体積: {:.1} mm³", initial_volume);
    println!("   最大深さ: {}", voxel_tree.max_depth());
    println!("   最小ボクセルサイズ: {:.3} mm", voxel_tree.voxel_size_at_max_depth());
    println!();

    // ========== 2. 工具経路1: 外周切削 ==========
    println!("2. 外周切削（ボックス除去）");
    
    // 外側10mmを除去
    let outline_region = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 10.0),
    );
    voxel_tree.remove_material_box(&outline_region);
    
    let volume_after_outline = voxel_tree.remaining_volume();
    let removed_outline = initial_volume - volume_after_outline;
    
    println!("   除去領域: 全体 x 10mm深さ");
    println!("   除去体積: {:.1} mm³", removed_outline);
    println!("   残存体積: {:.1} mm³", volume_after_outline);
    println!("   除去率: {:.1}%", (removed_outline / initial_volume) * 100.0);
    println!();

    // ========== 3. 工具経路2: ポケット加工（Z軸） ==========
    println!("3. ポケット加工（Z軸工具）");
    
    // 中央に直径20mmのポケット（深さ30mm）
    let pocket_center_x = 50.0;
    let pocket_center_y = 50.0;
    let pocket_radius = 10.0; // 半径10mm
    let pocket_z_start = 10.0;
    let pocket_z_end = 40.0;
    
    voxel_tree.remove_material_z_axis(
        pocket_center_x,
        pocket_center_y,
        pocket_z_start,
        pocket_z_end,
        pocket_radius,
    );
    
    let volume_after_pocket = voxel_tree.remaining_volume();
    let removed_pocket = volume_after_outline - volume_after_pocket;
    
    println!("   位置: 中央 (50, 50)");
    println!("   工具径: Φ{} mm", pocket_radius * 2.0);
    println!("   深さ: {} - {} mm", pocket_z_start, pocket_z_end);
    println!("   除去体積: {:.1} mm³", removed_pocket);
    println!("   残存体積: {:.1} mm³", volume_after_pocket);
    println!();

    // ========== 4. 工具経路3: 斜め切削（カプセル） ==========
    println!("4. 斜め切削（5軸加工）");
    
    let segment = LineSegment3D::new(
        Point3D::new(20.0, 20.0, 10.0),
        Point3D::new(80.0, 80.0, 40.0),
    )
    .unwrap();
    let tool_radius = 5.0;
    
    voxel_tree.remove_material_capsule(&segment, tool_radius);
    
    let volume_after_diagonal = voxel_tree.remaining_volume();
    let removed_diagonal = volume_after_pocket - volume_after_diagonal;
    
    println!("   開始点: (20, 20, 10)");
    println!("   終了点: (80, 80, 40)");
    println!("   工具径: Φ{} mm", tool_radius * 2.0);
    println!("   除去体積: {:.1} mm³", removed_diagonal);
    println!("   残存体積: {:.1} mm³", volume_after_diagonal);
    println!();

    // ========== 5. 削り残し検出 ==========
    println!("5. 削り残し検出");
    
    // 目的形状: 内側80x80x30mmの領域以外は削り残し
    let target_region = Aabb3D::new(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(90.0, 90.0, 40.0),
    );
    
    let undercuts = voxel_tree.detect_undercut(&target_region);
    
    println!("   目的領域: 内側80x80x30mm");
    println!("   検出された削り残し: {} 箇所", undercuts.len());
    
    if !undercuts.is_empty() {
        println!("   最初の5箇所:");
        for (i, bbox) in undercuts.iter().take(5).enumerate() {
            let size_x = bbox.width();
            let size_y = bbox.height();
            let size_z = bbox.depth();
            println!(
                "     [{}] 位置: ({:.1}, {:.1}, {:.1}), サイズ: {:.2} x {:.2} x {:.2}",
                i,
                bbox.min().x(),
                bbox.min().y(),
                bbox.min().z(),
                size_x,
                size_y,
                size_z
            );
        }
        
        // 削り残し体積の概算
        let undercut_volume: f64 = undercuts.iter().map(|b| b.volume()).sum();
        println!("   削り残し総体積（概算）: {:.1} mm³", undercut_volume);
    }
    println!();

    // ========== 6. 最終統計 ==========
    println!("6. 最終統計");
    let total_removed = initial_volume - volume_after_diagonal;
    let removal_rate = (total_removed / initial_volume) * 100.0;
    
    println!("   初期体積: {:.1} mm³", initial_volume);
    println!("   最終残存体積: {:.1} mm³", volume_after_diagonal);
    println!("   総除去体積: {:.1} mm³", total_removed);
    println!("   除去率: {:.1}%", removal_rate);
    println!("   Solidボクセル数: {}", voxel_tree.solid_voxel_count());
    println!();

    println!("=== 完了 ===");
}
