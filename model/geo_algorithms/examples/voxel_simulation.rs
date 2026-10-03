//! VoxelOctree 領域除去シミュレーション例
//!
//! このサンプルは、VoxelOctree による占有領域の除去と残存体積の計算を実演します。
//!
//! ## 実行方法
//!
//! ```bash
//! cargo run -p geo_algorithms --example voxel_simulation
//! ```

use geo_algorithms::octree::voxel::VoxelOctree;
use geo_algorithms::{Angle, Arc3D, LineSegment3D};
use geo_core::{Aabb3D, Point3D};

fn main() {
    println!("=== VoxelOctree 領域除去シミュレーション例 ===\n");

    println!("1. 占有領域の設定");
    let work_bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 50.0), // 100x100x50mm
    );

    let mut voxel_tree = VoxelOctree::new(work_bounds, 6);

    let initial_volume = voxel_tree.remaining_volume();
    println!("   サイズ: 100 x 100 x 50 mm");
    println!("   初期体積: {:.1} mm³", initial_volume);
    println!("   最大深さ: {}", voxel_tree.max_depth());
    println!(
        "   最小ボクセルサイズ: {:.3} mm",
        voxel_tree.voxel_size_at_max_depth()
    );
    println!();

    println!("2. 直方体領域の除去");

    // 下端から高さ 10 mm の層を除去
    let outline_region = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 10.0),
    );
    voxel_tree.remove_material_box(&outline_region);

    let volume_after_outline = voxel_tree.remaining_volume();
    let removed_outline = initial_volume - volume_after_outline;

    println!("   除去領域: XY 全体 x 高さ 10 mm");
    println!("   除去体積: {:.1} mm³", removed_outline);
    println!("   残存体積: {:.1} mm³", volume_after_outline);
    println!(
        "   除去率: {:.1}%",
        (removed_outline / initial_volume) * 100.0
    );
    println!();

    println!("3. Z軸方向の円柱領域の除去");

    // 中央の直径 20 mm・高さ 30 mm の円柱領域
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
    println!("   直径: Φ{} mm", pocket_radius * 2.0);
    println!("   Z範囲: {} - {} mm", pocket_z_start, pocket_z_end);
    println!("   除去体積: {:.1} mm³", removed_pocket);
    println!("   残存体積: {:.1} mm³", volume_after_pocket);
    println!();

    println!("4. 斜め方向のカプセル領域の除去");

    let segment = LineSegment3D::new(
        Point3D::new(20.0, 20.0, 10.0),
        Point3D::new(80.0, 80.0, 40.0),
    )
    .unwrap();
    let capsule_radius = 5.0;

    voxel_tree.remove_material_capsule(&segment, capsule_radius);

    let volume_after_diagonal = voxel_tree.remaining_volume();
    let removed_diagonal = volume_after_pocket - volume_after_diagonal;

    println!("   開始点: (20, 20, 10)");
    println!("   終了点: (80, 80, 40)");
    println!("   直径: Φ{} mm", capsule_radius * 2.0);
    println!("   除去体積: {:.1} mm³", removed_diagonal);
    println!("   残存体積: {:.1} mm³", volume_after_diagonal);
    println!();

    println!("4.5. 円弧に沿った掃引領域の除去");

    // XY平面上の90度円弧（中心: (25, 75), 半径: 15mm, Z: 15-25mm）
    let arc_center = Point3D::new(25.0, 75.0, 20.0);
    let arc_radius = 15.0;
    let arc_start_angle = Angle::from_degrees(0.0);
    let arc_end_angle = Angle::from_degrees(90.0);

    let arc = Arc3D::xy_arc(arc_center, arc_radius, arc_start_angle, arc_end_angle).unwrap();

    // 円弧を 16 線分で近似
    let sweep_radius = 4.0;
    let num_segments = 16;

    voxel_tree.remove_material_arc_polyline(&arc, sweep_radius, num_segments);

    let volume_after_arc = voxel_tree.remaining_volume();
    let removed_arc = volume_after_diagonal - volume_after_arc;

    println!(
        "   円弧中心: ({:.1}, {:.1}, {:.1})",
        arc_center.x(),
        arc_center.y(),
        arc_center.z()
    );
    println!("   半径: {} mm", arc_radius);
    println!(
        "   角度範囲: {}° - {}°",
        arc_start_angle.to_degrees(),
        arc_end_angle.to_degrees()
    );
    println!("   直径: Φ{} mm", sweep_radius * 2.0);
    println!("   近似線分数: {}", num_segments);
    println!("   除去体積: {:.1} mm³", removed_arc);
    println!("   残存体積: {:.1} mm³", volume_after_arc);

    // 円弧の長さ計算
    let arc_length = arc.length();
    println!("   円弧長: {:.2} mm", arc_length);
    println!();

    println!("5. 目的領域外の残存占有の検出");

    // 目的領域: 内側 80x80x30 mm。これ以外に残る占有を検出する
    let target_region = Aabb3D::new(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(90.0, 90.0, 40.0),
    );

    let undercuts = voxel_tree.detect_undercut(&target_region);

    println!("   目的領域: 内側80x80x30mm");
    println!("   検出された残存領域: {} 箇所", undercuts.len());

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

        // 残存領域の体積の概算
        let undercut_volume: f64 = undercuts.iter().map(|b| b.volume()).sum();
        println!("   残存領域の総体積（概算）: {:.1} mm³", undercut_volume);
    }
    println!();

    println!("6. 最終統計");
    let total_removed = initial_volume - volume_after_arc;
    let removal_rate = (total_removed / initial_volume) * 100.0;

    println!("   初期体積: {:.1} mm³", initial_volume);
    println!("   最終残存体積: {:.1} mm³", volume_after_arc);
    println!("   総除去体積: {:.1} mm³", total_removed);
    println!("   除去率: {:.1}%", removal_rate);
    println!("   Solidボクセル数: {}", voxel_tree.solid_voxel_count());
    println!();

    println!("=== 完了 ===");
}
