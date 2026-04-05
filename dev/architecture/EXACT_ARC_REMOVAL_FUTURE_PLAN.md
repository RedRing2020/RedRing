# 正確な円弧除去アルゴリズム実装計画（将来課題）

**作成日**: 2026年2月12日  
**優先度**: 低（差別化要素として検討）  
**関連Issue**: 未作成（本ドキュメント後に作成）

## 概要

現在実装済みの線分近似版（`remove_material_arc_polyline()`）は実用十分な精度（16線分で誤差0.5%）を提供していますが、以下のケースで正確な円弧除去が優位性を持つ可能性があります：

1. **超高精度加工**: 航空宇宙・医療機器など±0.01mm以下の公差が必要な場合
2. **CAMエンジン差別化**: 商用CAMソフトとの技術的優位性を示す場合
3. **研究開発**: 幾何計算アルゴリズムの研究プラットフォームとして

## 技術的課題

### 1. 円弧-AABB間距離計算

最も複雑な部分。以下のステップが必要：

```rust
/// 円弧からAABBまでの最短距離を計算
///
/// # Algorithm
///
/// 1. 円弧を含む平面とAABBの位置関係を判定
/// 2. AABBの8頂点を円弧平面に投影
/// 3. 投影点が円弧の角度範囲内かチェック
/// 4. 範囲内なら投影点までの距離、範囲外なら端点までの距離
fn arc_to_aabb_distance<T: Scalar>(
    arc: &Arc3D<T>,
    aabb: &Aabb3D<T>
) -> T {
    // 実装規模: 約200-300行のコード
    // テストケース: 約50-100行
    todo!()
}
```

**実装工数**: 約20-30時間

### 2. VoxelOctreeへの統合

```rust
impl<T: Scalar> VoxelNode<T> {
    fn remove_material_arc_exact(
        &mut self,
        arc: &Arc3D<T>,
        radius: T,
        max_depth: usize
    ) {
        match self.state {
            VoxelState::Empty => (),
            
            VoxelState::Solid => {
                // 正確な距離計算による枝刈り
                let distance = arc_to_aabb_distance(arc, &self.bounds);
                
                if distance > radius {
                    return; // 範囲外
                }
                
                // 完全包含判定
                if self.is_aabb_inside_arc_sweep_exact(arc, radius) {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }
                
                // 部分的交差 → 細分化
                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_arc_exact(arc, radius, max_depth);
                    }
                } else {
                    self.state = VoxelState::Empty;
                }
            }
            
            VoxelState::Mixed => {
                // 子ノードに再帰
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_arc_exact(arc, radius, max_depth);
                    }
                    self.update_state_from_children();
                }
            }
        }
    }
}
```

**実装工数**: 約10-15時間

### 3. テスト・検証

- 単体テスト: 円弧-AABB距離計算の正確性検証
- 統合テスト: VoxelOctree全体での動作確認
- ベンチマーク: 線分近似版との性能比較
- 精度検証: 理論値との比較

**実装工数**: 約10-15時間

## パフォーマンス予測

### 計算量分析

| 指標                | 線分近似（16線分） | 正確な円弧       |
|---------------------|-------------------|-----------------|
| 距離計算の複雑度     | O(1) × 16回       | O(1) × 1回      |
| 枝刈り効率          | 中程度            | 高い（正確な距離）|
| 総計算時間（予測）   | 基準              | 0.5-0.8倍       |

### パフォーマンステスト設計

```rust
#[test]
fn bench_arc_removal_performance() {
    let bounds = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    
    let arc = Arc3D::xy_arc(
        Point3D::new(50.0, 50.0, 50.0),
        30.0,
        Angle::from_degrees(0.0),
        Angle::from_degrees(180.0),
    ).unwrap();
    
    // 線分近似版
    let start_polyline = std::time::Instant::now();
    let mut tree_polyline = VoxelOctree::new(bounds, 6);
    tree_polyline.remove_material_arc_polyline(&arc, 10.0, 16);
    let time_polyline = start_polyline.elapsed();
    
    // 正確版
    let start_exact = std::time::Instant::now();
    let mut tree_exact = VoxelOctree::new(bounds, 6);
    tree_exact.remove_material_arc_exact(&arc, 10.0);
    let time_exact = start_exact.elapsed();
    
    println!("線分近似: {:?}", time_polyline);
    println!("正確版:   {:?}", time_exact);
    println!("速度比:   {:.2}x", 
             time_polyline.as_secs_f64() / time_exact.as_secs_f64());
             
    // 精度比較
    let diff = (tree_polyline.remaining_volume() - tree_exact.remaining_volume()).abs();
    println!("体積差:   {:.2} mm³", diff);
}
```

