# 円弧補間カッターパスの切削シミュレーション実装調査

**作成日**: 2026年2月12日  
**最終更新日**: 2026年2月12日

## 調査目的

CNCマシンの実際のカッターパス（G02/G03円弧補間）を正確にシミュレーションする方法を調査し、以下を明らかにする：

1. 線分近似による実装の妥当性
2. 円弧を正確にシミュレーションするアルゴリズムの実装可能性
3. 両アプローチのトレードオフ比較

## 現状分析

### 実装済み機能

#### VoxelOctree材料除去API（`model/geo_algorithms/src/octree/voxel.rs`）

```rust
// Box形状による除去
pub fn remove_material_box(&mut self, tool_aabb: &Aabb3D<T>);

// 線分＋半径（カプセル）による除去
pub fn remove_material_capsule(&mut self, segment: &LineSegment3D<T>, radius: T);

// Z軸方向円柱除去（高速版）
pub fn remove_material_z_axis(&mut self, center_x: T, center_y: T, z_start: T, z_end: T, radius: T);
```

#### 幾何プリミティブ

RedRingには既に充実した円弧実装が存在：

- **Arc2D** (`model/geo_primitives/src/arc_2d.rs`)
  - 2D円弧（平面上）
  - 角度範囲指定（start_angle → end_angle）
  
- **Arc3D** (`model/geo_primitives/src/arc_3d.rs`)
  - 3D空間での円弧
  - 中心・半径・法線・開始/終了角度で定義
  - 3点から円弧を生成: `from_three_points()`
  - XY/XZ/YZ平面円弧の便利メソッド
  
- **サンプリング機能** (`arc_3d_extensions.rs`)
  ```rust
  pub fn sample_points(&self, num_points: usize) -> Vec<Point3D<T>>
  ```

#### 距離計算（`model/geo_commons/src/metrics/distance.rs`）

現在実装されている距離計算関数：
- 楕円-点間距離
- 球-無限直線間距離
- 線分-AABB間距離（カプセル除去で使用中）

### 円弧補間対応の実装アプローチ

## アプローチ1: 線分近似（Polyline Approximation）

### 概要

円弧を短い線分の連続（折れ線）に近似し、既存の `remove_material_capsule()` を繰り返し呼び出す。

### 実装方法

```rust
/// 円弧経路による材料除去（線分近似版）
///
/// # Arguments
///
/// * `arc` - 工具経路の円弧（Arc3D）
/// * `radius` - 工具半径
/// * `num_segments` - 近似に使用する線分の数
pub fn remove_material_arc_polyline(
    &mut self,
    arc: &Arc3D<T>,
    radius: T,
    num_segments: usize
) {
    // 円弧を等間隔でサンプリング
    let points = arc.sample_points(num_segments + 1);
    
    // 隣接点間を線分で除去
    for i in 0..num_segments {
        let segment = LineSegment3D::new(points[i], points[i + 1])?;
        self.remove_material_capsule(&segment, radius);
    }
}
```

### メリット

- ✅ **実装が容易**: 既存の `remove_material_capsule()` を再利用
- ✅ **任意の曲線に対応可能**: NURBS、スプラインなど他の曲線も同様に近似可能
- ✅ **即座に利用可能**: 新規アルゴリズム不要

### デメリット

- ❌ **精度と計算量のトレードオフ**: 精度向上には線分数を増やす必要があり、計算コストが増大
- ❌ **円弧の滑らかさが失われる**: 折れ線による階段状の近似
- ❌ **過剰除去の可能性**: 線分の外側が円弧より大きくなる領域が発生

### 精度評価

線分数と誤差の関係：

| 線分数 | 最大誤差（半径10mm, 90度円弧） | 計算量 |
|--------|-------------------------------|--------|
| 4      | ~0.76mm (7.6%)                | 4回    |
| 8      | ~0.19mm (1.9%)                | 8回    |
| 16     | ~0.05mm (0.5%)                | 16回   |
| 32     | ~0.01mm (0.1%)                | 32回   |

