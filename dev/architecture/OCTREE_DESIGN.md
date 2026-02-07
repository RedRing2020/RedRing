# Octree空間分割 設計・実装計画

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**ステータス**: 設計フェーズ  
**優先度**: 🟠 Tier 2（形状可視化完了後に着手）  
**関連Issue**: #207（Octree実装）, #208（Octree可視化）

---

## 📋 目次

1. [概要](#概要)
2. [背景・動機](#背景動機)
3. [技術設計](#技術設計)
4. [実装計画](#実装計画)
5. [可視化設計](#可視化設計)
6. [応用シナリオ](#応用シナリオ)

---

## 概要

Octree（八分木）は3D空間を再帰的に8つの子領域に分割する空間データ構造です。以下の機能に必須：

- **切削シミュレーション**: ワークの材料除去を高速計算
- **衝突判定高速化**: 大量形状の粗判定（Phase 3の拡張）
- **近傍検索**: 最近傍点・k近傍探索
- **デバッグ可視化**: Octree構造の視覚的確認

---

## 背景・動機

### なぜOctreeが必要か

#### 1. 切削シミュレーション

CAMシステムでは、**ワーク（素材）から材料を削る過程**をシミュレートします：

```text
初期状態:       切削後:
┌─────────┐    ┌─────────┐
│ワーク    │    │  ╱╲    │
│ (材料)  │ →  │ ╱  ╲   │ ← 削り取られた領域
│         │    │╱____╲  │
└─────────┘    └─────────┘
```

**問題**: 任意形状の削り取りを計算するには？

**Octreeによる解決**:
- ワークをボクセル（立方体セル）に分割
- 工具が通過した領域のボクセルを「削除」
- 適応的な分割により、高精度と高速性を両立

#### 2. 衝突判定の高速化

現在（Phase 3完了）の衝突判定：
```rust
// 全ペアの総当たり判定 O(n²)
for shape1 in shapes {
    for shape2 in shapes {
        if collide(shape1, shape2) { ... }
    }
}
```

**Octreeによる高速化**:
```rust
// 空間分割により O(n log n)
let octree = Octree::from_shapes(shapes);
for shape in shapes {
    let candidates = octree.query_region(shape.bbox());
    for candidate in candidates {
        if collide(shape, candidate) { ... }
    }
}
```

**効果**: 1000形状で 100万回 → 約1万回に削減（99%削減）

#### 3. 近傍検索

CAM演算でよく使う操作：
- 「工具位置から最も近い形状」を探索
- 「一定範囲内のすべての形状」を列挙

**Octreeなし**: O(n) - 全形状を線形探索  
**Octreeあり**: O(log n) - 空間分割により高速

---

## 技術設計

### 3.1 データ構造

#### 3.1.1 基本構造

```rust
// model/geo_algorithms/src/octree/mod.rs

use geo_primitives::{Point3D, BBox3D};
use geo_foundation::Scalar;

/// Octreeノード
pub struct OctreeNode<T: Scalar, D> {
    /// ノードの境界ボックス
    bounds: BBox3D<T>,
    
    /// 深さレベル（ルート=0）
    depth: usize,
    
    /// このノードに含まれるデータ
    data: Vec<D>,
    
    /// 子ノード（8個 or None）
    children: Option<Box<[OctreeNode<T, D>; 8]>>,
}

/// Octree全体の管理構造
pub struct Octree<T: Scalar, D> {
    root: OctreeNode<T, D>,
    max_depth: usize,       // 最大分割深さ
    max_items: usize,       // 分割閾値（ノードあたり）
    total_nodes: usize,     // 統計情報
    total_items: usize,
}
```

#### 3.1.2 子ノードの配置

8つの子ノードは以下の順序で配置：

```text
     Y
     ↑
    5───6
   /│  /│
  1─┼─2 │  Z
  │ 4─┼─7  ↗
  │/  │/
  0───3  → X

0: (x_min, y_min, z_min)  1: (x_min, y_max, z_min)
2: (x_max, y_max, z_min)  3: (x_max, y_min, z_min)
4: (x_min, y_min, z_max)  5: (x_min, y_max, z_max)
6: (x_max, y_max, z_max)  7: (x_max, y_min, z_max)
```

```rust
impl<T: Scalar, D> OctreeNode<T, D> {
    /// 子ノードのインデックスを計算
    fn child_index(&self, point: &Point3D<T>) -> usize {
        let center = self.bounds.center();
        let x_bit = if point.x() >= center.x() { 1 } else { 0 };
        let y_bit = if point.y() >= center.y() { 1 } else { 0 };
        let z_bit = if point.z() >= center.z() { 1 } else { 0 };
        
        // ビット演算でインデックス計算
        (z_bit << 2) | (y_bit << 1) | x_bit
    }
    
    /// 8つの子ノードを生成
    fn subdivide(&mut self) {
        let center = self.bounds.center();
        let min = self.bounds.min();
        let max = self.bounds.max();
        
        let children = [
            // 0: (x_min, y_min, z_min) ~ (x_mid, y_mid, z_mid)
            BBox3D::new(min, center),
            // 1: (x_min, y_max, z_min) ~ ...
            BBox3D::new(
                Point3D::new(min.x(), center.y(), min.z()),
                Point3D::new(center.x(), max.y(), center.z())
            ),
            // ... 残り6つ
        ];
        
        self.children = Some(Box::new(children.map(|bbox| {
            OctreeNode::new(bbox, self.depth + 1)
        })));
    }
}
```

---

### 3.2 基本操作

#### 3.2.1 挿入（Insert）

```rust
impl<T: Scalar, D> Octree<T, D>
where
    D: HasBoundingBox<T>,
{
    /// データを挿入
    pub fn insert(&mut self, item: D) {
        self.root.insert(item, self.max_depth, self.max_items);
        self.total_items += 1;
    }
}

impl<T: Scalar, D> OctreeNode<T, D>
where
    D: HasBoundingBox<T>,
{
    fn insert(&mut self, item: D, max_depth: usize, max_items: usize) {
        // 1. このノードの範囲外なら挿入失敗
        if !self.bounds.intersects(&item.bounding_box()) {
            return;
        }
        
        // 2. 分割済みなら子ノードに委譲
        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                child.insert(item.clone(), max_depth, max_items);
            }
            return;
        }
        
        // 3. 未分割 - このノードにデータ追加
        self.data.push(item);
        
        // 4. 分割条件チェック
        if self.data.len() > max_items && self.depth < max_depth {
            self.subdivide();
            // 既存データを子ノードに再配置
            let old_data = std::mem::take(&mut self.data);
            for item in old_data {
                self.insert(item, max_depth, max_items);
            }
        }
    }
}
```

#### 3.2.2 範囲検索（Query Region）

```rust
impl<T: Scalar, D> Octree<T, D> {
    /// 境界ボックスと交差するすべてのデータを取得
    pub fn query_region(&self, region: &BBox3D<T>) -> Vec<&D> {
        let mut results = Vec::new();
        self.root.query_region(region, &mut results);
        results
    }
}

impl<T: Scalar, D> OctreeNode<T, D> {
    fn query_region<'a>(&'a self, region: &BBox3D<T>, results: &mut Vec<&'a D>) {
        // 1. このノードと範囲が交差しないなら終了
        if !self.bounds.intersects(region) {
            return;
        }
        
        // 2. このノードのデータを結果に追加
        results.extend(self.data.iter());
        
        // 3. 子ノードがあれば再帰的に検索
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_region(region, results);
            }
        }
    }
}
```

#### 3.2.3 最近傍探索（Nearest Neighbor）

```rust
impl<T: Scalar, D> Octree<T, D>
where
    D: HasPosition<T>,
{
    /// 点pから最も近いデータを探索
    pub fn nearest(&self, p: &Point3D<T>) -> Option<(&D, T)> {
        self.root.nearest(p, None)
    }
}

impl<T: Scalar, D> OctreeNode<T, D>
where
    D: HasPosition<T>,
{
    fn nearest(&self, p: &Point3D<T>, best: Option<(&D, T)>) -> Option<(&D, T)> {
        // 1. このノードの最小距離が現在の最良距離より大きい -> 枝刈り
        let min_dist = self.bounds.distance_to_point(p);
        if let Some((_, best_dist)) = best {
            if min_dist > best_dist {
                return best;
            }
        }
        
        // 2. リーフノードならデータから最近傍を探す
        if self.children.is_none() {
            return self.data.iter()
                .map(|item| (item, item.position().distance_to(p)))
                .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap())
                .or(best);
        }
        
        // 3. 子ノードを距離順にソートして探索
        let mut child_dists: Vec<_> = self.children.as_ref().unwrap()
            .iter()
            .map(|child| (child, child.bounds.distance_to_point(p)))
            .collect();
        child_dists.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap());
        
        let mut current_best = best;
        for (child, _) in child_dists {
            current_best = child.nearest(p, current_best);
        }
        
        current_best
    }
}
```

---

### 3.3 切削シミュレーション特化機能

#### 3.3.1 ボクセルOctree

材料除去シミュレーションには「セル単位の状態管理」が必要：

```rust
/// ボクセルの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelState {
    Solid,     // 材料あり
    Empty,     // 材料除去済み
    Mixed,     // 部分的に除去（要細分化）
}

/// ボクセルOctree（切削シミュレーション用）
pub struct VoxelOctree<T: Scalar> {
    root: VoxelNode<T>,
    max_depth: usize,
    voxel_size_at_max_depth: T, // 最小ボクセルサイズ
}

pub struct VoxelNode<T: Scalar> {
    bounds: BBox3D<T>,
    depth: usize,
    state: VoxelState,
    children: Option<Box<[VoxelNode<T>; 8]>>,
}

impl<T: Scalar> VoxelOctree<T> {
    /// 工具による材料除去
    pub fn remove_material(&mut self, tool_shape: &impl Shape3D<T>) {
        self.root.remove_material(tool_shape, self.max_depth);
    }
    
    /// 残存材料の体積計算
    pub fn remaining_volume(&self) -> T {
        self.root.volume()
    }
    
    /// 削り残し検出
    pub fn detect_undercut(&self, target_shape: &impl Shape3D<T>) -> Vec<BBox3D<T>> {
        // target_shape と比較して削り残しを検出
        // ...
    }
}

impl<T: Scalar> VoxelNode<T> {
    fn remove_material(&mut self, tool: &impl Shape3D<T>, max_depth: usize) {
        match self.state {
            VoxelState::Empty => return, // 既に空なら何もしない
            VoxelState::Solid => {
                // 工具と交差判定
                if tool.intersects_bbox(&self.bounds) {
                    if self.depth < max_depth {
                        // 細分化して再帰
                        self.subdivide();
                        for child in self.children.as_mut().unwrap().iter_mut() {
                            child.remove_material(tool, max_depth);
                        }
                    } else {
                        // 最大深さ到達 - セル単位で削除
                        self.state = VoxelState::Empty;
                    }
                }
            }
            VoxelState::Mixed => {
                // 子ノードに委譲
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material(tool, max_depth);
                    }
                }
            }
        }
    }
    
    fn volume(&self) -> T {
        match self.state {
            VoxelState::Empty => T::ZERO,
            VoxelState::Solid => self.bounds.volume(),
            VoxelState::Mixed => {
                self.children.as_ref().unwrap()
                    .iter()
                    .map(|c| c.volume())
                    .sum()
            }
        }
    }
}
```

---

## 実装計画

### Phase 1: 基本Octree実装（1週間）

#### ステップ1: データ構造実装（2日）

**実装ファイル**:
- `model/geo_algorithms/Cargo.toml` - 依存関係追加
- `model/geo_algorithms/src/octree/mod.rs` - モジュール定義
- `model/geo_algorithms/src/octree/node.rs` - OctreeNode 実装
- `model/geo_algorithms/src/octree/octree.rs` - Octree 実装

**実装項目**:
- [x] `OctreeNode` 構造体
- [x] `Octree` 構造体
- [x] `child_index()` - 子ノードインデックス計算
- [x] `subdivide()` - ノード分割

**テスト**:
- ノード分割の境界計算
- 子インデックスの正確性

#### ステップ2: 基本操作実装（3日）

**実装項目**:
- [x] `insert()` - データ挿入
- [x] `query_region()` - 範囲検索
- [x] `query_point()` - 点検索
- [x] `nearest()` - 最近傍探索

**テスト**:
- ランダムデータでの挿入・検索
- 境界条件テスト（空Octree、単一要素）
- パフォーマンステスト（1000要素）

#### ステップ3: 統合・ドキュメント（2日）

**ドキュメント**:
- API ドキュメント（Rustdoc）
- 使用例（examples/octree_basic.rs）
- パフォーマンス特性の文書化

**統合テスト**:
- 既存の衝突判定との統合テスト
- メモリ使用量の測定

---

### Phase 2: ボクセルOctree実装（1週間）

#### ステップ1: VoxelOctree 構造（2日）

**実装ファイル**:
- `model/geo_algorithms/src/octree/voxel.rs`

**実装項目**:
- [x] `VoxelState` 列挙型
- [x] `VoxelNode` 構造体
- [x] `VoxelOctree` 構造体

#### ステップ2: 材料除去シミュレーション（3日）

**実装項目**:
- [x] `remove_material()` - 工具形状による除去
- [x] `remaining_volume()` - 残存体積計算
- [x] `detect_undercut()` - 削り残し検出

**テスト**:
- 単純形状（円筒工具）での材料除去
- 複雑パスでのシミュレーション
- 削り残し検出の正確性

#### ステップ3: 最適化・ドキュメント（2日）

**最適化**:
- メモリプール使用（ノード再利用）
- 並列化（Rayon）

**ドキュメント**:
- 切削シミュレーションの使用例
- パフォーマンスガイドライン

---

## 可視化設計

### 5.1 可視化要件

**目的**: Octreeの内部構造をデバッグ目的で可視化

**表示項目**:
1. **境界ボックス**: 各ノードの範囲を線で表示
2. **深さレベル**: 色分けで深さを表現
3. **データ分布**: データ数に応じた透明度
4. **選択的表示**: 特定深さのみ表示

**UI操作**:
- スライダーで表示深さ変更（0-max_depth）
- トグルで全体/選択レベルの切り替え
- アニメーション再生（構築過程）

---

### 5.2 実装設計

#### ViewModel層: Octree → 頂点変換

```rust
// viewmodel/converter/src/octree_converter.rs

pub struct OctreeVisualizationOptions {
    pub show_depth_range: Range<usize>, // 表示する深さ範囲
    pub color_by_depth: bool,           // 深さで色分け
    pub show_data_count: bool,          // データ数表示
}

pub fn octree_to_wireframe<T: Scalar>(
    octree: &Octree<T, impl Any>,
    options: &OctreeVisualizationOptions,
) -> Vec<Vertex3D> {
    let mut vertices = Vec::new();
    
    octree.traverse(|node, depth| {
        // 表示深さフィルタ
        if !options.show_depth_range.contains(&depth) {
            return;
        }
        
        // 境界ボックスの12辺を追加
        let bbox = node.bounds();
        let color = depth_to_color(depth); // 深さに応じた色
        
        vertices.extend(bbox_to_line_vertices(bbox, color));
    });
    
    vertices
}

fn bbox_to_line_vertices<T: Scalar>(bbox: &BBox3D<T>, color: [f32; 3]) -> Vec<Vertex3D> {
    let min = bbox.min();
    let max = bbox.max();
    
    // 8頂点
    let v000 = [min.x(), min.y(), min.z()];
    let v001 = [min.x(), min.y(), max.z()];
    // ... 残り6頂点
    
    // 12辺をLineListで表現（24頂点）
    vec![
        Vertex3D { position: v000, color },
        Vertex3D { position: v100, color },
        // ... 残り22頂点
    ]
}

fn depth_to_color(depth: usize) -> [f32; 3] {
    // 深さに応じた色グラデーション（青→緑→黄→赤）
    match depth {
        0 => [0.2, 0.5, 1.0], // 青
        1 => [0.2, 0.8, 0.8], // シアン
        2 => [0.2, 1.0, 0.2], // 緑
        3 => [1.0, 1.0, 0.2], // 黄
        _ => [1.0, 0.2, 0.2], // 赤
    }
}
```

#### View層: レンダリング

```rust
// view/stage/src/octree_stage.rs

pub struct OctreeStage {
    resources: LineResources,          // 線描画リソース
    current_depth_range: Range<usize>, // 現在の表示深さ
    animation_time: f32,               // アニメーション時刻
}

impl RenderStage for OctreeStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // LineResources を使ってワイヤーフレーム描画
        self.resources.render(encoder, view);
    }
    
    fn update(&mut self) {
        // アニメーション更新
        if self.animation_playing {
            self.animation_time += 0.016; // 60FPS
            self.update_visible_depth();
        }
    }
}

impl OctreeStage {
    /// 表示深さ範囲を変更
    pub fn set_depth_range(&mut self, range: Range<usize>) {
        self.current_depth_range = range;
        self.rebuild_vertices();
    }
    
    /// アニメーション再生（深さ0から徐々に表示）
    pub fn play_animation(&mut self) {
        self.animation_playing = true;
        self.animation_time = 0.0;
    }
    
    fn update_visible_depth(&mut self) {
        // 時刻に応じて表示深さを変更
        let max_depth = (self.animation_time / 0.5) as usize; // 0.5秒ごとに深さ+1
        self.set_depth_range(0..max_depth.min(self.octree_max_depth));
    }
}
```

#### App層: UI統合

```rust
// view/app/src/app_state.rs

impl AppState {
    fn handle_octree_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Key0 => {
                // Octree可視化トグル
                self.octree_visible = !self.octree_visible;
            }
            KeyCode::BracketLeft => {
                // 表示深さを浅く
                self.octree_depth_range.end = self.octree_depth_range.end.saturating_sub(1);
            }
            KeyCode::BracketRight => {
                // 表示深さを深く
                self.octree_depth_range.end += 1;
            }
            KeyCode::KeyP => {
                // アニメーション再生
                self.octree_stage.play_animation();
            }
            _ => {}
        }
    }
}
```

---

## 応用シナリオ

### 6.1 切削シミュレーション

```rust
// 使用例
let workpiece = BBox3D::new(
    Point3D::new(0.0, 0.0, 0.0),
    Point3D::new(100.0, 100.0, 50.0)
);

let mut voxel_octree = VoxelOctree::new(workpiece, 8); // 深さ8 (256分割)

// 工具経路に沿って材料除去
for segment in toolpath.segments() {
    let tool = CylindricalSolid3D::new(segment.start, segment.end, tool_diameter / 2.0);
    voxel_octree.remove_material(&tool);
}

// 残存体積の確認
let remaining = voxel_octree.remaining_volume();
println!("Removed: {:.2}%", (1.0 - remaining / workpiece.volume()) * 100.0);

// 削り残し検出
let undercuts = voxel_octree.detect_undercut(&target_shape);
if !undercuts.is_empty() {
    eprintln!("Warning: {} undercut regions detected", undercuts.len());
}
```

### 6.2 衝突判定高速化

```rust
// 使用例
let mut octree = Octree::new(scene_bbox);

// すべての形状を登録
for shape in shapes {
    octree.insert(shape);
}

// 高速な衝突判定
for tool_segment in toolpath.segments() {
    let candidates = octree.query_region(&tool_segment.bounding_box());
    
    for shape in candidates {
        if tool_segment.intersects(shape) {
            eprintln!("Collision detected at {:?}", tool_segment.start());
        }
    }
}
```

### 6.3 近傍点検索（サーフェス検証）

```rust
// 使用例: メッシュの品質検証
let mesh_octree = Octree::from_vertices(&mesh.vertices);

let max_distance = 0.01; // 許容誤差
for sample_point in sample_points {
    if let Some((nearest_vertex, dist)) = mesh_octree.nearest(&sample_point) {
        if dist > max_distance {
            eprintln!("Gap detected: {:.4} mm at {:?}", dist, sample_point);
        }
    }
}
```

---

## まとめ

### 実装優先順位

1. **Phase 1: 基本Octree** (1週間)
   - Issue #207
   - 衝突判定・近傍検索に即利用可能

2. **Phase 2: Octree可視化** (1週間)
   - Issue #208
   - デバッグ効率向上

3. **Phase 3: ボクセルOctree** (1週間)
   - 切削シミュレーション対応
   - Phase 4以降で本格活用

### 期待効果

- ✅ 衝突判定: 99%の計算量削減（1000形状で）
- ✅ 切削シミュレーション: リアルタイム可視化が可能
- ✅ デバッグ効率: 空間構造の視覚的確認
- ✅ 将来拡張: メッシュ生成、LOD、光線追跡への応用

---

## 関連情報

- **先行技術**: PCL (Point Cloud Library), OpenVDB
- **参考論文**: "Octree-Based Collision Detection" (Meagher, 1982)
- **実装例**: Unity Octree, UE5 Octree