## 実装の優先順位判定基準

以下の条件を**全て**満たす場合のみ、正確版の実装を検討：

### 必須条件

1. **ユーザー要望**: 実際のユーザーから高精度円弧処理の要望がある
2. **パフォーマンス問題**: 線分近似版がボトルネックになっている（プロファイリングで確認）
3. **差別化価値**: 商用CAMソフトとの技術的差別化が明確に示せる

### 推奨条件（いずれか1つ）

- 研究論文・学会発表での技術デモとして活用
- 高精度加工の実際のユースケース（航空宇宙・医療機器など）
- CAMエンジンとしての製品化計画

## 代替案・補完策

正確版を実装しない場合の補完策：

### 1. 線分数の動的調整

```rust
/// 円弧の特性に応じて線分数を自動調整
pub fn remove_material_arc_adaptive(
    &mut self,
    arc: &Arc3D<T>,
    radius: T,
    target_error: T  // 目標誤差（例: 0.01mm）
) {
    // 必要な線分数を計算
    let arc_length = arc.length();
    let curvature = T::ONE / arc.radius();
    
    // 誤差式: error ≈ r * (1 - cos(θ/2))
    // 目標誤差から必要な線分数を逆算
    let required_segments = calculate_segments_for_error(
        arc.radius(), 
        arc.angle_span(), 
        target_error
    );
    
    self.remove_material_arc_polyline(arc, radius, required_segments);
}
```

### 2. ドキュメント強化

- 線分近似の精度特性を明確に記載
- パラメータ選択ガイドの充実
- 誤差計算式の提供

### 3. 他の曲線型への対応

正確版よりも優先度が高い：

- **NURBS曲線**: 同様の線分近似で対応可能
- **Bスプライン**: 同様の線分近似で対応可能
- **複合経路**: G-codeパーサーとの統合

## Issue作成時のテンプレート

将来的にIssueを作成する場合のテンプレート：

```markdown
# 正確な円弧除去アルゴリズム実装

## 背景

現在の線分近似版（`remove_material_arc_polyline()`）は実用十分な精度を提供していますが、
以下の理由で正確な円弧処理の実装を検討します：

[具体的な理由を記載]

## 実装内容

### Phase 1: 円弧-AABB距離計算（geo_commons）

- [ ] `arc_to_aabb_distance()` 実装
- [ ] 単体テスト（20ケース以上）
- [ ] ドキュメント整備

### Phase 2: VoxelOctree統合

- [ ] `VoxelNode::remove_material_arc_exact()` 実装
- [ ] `VoxelOctree::remove_material_arc_exact()` 公開API
- [ ] 統合テスト

### Phase 3: パフォーマンス検証

- [ ] ベンチマーク実装
- [ ] 線分近似版との比較
- [ ] 精度評価

### Phase 4: ドキュメント・使用例

- [ ] Rustdoc コメント
- [ ] 使用例追加
- [ ] パフォーマンスガイド

## 期待効果

- **精度向上**: 線分近似の誤差を完全に排除
- **差別化**: 商用CAMソフトとの技術的優位性
- **研究価値**: 幾何計算アルゴリズムの実装例として

## 実装工数

合計: **40-60時間**

## 優先度

**低** - 現在の線分近似版で実用上十分な精度を提供しているため、
他の優先度の高い機能（NURBS対応、トポロジー構築など）を優先。

## 関連文書

- `dev/architecture/ARC_INTERPOLATION_CUTTING_SIMULATION_RESEARCH.md`
- `dev/architecture/EXACT_ARC_REMOVAL_FUTURE_PLAN.md`（本ドキュメント）
```

## まとめ

正確な円弧除去は技術的に実装可能であり、差別化要素として価値がありますが、
現時点では以下の理由で**優先度は低い**と判断します：

1. **実用十分な精度**: 線分近似版で一般的なCNC加工公差（±0.05mm）を満たす
2. **高い実装コスト**: 40-60時間の工数が必要
3. **他の優先課題**: NURBS対応、トポロジー構築など、より影響の大きい機能が存在

**推奨アクション**:
- 現状は線分近似版で運用
- ユーザーからの要望や差別化の必要性が明確になった時点で再検討
- その間、線分近似版の品質向上（ドキュメント、適応的線分数調整など）に注力

## 更新履歴

- 2026-02-12: 初回作成（将来の正確版実装計画）