誤差計算式（円弧の弦と円弧自体の差）:
```
error = r * (1 - cos(θ/2))
where θ = arc_span / num_segments
```

### 推奨パラメータ

- **粗加工**: 4-8線分（速度重視）
- **仕上げ加工**: 16-32線分（精度重視）
- **高精度**: 64線分以上（特殊用途）

## アプローチ2: 正確な円弧シミュレーション（Exact Arc Handling）

### 概要

円弧の幾何学的性質を利用し、ボクセルと円弧の正確な距離計算により材料除去を行う。

### 必要な実装

#### 2.1 円弧-AABB間距離計算

```rust
/// 円弧の中心軸からAABBまでの最短距離を計算
///
/// # 計算ステップ
///
/// 1. AABBの8頂点それぞれから円弧までの距離を計算
/// 2. AABBのエッジ（12本）と円弧の交差を検出
/// 3. 最小距離を返す
fn arc_to_aabb_distance<T: Scalar>(
    arc: &Arc3D<T>,
    aabb: &Aabb3D<T>
) -> T {
    // 実装候補:
    // 1. 円弧を含む平面とAABBの位置関係を判定
    // 2. AABB頂点から円弧平面への投影
    // 3. 投影点が円弧の角度範囲内かチェック
    // 4. 範囲内なら投影点までの距離、範囲外なら端点までの距離
    todo!()
}
```

#### 2.2 点-円弧間距離

```rust
/// 点から円弧までの最短距離
///
/// # アルゴリズム
///
/// 1. 点を円弧の平面に投影
/// 2. 投影点と円弧中心を結ぶ方向を計算
/// 3. その方向が円弧の角度範囲内にあるか判定
///    - 範囲内: 点から円弧円周上の最近点までの距離
///    - 範囲外: 点から円弧の端点（start/end）までの距離の最小値
fn point_to_arc_distance<T: Scalar>(
    point: Point3D<T>,
    arc: &Arc3D<T>
) -> T {
    // 1. 点を円弧平面に投影
    let plane_normal = arc.normal();
    let to_point = point.vector_from(&arc.center());
    let plane_distance = to_point.dot(plane_normal.as_vector());
    let projected_point = point - plane_normal.as_vector() * plane_distance;
    
    // 2. 投影点から中心への方向
    let to_projected = projected_point.vector_from(&arc.center());
    let direction_angle = to_projected.angle_in_plane(arc.start_direction());
    
    // 3. 角度範囲チェック
    if arc.angle_range().contains(direction_angle) {
        // 範囲内: 円弧上の最近点までの距離
        let on_arc = arc.center() + to_projected.normalize() * arc.radius();
        point.distance_to(&on_arc)
    } else {
        // 範囲外: 端点までの距離の最小値
        let dist_start = point.distance_to(&arc.start_point());
        let dist_end = point.distance_to(&arc.end_point());
        dist_start.min(dist_end)
    }
}
```

#### 2.3 VoxelOctreeへの統合

```rust
impl<T: Scalar> VoxelNode<T> {
    /// 円弧経路による材料除去（正確版）
    fn remove_material_arc(
        &mut self,
        arc: &Arc3D<T>,
        radius: T,
        max_depth: usize
    ) {
        match self.state {
            VoxelState::Empty => (),
            
            VoxelState::Solid => {
                // 円弧からAABBへの距離計算
                let distance = arc_to_aabb_distance(arc, &self.bounds);
                
                // 工具範囲外なら枝刈り
                if distance > radius {
                    return;
                }
                
                // 完全包含判定（全8頂点が範囲内）
                if self.is_aabb_inside_arc_sweep(arc, radius) {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }
                
                // 部分的交差 → 細分化
                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_arc(arc, radius, max_depth);
                    }
                } else {
                    self.state = VoxelState::Empty;
                }
            }
            
            VoxelState::Mixed => {
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_arc(arc, radius, max_depth);
                    }
                    // 子の状態による親の状態更新
                    self.update_state_from_children();
                }
            }
        }
    }
    
    /// AABBが円弧の工具経路内に完全に含まれるか判定
    fn is_aabb_inside_arc_sweep(
        &self,
        arc: &Arc3D<T>,
        radius: T
    ) -> bool {
        let vertices = self.bounds.vertices(); // 8頂点
        
        for vertex in vertices {
            let distance = point_to_arc_distance(vertex, arc);
            if distance > radius {
                return false; // 1つでも外側なら完全包含ではない
            }
        }
        
        true
    }
}
```

