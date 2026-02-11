//! Octree基本使用例
//!
//! このサンプルは、Octreeによる空間分割と効率的な検索を実演します。
//!
//! ## 実行方法
//!
//! ```bash
//! cargo run -p geo_algorithms --example octree_basic
//! ```

use geo_algorithms::octree::{HasBoundingBox, HasPosition, Octree};
use geo_core::{Aabb3D, Point3D};

/// サンプル用の3D球形状
#[derive(Debug, Clone)]
struct Sphere {
    center: Point3D<f64>,
    radius: f64,
    id: usize,
}

impl Sphere {
    fn new(x: f64, y: f64, z: f64, radius: f64, id: usize) -> Self {
        Self {
            center: Point3D::new(x, y, z),
            radius,
            id,
        }
    }
}

impl HasBoundingBox<f64> for Sphere {
    fn bounding_box(&self) -> Aabb3D<f64> {
        Aabb3D::new(
            Point3D::new(
                self.center.x() - self.radius,
                self.center.y() - self.radius,
                self.center.z() - self.radius,
            ),
            Point3D::new(
                self.center.x() + self.radius,
                self.center.y() + self.radius,
                self.center.z() + self.radius,
            ),
        )
    }
}

impl HasPosition<f64> for Sphere {
    fn position(&self) -> Point3D<f64> {
        self.center
    }
}

fn main() {
    println!("=== Octree基本使用例 ===\n");

    // ========== 1. Octree作成 ==========
    println!("1. Octree作成");
    let scene_bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    let mut octree: Octree<f64, Sphere> = Octree::new(scene_bounds, 8, 10);
    println!("   境界: (0,0,0) - (100,100,100)");
    println!("   最大深さ: 8");
    println!("   ノードあたり最大要素数: 10\n");

    // ========== 2. データ挿入 ==========
    println!("2. データ挿入");
    
    // 100個の球を生成（簡易PRNG使用）
    let mut seed = 12345u64;
    for i in 0..100 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let x = ((seed % 10000) as f64) / 100.0; // 0-100
        
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let y = ((seed % 10000) as f64) / 100.0;
        
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let z = ((seed % 10000) as f64) / 100.0;
        
        let radius = 2.0 + (i % 5) as f64;
        
        octree.insert(Sphere::new(x, y, z, radius, i));
    }

    println!("   挿入: {} 個の球", octree.total_items());
    println!("   総ノード数: {}", octree.total_nodes());
    println!();

    // ========== 3. 範囲検索 ==========
    println!("3. 範囲検索");
    let query_region = Aabb3D::new(
        Point3D::new(10.0, 10.0, 10.0),
        Point3D::new(30.0, 30.0, 30.0),
    );
    
    let results = octree.query_region(&query_region);
    println!("   検索範囲: (10,10,10) - (30,30,30)");
    println!("   検出: {} 個の候補", results.len());
    
    if !results.is_empty() {
        println!("   最初の3つ:");
        for (i, sphere) in results.iter().take(3).enumerate() {
            println!(
                "     [{}] ID={}, 中心=({:.1}, {:.1}, {:.1}), 半径={:.1}",
                i,
                sphere.id,
                sphere.center.x(),
                sphere.center.y(),
                sphere.center.z(),
                sphere.radius
            );
        }
    }
    println!();

    // ========== 4. 最近傍探索 ==========
    println!("4. 最近傍探索");
    let query_point = Point3D::new(50.0, 50.0, 50.0);
    
    if let Some((nearest, distance)) = octree.nearest(&query_point) {
        println!("   検索点: (50.0, 50.0, 50.0)");
        println!(
            "   最近傍: ID={}, 中心=({:.1}, {:.1}, {:.1})",
            nearest.id,
            nearest.center.x(),
            nearest.center.y(),
            nearest.center.z()
        );
        println!("   距離: {:.2}", distance);
    } else {
        println!("   検索結果なし");
    }
    println!();

    // ========== 5. ノード走査 ==========
    println!("5. ノード走査（統計情報）");
    let mut depth_counts = vec![0usize; 10];
    let mut total_data = 0;
    
    octree.traverse(|node, _depth| {
        let d = node.depth();
        depth_counts[d] += 1;
        total_data += node.data().len();
    });

    println!("   深さ別ノード数:");
    for (depth, count) in depth_counts.iter().enumerate() {
        if *count > 0 {
            println!("     深さ {}: {} ノード", depth, count);
        }
    }
    println!("   総データ参照数: {}", total_data);
    println!();

    println!("=== 完了 ===");
}
