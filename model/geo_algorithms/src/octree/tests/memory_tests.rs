use super::*;

#[test]
fn test_memory_usage_estimation_regression() {
    let mut octree = Octree::new(
        default_octree_bounds(),
        OCTREE_DEFAULT_MAX_DEPTH,
        OCTREE_MEMORY_TEST_MAX_ITEMS,
    );

    let estimated_empty = estimate_octree_memory_bytes(&octree);
    assert!(estimated_empty > 0);

    for i in 0..PERFORMANCE_SAMPLE_COUNT {
        let x = (i % OCTREE_TEST_BOUNDS_MAX as usize) as f64 + 0.1;
        let y = ((i / 10) % OCTREE_TEST_BOUNDS_MAX as usize) as f64 + 0.2;
        let z = ((i / 100) % OCTREE_TEST_BOUNDS_MAX as usize) as f64 + 0.3;
        octree.insert(TestPoint {
            pos: Point3D::new(x, y, z),
        });
    }

    let estimated_populated = estimate_octree_memory_bytes(&octree);
    let bytes_per_item = estimated_populated as f64 / octree.total_items() as f64;

    eprintln!("=== メモリ使用量（概算）===");
    eprintln!("空状態: {} bytes", estimated_empty);
    eprintln!("挿入後: {} bytes", estimated_populated);
    eprintln!("総ノード数: {}", octree.total_nodes());
    eprintln!("総要素数: {}", octree.total_items());
    eprintln!("1要素あたり概算: {:.2} bytes", bytes_per_item);

    assert!(
        estimated_populated > estimated_empty,
        "挿入後の概算メモリ使用量が増加していません"
    );
    assert!(octree.total_nodes() > 1, "分割が発生していません");
    assert!(
        bytes_per_item < MEMORY_BYTES_PER_ITEM_MAX,
        "1要素あたりメモリが過大です"
    );

    octree.clear();
    let estimated_after_clear = estimate_octree_memory_bytes(&octree);
    assert_eq!(octree.total_items(), 0);
    assert_eq!(octree.total_nodes(), 1);
    assert_eq!(
        estimated_after_clear, estimated_empty,
        "clear後に概算メモリ使用量が初期値へ戻っていません"
    );
}