### メリット

- ✅ **数学的に正確**: 円弧の幾何学的定義に基づく厳密な計算
- ✅ **滑らかな結果**: 線分近似のような階段状のアーティファクトなし
- ✅ **一定の精度保証**: ボクセル解像度のみに依存、線分数に依存しない

### デメリット

- ❌ **実装の複雑さ**: 円弧-AABB距離計算は複雑（約200-300行のコード）
- ❌ **計算コスト**: 距離計算が線分版より複雑（ただし呼び出し回数は少ない）
- ❌ **デバッグ困難**: 幾何計算のバグは発見しにくい
- ❌ **拡張性**: NURBS曲線など他の曲線には別アルゴリズムが必要

### パフォーマンス予測

| 指標              | 線分近似（16線分） | 正確な円弧       |
|-------------------|-------------------|-----------------|
| 距離計算の複雑度   | O(1) × 16回       | O(1) × 1回      |
| 枝刈り効率        | 中程度            | 高い（正確な距離）|
| 総計算時間        | 基準              | 0.5-0.8倍       |

## アプローチ3: ハイブリッド方式（Adaptive）

### 概要

円弧の特性に応じて、線分近似と正確な計算を使い分ける。

### 判定基準

```rust
pub fn remove_material_arc_adaptive(
    &mut self,
    arc: &Arc3D<T>,
    radius: T
) {
    // 判定基準
    let arc_length = arc.arc_length();
    let curvature = arc.curvature(); // 1 / radius
    
    if arc_length < self.voxel_size_at_max_depth() * T::from_f64(2.0) {
        // 円弧が非常に短い → 単一カプセルで近似
        let segment = LineSegment3D::new(arc.start_point(), arc.end_point())?;
        self.remove_material_capsule(&segment, radius);
    } else if curvature * radius < T::from_f64(0.01) {
        // 曲率が小さい（ほぼ直線） → 線分近似（少数）
        self.remove_material_arc_polyline(arc, radius, 4);
    } else {
        // 複雑な円弧 → 正確な計算
        self.remove_material_arc_exact(arc, radius);
    }
}
```

### メリット

- ✅ パフォーマンスと精度のバランス
- ✅ 簡単なケースは高速処理
- ✅ 複雑なケースのみ正確な計算

## 実装の優先順位と推奨事項

### フェーズ1: 線分近似版（即座に実装可能）

**対象**: Issue #206の拡張として即座に実装

```rust
// model/geo_algorithms/src/octree/voxel.rs に追加

impl<T: Scalar> VoxelOctree<T> {
    /// 円弧経路による材料除去（線分近似版）
    ///
    /// # Arguments
    ///
    /// * `arc` - 工具経路の円弧
    /// * `radius` - 工具半径
    /// * `num_segments` - 近似に使用する線分数（推奨: 8-32）
    ///
    /// # Performance
    ///
    /// 線分数に比例して計算時間が増加。
    /// 推奨値：
    /// - 粗加工: 8線分
    /// - 仕上げ: 16-32線分
    ///
    /// # Example
    ///
    /// \`\`\`rust,ignore
    /// use geo_primitives::Arc3D;
    /// 
    /// let arc = Arc3D::xy_arc(
    ///     Point3D::new(50.0, 50.0, 0.0),
    ///     20.0,
    ///     Angle::degrees(0.0),
    ///     Angle::degrees(90.0)
    /// ).unwrap();
    /// 
    /// voxel_tree.remove_material_arc_polyline(&arc, 5.0, 16);
    /// \`\`\`
    pub fn remove_material_arc_polyline(
        &mut self,
        arc: &Arc3D<T>,
        radius: T,
        num_segments: usize
    ) {
        let points = arc.sample_points(num_segments + 1);
        
        for i in 0..num_segments {
            if let Some(segment) = LineSegment3D::new(points[i], points[i + 1]) {
                self.remove_material_capsule(&segment, radius);
            }
        }
    }
}
```

**理由**:
1. 既存コード再利用で実装工数は1-2時間
2. 実用十分な精度（16線分で0.5%誤差）
3. NURBS曲線など他の曲線にも応用可能

### フェーズ2: 正確な円弧計算版（研究・最適化フェーズ）

**対象**: パフォーマンス最適化が必要になった際に検討

**前提条件**:
1. 円弧-AABB距離計算の実装完了（geo_commons への追加）
2. パフォーマンステストでボトルネック確認
3. ユーザーからの精度要求

**実装工数**: 約40-60時間（設計・実装・テスト含む）

### フェーズ3: ハイブリッド版（将来の拡張）

**対象**: CAMエンジンとしての成熟後

## 結論と推奨事項

### 推奨実装: **アプローチ1（線分近似）**

**根拠**:

1. **実用十分な精度**: 
   - 16線分近似で誤差0.5%（0.05mm / 10mm工具）
   - CNC加工の一般的な公差（±0.05mm）内に収まる

2. **即座に利用可能**:
   - 実装工数: 1-2時間（テスト含む）
   - 既存の `Arc3D::sample_points()` と `remove_material_capsule()` を組み合わせるのみ

3. **拡張性**:
   - NURBS曲線、Bスプラインなど他の曲線型も同様に対応可能
   - `sample_points()` メソッドを持つ任意の曲線に適用可能

4. **パフォーマンス**:
   - 16線分 × 既存カプセル除去 = 十分高速
   - Octreeの枝刈りにより無駄な計算は削減される

### 線分近似版の実装品質基準

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_arc_polyline_removal_accuracy() {
        // 90度円弧、半径20mm、工具半径5mm
        let arc = Arc3D::xy_arc(
            Point3D::new(50.0, 50.0, 0.0),
            20.0,
            Angle::degrees(0.0),
            Angle::degrees(90.0)
        ).unwrap();
        
        let mut tree_8seg = VoxelOctree::new(bounds, 6);
        tree_8seg.remove_material_arc_polyline(&arc, 5.0, 8);
        
        let mut tree_16seg = VoxelOctree::new(bounds, 6);
        tree_16seg.remove_material_arc_polyline(&arc, 5.0, 16);
        
        // 16線分版は8線分版より多く除去されるはず
        assert!(tree_16seg.remaining_volume() < tree_8seg.remaining_volume());
        
        // 収束性確認：線分数を増やすと体積差が減少
        let diff_8_16 = tree_8seg.remaining_volume() - tree_16seg.remaining_volume();
        
        let mut tree_32seg = VoxelOctree::new(bounds, 6);
        tree_32seg.remove_material_arc_polyline(&arc, 5.0, 32);
        let diff_16_32 = tree_16seg.remaining_volume() - tree_32seg.remaining_volume();
        
        assert!(diff_16_32 < diff_8_16); // 収束している
    }
    
    #[test]
    fn test_arc_vs_segment_removal() {
        // 直線の円弧 = 線分と同じ結果になるべき
        let straight_arc = Arc3D::xy_arc(
            Point3D::new(50.0, 50.0, 25.0),
            1000.0, // 大きな半径 = ほぼ直線
            Angle::degrees(0.0),
            Angle::degrees(0.01) // 微小角度
        ).unwrap();
        
        let segment = LineSegment3D::new(
            straight_arc.start_point(),
            straight_arc.end_point()
        ).unwrap();
        
        let mut tree_arc = VoxelOctree::new(bounds, 5);
        tree_arc.remove_material_arc_polyline(&straight_arc, 5.0, 16);
        
        let mut tree_seg = VoxelOctree::new(bounds, 5);
        tree_seg.remove_material_capsule(&segment, 5.0);
        
        // 結果がほぼ同じであることを確認（ボクセル誤差許容）
        let diff = (tree_arc.remaining_volume() - tree_seg.remaining_volume()).abs();
        let total = bounds.volume();
        assert!(diff / total < 0.01); // 1%以内の誤差
    }
}
```

## 実装スケジュール案

### Phase 1: 基本実装（1-2時間）

- [ ] `remove_material_arc_polyline()` 実装
- [ ] 基本テスト（3-5ケース）
- [ ] 使用例の追加（examples/voxel_simulation.rs）

### Phase 2: ドキュメント整備（30分）

- [ ] Rustdoc コメント追加
- [ ] パラメータ選択ガイド記述
- [ ] 精度とパフォーマンスのトレードオフ説明

### Phase 3: 検証（1-2時間）

- [ ] 精度テスト（線分数vs誤差）
- [ ] パフォーマンステスト
- [ ] エッジケーステスト（小円弧、大円弧、完全円）

## 将来の拡張可能性

### NURBS曲線対応

同様のアプローチで対応可能：

```rust
pub fn remove_material_nurbs_curve(
    &mut self,
    curve: &NurbsCurve3D<T>,
    radius: T,
    num_segments: usize
) {
    let points = curve.sample_uniform(num_segments + 1);
    
    for i in 0..num_segments {
        if let Some(segment) = LineSegment3D::new(points[i], points[i + 1]) {
            self.remove_material_capsule(&segment, radius);
        }
    }
}
```

### 工具経路全体のシミュレーション

G-codeパーサーと組み合わせ：

```rust
pub enum ToolPath<T: Scalar> {
    Rapid(Point3D<T>),          // G00
    Line(LineSegment3D<T>),     // G01
    ArcCW(Arc3D<T>),            // G02
    ArcCCW(Arc3D<T>),           // G03
}

pub fn simulate_toolpath(
    &mut self,
    paths: &[ToolPath<T>],
    tool_radius: T
) {
    for path in paths {
        match path {
            ToolPath::Line(seg) => {
                self.remove_material_capsule(seg, tool_radius);
            }
            ToolPath::ArcCW(arc) | ToolPath::ArcCCW(arc) => {
                self.remove_material_arc_polyline(arc, tool_radius, 16);
            }
            ToolPath::Rapid(_) => {
                // 早送り移動は材料除去しない
            }
        }
    }
}
```

## 参考文献・関連資料

### 内部設計文書

- `dev/architecture/OCTREE_DESIGN.md` - Octree設計詳細
- `model/geo_primitives/src/arc_3d.rs` - Arc3D実装
- `model/geo_primitives/src/arc_3d_extensions.rs` - サンプリング機能

### 外部参考資料

1. **G-code 円弧補間**:
   - G02: 時計回り円弧
   - G03: 反時計回り円弧
   - パラメータ: I, J, K（中心オフセット）または R（半径）

2. **CNC加工精度**:
   - 一般的な公差: ±0.05mm - ±0.1mm
   - 高精度加工: ±0.01mm - ±0.02mm

3. **ボクセル解像度**:
   - max_depth=6 → ボクセルサイズ = 辺長 / 64
   - 100mm ワーク → 最小1.56mm/voxel
   - max_depth=8 → 最小0.39mm/voxel

## 更新履歴

- 2026-02-12: 初回作成（円弧補間対応の調査・設計）
