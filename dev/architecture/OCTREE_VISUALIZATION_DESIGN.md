# Octree可視化システム設計書

**作成日**: 2026年2月12日  
**対象Issue**: [#207](https://github.com/RedRing2020/RedRing/issues/207)  
**依存**: Issue #206（Octree実装）完了  
**状態**: � 実装中

## 2026年2月21日 追記: 初期表示カメラの自動フィット

- 背景のみ表示されるケースへの対策として、`load_debug_octree()` の初期カメラを固定値からAABBベースへ変更。
- Octreeワイヤーフレーム頂点のAABBを計算し、中心を `camera.target` に設定。
- 直交投影範囲はAABBのXY最大寸法に40%マージンを加えた正方範囲を採用。
- Z方向寸法に応じて `camera.distance` を自動決定し、極小オブジェクトでも最小表示距離を確保。
- これにより、サンプルデータのスケールや配置に依存せず、初回表示時に対象が画角に入ることを保証する。

## 2026年2月21日 追記: 行列合成順序の統一

- Octree（LineResources）とMesh描画でビュー投影行列の合成順序を `proj * view` に統一。
- シェーダー側は `clip = view_proj * world` で評価しているため、CPU側での `view * proj` は不整合となる。
- `ToolPathStage` 既存実装に合わせて統一し、背景のみ表示される事象の根本原因を解消。

## 2026年2月21日 追記: 直交投影境界の座標系修正

- Octree初期表示で `camera.target` と `orthographic_bounds` の中心補正が二重適用され、表示が左下へ寄る問題を修正。
- `orthographic_bounds` はワールド座標中心ではなく、ビュー変換後のカメラ空間基準（`-half_extent..+half_extent`）で設定。
- これにより、ワーク座標が正方向のみでも初期表示が画面中心に一致する。

## 2026年2月21日 追記: 複数ボクセル可視化テストデータ

- 既存のデバッグ生成は初期状態の `VoxelOctree` をそのまま可視化しており、実質1ボクセル表示となっていた。
- Octreeらしい分割状態を視認できるよう、デバッグ生成で `remove_material_box` を適用し、部分除去による複数Solidボクセルを生成する。
- 可視化対象は従来通り `VoxelState::Solid` を維持し、複数セルのワイヤーフレームで分割構造を確認できるようにする。

## 2026年2月21日 追記: 完全一致辺の重複表示削除

- 隣接ボクセルで共有される同一辺が重複描画され、視認性が低下する問題に対処。
- 対象は「完全一致する線分（端点2点が一致）」のみ。
- 端点順序の違い（A→B と B→A）は同一辺として扱う。
- 実装は ViewModel 変換後（LineList頂点列）で行い、GPU負荷と描画ノイズを低減する。
- 部分重なり線分の統合は本対応スコープ外とし、将来課題とする。

## 2026年2月21日 追記: 本Issueでの完了条件調整（合意事項）

- 深さ変更（depth切替）に関する完了条件は、本Issueではオミットする。
- 複数ボクセルのテストデータ表示により、可視化改善の確認を完了とする。
- Octreeの分割・除去に伴うアニメーションは、切削シミュレーション実装時に別途対応する。

## 2026年2月21日 追記: カメラビューイング処理の共通化

- 各ステージ/リソースで重複していた `proj * view` 合成処理を `viewmodel_graphics::camera::build_view_projection_matrix` に集約。
- ステージ/レンダラーは共通関数を呼ぶのみとし、カメラ行列合成ロジックの単一点管理を実現。

## 📋 概要

Octreeの内部構造を可視化し、デバッグ・検証・教育目的で活用する。

**スコープ**: このIssueは**Octree可視化のみ**に焦点を当てます。カッターパス表示・工具表示は別Issueで実装します。

### 段階的実装戦略

**Phase 1: ワイヤーフレーム表示（デバッグ用）** - 1週間
- Octree構造の確認・デバッグ
- 空間分割の可視化
- 軽量・高速表示

**Phase 2: シェーディング表示（検証用）** - 1週間  
- VoxelOctreeの切削結果表示
- 状態（Solid/Empty/Mixed）による色分け
- リアルタイム陰影表示
- 視覚的な検証支援

### 目的

1. **デバッグ効率向上**
   - Octreeの分割状況を視覚的に確認
   - データ分布の偏り検出
   - アルゴリズムの挙動確認

2. **切削シミュレーション検証**
   - VoxelOctreeの状態確認
   - 削り残し領域の特定
   - 分割深さの適切性確認

3. **教育・デモ用**
   - Octree構造の理解促進
   - 構築過程のアニメーション

## 🏗️ アーキテクチャ設計

### レイヤー構成

```text
App Layer (view/app)
  ↓ キー入力
ViewModel Layer (viewmodel/converter)
  ↓ Octree → Vertex3D[]
View Layer (view/stage)
  ↓ GPU描画
Render Layer (view/render)
```

### データフロー

```text
Octree<T, D>
  ↓ octree_to_wireframe()
Vec<Vertex3D>
  ↓ LineResources::update_vertices()
GPU Buffer
  ↓ RenderStage::render()
画面表示
```

## 📐 ViewModel層設計

### ファイル: `viewmodel/converter/src/octree_converter.rs`

#### データ構造

```rust
/// Octree可視化オプション
#[derive(Debug, Clone)]
pub struct OctreeVisualizationOptions {
    /// 表示する深さ範囲（例: 0..3 で深さ0-2を表示）
    pub show_depth_range: Range<usize>,

    /// 深さで色分けするか
    pub color_by_depth: bool,

    /// データ数を透明度で表現するか
    pub show_data_count: bool,

    /// 空ノードも表示するか
    pub show_empty_nodes: bool,

    /// カスタムカラーマップ（None = デフォルト）
    pub custom_colormap: Option<ColorMap>,
}

impl Default for OctreeVisualizationOptions {
    fn default() -> Self {
        Self {
            show_depth_range: 0..8,
            color_by_depth: true,
            show_data_count: false,
            show_empty_nodes: false,
            custom_colormap: None,
        }
    }
}

/// カラーマップ（深さ → RGB）
pub struct ColorMap {
    colors: Vec<[f32; 3]>,
}
```

#### 主要関数

```rust
/// Octree → ワイヤーフレーム頂点変換
///
/// # Arguments
/// 
/// * `octree` - 可視化対象のOctree
/// * `options` - 可視化オプション
///
/// # Returns
///
/// GPU描画用の頂点配列（LineList形式）
pub fn octree_to_wireframe<T: Scalar, D>(
    octree: &Octree<T, D>,
    options: &OctreeVisualizationOptions,
) -> Vec<Vertex3D> {
    let mut vertices = Vec::new();

    octree.traverse(|node, depth| {
        // 深さフィルタ
        if !options.show_depth_range.contains(&depth) {
            return;
        }

        // 空ノードフィルタ
        if !options.show_empty_nodes && node.data().is_empty() {
            return;
        }

        // 色決定
        let color = if options.color_by_depth {
            depth_to_color(depth, options.custom_colormap.as_ref())
        } else {
            [1.0, 1.0, 1.0] // 白
        };

        // 境界ボックス → 12辺の頂点追加
        vertices.extend(bbox_to_line_vertices(node.bounds(), color));
    });

    vertices
}

/// VoxelOctree専用の可視化（状態で色分け）
pub fn voxel_octree_to_wireframe<T: Scalar>(
    voxel_octree: &VoxelOctree<T>,
    options: &OctreeVisualizationOptions,
) -> Vec<Vertex3D> {
    let mut vertices = Vec::new();

    // VoxelNode走査用のヘルパー（仮実装）
    // 実際は VoxelOctree に traverse() を追加する必要がある
    
    vertices
}
```

#### ヘルパー関数

```rust
/// 深さ → 色変換（グラデーション）
fn depth_to_color(depth: usize, colormap: Option<&ColorMap>) -> [f32; 3] {
    if let Some(map) = colormap {
        map.get_color(depth)
    } else {
        // デフォルトカラーマップ（青 → 緑 → 黄 → 赤）
        match depth {
            0 => [0.2, 0.5, 1.0],  // 青
            1 => [0.2, 0.8, 0.8],  // シアン
            2 => [0.2, 1.0, 0.2],  // 緑
            3 => [1.0, 1.0, 0.2],  // 黄
            4 => [1.0, 0.6, 0.2],  // オレンジ
            _ => [1.0, 0.2, 0.2],  // 赤
        }
    }
}

/// 境界ボックス → ワイヤーフレーム頂点（12辺 = 24頂点）
fn bbox_to_line_vertices<T: Scalar>(
    bbox: &Aabb3D<T>,
    color: [f32; 3],
) -> Vec<Vertex3D> {
    let min = bbox.min();
    let max = bbox.max();

    // 8頂点定義
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
        Vertex3D { position: v000, color }, Vertex3D { position: v100, color },
        Vertex3D { position: v100, color }, Vertex3D { position: v110, color },
        Vertex3D { position: v110, color }, Vertex3D { position: v010, color },
        Vertex3D { position: v010, color }, Vertex3D { position: v000, color },
        
        // 上面 (z = max)
        Vertex3D { position: v001, color }, Vertex3D { position: v101, color },
        Vertex3D { position: v101, color }, Vertex3D { position: v111, color },
        Vertex3D { position: v111, color }, Vertex3D { position: v011, color },
        Vertex3D { position: v011, color }, Vertex3D { position: v001, color },
        
        // 垂直辺（4本）
        Vertex3D { position: v000, color }, Vertex3D { position: v001, color },
        Vertex3D { position: v100, color }, Vertex3D { position: v101, color },
        Vertex3D { position: v110, color }, Vertex3D { position: v111, color },
        Vertex3D { position: v010, color }, Vertex3D { position: v011, color },
    ]
}
```

#### サンプルデータ生成関数（Phase 1実装）

```rust
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
    // VoxelOctree作成 → voxel_octree_to_wireframe() 変換
    // → 位置データのみ抽出して返す
}
```

**アーキテクチャ上の役割**:
- View層（app）がModel層（geo_algorithms）に直接依存することを回避
- ViewModel層でサンプルデータ生成を完結させる設計パターン

### テスト項目

- [ ] 深さ0のみ表示（ルートノードのみ）
- [ ] 深さ0-3表示（複数深さ）
- [ ] 色グラデーション確認
- [ ] 空ノードフィルタリング
- [ ] 大規模Octree（1000+ノード）でのパフォーマンス

## 🎨 View層設計

### ファイル: `view/stage/src/octree_stage.rs`

#### データ構造

```rust
/// OctreeStage - Octree可視化ステージ
pub struct OctreeStage {
    /// 線描画リソース（LineResources を流用）
    line_resources: LineResources,

    /// 現在の表示深さ範囲
    current_depth_range: Range<usize>,

    /// Octreeの最大深さ
    octree_max_depth: usize,

    /// アニメーション状態
    animation: AnimationState,

    /// 可視化オプション
    options: OctreeVisualizationOptions,
}

/// アニメーション状態
#[derive(Debug, Clone)]
enum AnimationState {
    Stopped,
    Playing {
        elapsed: f32,      // 経過時間（秒）
        speed: f32,        // 速度倍率
    },
}
```

#### 実装

```rust
impl RenderStage for OctreeStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.line_resources.render(encoder, view);
    }

    fn update(&mut self) {
        if let AnimationState::Playing { ref mut elapsed, speed } = self.animation {
            *elapsed += 0.016 * speed; // 60FPS想定

            // 経過時間に応じて深さ更新（0.5秒ごとに+1）
            let target_depth = (*elapsed / 0.5) as usize;

            if target_depth <= self.octree_max_depth {
                self.set_depth_range(0..(target_depth + 1));
            } else {
                // アニメーション終了
                self.animation = AnimationState::Stopped;
            }
        }
    }
}

impl OctreeStage {
    /// コンストラクタ
    pub fn new<T: Scalar, D>(
        octree: &Octree<T, D>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let options = OctreeVisualizationOptions::default();
        let vertices = octree_to_wireframe(octree, &options);
        let line_resources = LineResources::new(device, queue, vertices);

        Self {
            line_resources,
            current_depth_range: 0..1,
            octree_max_depth: octree.max_depth(),
            animation: AnimationState::Stopped,
            options,
        }
    }

    /// 表示深さ範囲変更
    pub fn set_depth_range(&mut self, range: Range<usize>) {
        self.current_depth_range = range.clone();
        self.options.show_depth_range = range;
        self.rebuild_vertices();
    }

    /// アニメーション再生
    pub fn play_animation(&mut self, speed: f32) {
        self.animation = AnimationState::Playing {
            elapsed: 0.0,
            speed,
        };
    }

    /// アニメーション停止
    pub fn stop_animation(&mut self) {
        self.animation = AnimationState::Stopped;
    }

    /// 頂点データ再構築
    fn rebuild_vertices(&mut self) {
        // Octree参照が必要なので、AppStateから渡す方式に変更予定
        // ここでは仮実装
    }
}
```

### テスト項目

- [ ] OctreeStage作成
- [ ] 描画確認
- [ ] 深さ範囲変更
- [ ] アニメーション再生・停止
- [ ] 60FPS維持（1000ノード）

## 🎮 App層設計

### ファイル: `view/app/src/app_state.rs`

#### 実装内容（Phase 1）

**追加メソッド**: `load_debug_octree()`

```rust
/// デバッグ用：VoxelOctree可視化を表示
pub fn load_debug_octree(&mut self) {
    use viewmodel::octree_converter::create_sample_voxel_octree_wireframe;

    tracing::info!("VoxelOctree可視化デバッグ開始");

    // ViewModelでサンプルデータ生成（ワイヤーフレーム頂点）
    let positions = create_sample_voxel_octree_wireframe();

    tracing::info!("ワイヤーフレーム頂点数: {}", positions.len());

    // OctreeStageを作成してデータ設定
    let mut octree_stage = Box::new(OctreeStage::new(
        &self.graphic.device,
        self.graphic.config.format,
    ));
    octree_stage.set_wireframe_data(&self.graphic.device, positions);

    // カメラを標準CAD視点に設定
    self.camera.reset_to_standard_cad_view();

    self.renderer.set_stage(octree_stage);

    // カメラユニフォーム更新
    self.update_camera_uniforms();

    tracing::info!("VoxelOctree可視化デバッグ完了");
}
```

**アーキテクチャ遵守**:
- View層（app）はViewModel層（viewmodel/converter）の関数のみ呼び出す
- Model層（geo_algorithms, geo_core）への直接依存を回避
- サンプルデータ生成は ViewModel の `create_sample_voxel_octree_wireframe()` で実装

#### キーバインディング（Phase 1）

| キー | 機能 | 説明 |
|------|------|------|
| `o` | VoxelOctree可視化 | サンプルデータ読み込み + ワイヤーフレーム表示 |

**使用方法**:
1. アプリ起動: `cargo run`
2. `o` キーを押す → VoxelOctree 可視化表示
3. マウスドラッグ: カメラ回転
4. マウスホイール: カメラズーム
5. `h` キー: 全キーバインド一覧表示

---

### 将来の拡張（Phase 2 以降）

以下の機能は今後のイテレーションで追加予定：

| キー | 機能（予定） | 説明 |
|------|------|------|
| `0` | Octree表示切替 | 表示/非表示トグル |
| `[` | 深さ減少 | 表示深さを1つ浅く |
| `]` | 深さ増加 | 表示深さを1つ深く |
| `P` | アニメーション再生 | 深さ0から段階的に表示 |
| `S` | アニメーション停止 | アニメーション中断 |
| `-` | 速度減速 | アニメーション速度を遅く |
| `+` | 速度加速 | アニメーション速度を速く |

## 📊 パフォーマンス設計

### 目標

- **1000ノードOctree**: 60FPS維持
- **10000ノードOctree**: 30FPS以上
- **頂点数**: 1ノード = 24頂点（12辺 × 2）

### 最適化戦略

1. **頂点数削減**
   - 深さフィルタリング
   - 空ノード除外

2. **GPU効率化**
   - LineResources の効率的利用
   - バッファ再利用

3. **更新最小化**
   - 深さ変更時のみ再構築
   - アニメーション中はフレーム毎に更新しない

## 🧪 テスト計画

### 単体テスト

```rust
#[test]
fn test_octree_to_wireframe_empty() {
    let bbox = Aabb3D::new(Point3D::origin(), Point3D::new(100.0, 100.0, 100.0));
    let octree = Octree::<f64, ()>::new(bbox, 8, 10);
    let options = OctreeVisualizationOptions::default();
    
    let vertices = octree_to_wireframe(&octree, &options);
    
    // ルートノードのみ（24頂点）
    assert_eq!(vertices.len(), 24);
}

#[test]
fn test_depth_filter() {
    // 深さ0-1のみ表示
    let options = OctreeVisualizationOptions {
        show_depth_range: 0..2,
        ..Default::default()
    };
    
    // 実際のOctreeで検証
}

#[test]
fn test_color_gradient() {
    assert_eq!(depth_to_color(0, None), [0.2, 0.5, 1.0]); // 青
    assert_eq!(depth_to_color(3, None), [1.0, 1.0, 0.2]); // 黄
}
```

### 統合テスト

- [ ] OctreeStage作成 → 描画
- [ ] キー入力 → 深さ変更 → 即座反映
- [ ] アニメーション再生 → スムーズな遷移
- [ ] 大規模Octree（1000ノード）でのFPS計測

## 📁 ファイル構成

```
viewmodel/converter/src/
  ├── octree_converter.rs          (新規)
  └── lib.rs                        (修正: re-export)

view/stage/src/
  ├── octree_stage.rs               (新規)
  └── lib.rs                        (修正: re-export)

view/app/src/
  └── app_state.rs                  (修正: キー入力追加)

model/geo_algorithms/examples/
  └── octree_visualization.rs       (新規: 使用例)
```

## 📝 実装手順

### 🔵 Phase 1: ワイヤーフレーム表示（1週間）

デバッグ・検証用の軽量表示を実装

#### Day 1-2: ViewModel層（ワイヤーフレーム）

- [ ] `octree_converter.rs` 作成
- [ ] `octree_to_wireframe()` 実装
- [ ] `bbox_to_line_vertices()` 実装
- [ ] 単体テスト作成
- [ ] カラーマップ実装

#### Day 3-4: View層（ワイヤーフレーム）

- [ ] `octree_stage.rs` 作成
- [ ] `OctreeStage` 実装（LineResources利用）
- [ ] アニメーション実装
- [ ] 深さフィルタリング

#### Day 5-6: App層統合

- [ ] `app_state.rs` 修正
- [ ] キー入力実装（`0`, `[`, `]`, `P`, `S`）
- [ ] 統合テスト
- [ ] パフォーマンス測定（1000ノード）

#### Day 7: ドキュメント・Phase 1完了

- [ ] Rustdoc整備
- [ ] 使用例作成（`examples/octree_wireframe.rs`）
- [ ] スクリーンショット撮影
### Phase 1完了条件（ワイヤーフレーム）

- [ ] `octree_to_wireframe()` 実装完了
- [ ] `OctreeStage` 実装完了
- [ ] キー入力対応完了（`0`, `[`, `]`, `P`, `S`）
- [ ] 深さ0-8の表示確認
- [ ] アニメーション動作確認
- [ ] 60FPS維持（1000ノード）
- [ ] API ドキュメント整備
- [ ] スクリーンショット3枚以上

### Phase 2完了条件（シェーディング）

- [ ] `voxel_octree_to_mesh()` 実装完了
- [ ] `VoxelShadingStage` 実装完了
- [ ] モード切替実装（`M`キー）
- [ ] 状態フィルタ実装（`1`, `2`, `3`キー）
- [ ] ライティング実装
- [ ] 30FPS以上維持（10000ボクセル）
- [ ] Octree検証統合例作成
- [ ] Before/Afterスクリーンショット
#### Day 1-2: ViewModel層（シェーディング）

**ファイル**: `viewmodel/converter/src/voxel_mesh_converter.rs` (新規)

- [ ] `VoxelMeshOptions` 構造体
- [ ] `voxel_octree_to_mesh()` 実装
  - VoxelNode走査
  - Solidボクセル → 立方体メッシュ変換
  - 法線ベクトル計算
- [ ] 状態別カラーマップ（Solid=青, Mixed=黄, Empty=透明）
- [ ] 単体テスト

**データ構造**:
```rust
/// ボクセルメッシュ変換オプション
pub struct VoxelMeshOptions {
    /// 表示するボクセル状態
    pub show_states: Vec<VoxelState>,
    
    /// 状態による色分け
    pub color_by_state: bool,
    
    /// 表示深さ範囲
    pub depth_range: Range<usize>,
    
    /// ワイヤーフレームモード（Phase 1との互換性）
    pub wireframe_mode: bool,
}

/// ボクセルメッシュ変換
pub fn voxel_octree_to_mesh<T: Scalar>(
    voxel_octree: &VoxelOctree<T>,
    options: &VoxelMeshOptions,
) -> (Vec<Vertex3D>, Vec<u32>) {
    // 頂点・インデックス生成
}

/// 立方体メッシュ生成（36頂点 = 6面 × 2三角形 × 3頂点）
fn cube_to_mesh(
    bbox: &Aabb3D<f32>,
    state: VoxelState,
    color: [f32; 3],
) -> (Vec<Vertex3D>, Vec<u32>) {
    // 各面の法線計算
    // 三角形メッシュ生成
}
```

#### Day 3-4: View層（シェーディング）

**ファイル**: `view/stage/src/voxel_shading_stage.rs` (新規)

- [ ] `VoxelShadingStage` 実装
- [ ] ShadingResources 作成（render_3d.wgsl利用）
- [ ] ライティング設定（方向光源）
- [ ] カメラ連動
- [ ] 深さバッファ有効化

**実装**:
```rust
/// VoxelShadingStage - ボクセル陰影表示ステージ
pub struct VoxelShadingStage {
    /// メッシュ描画リソース
    shading_resources: ShadingResources,
    
    /// 頂点バッファ
    vertices: Vec<Vertex3D>,
    
    /// インデックスバッファ
    indices: Vec<u32>,
    
    /// ライト設定
    light_direction: [f32; 3],
    
    /// 表示オプション
    options: VoxelMeshOptions,
}

impl RenderStage for VoxelShadingStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        // render_3d.wgsl を使用したシェーディング描画
        self.shading_resources.render(encoder, view);
    }
}
```

#### Day 5: 切り替え機能実装

**ファイル**: `view/app/src/app_state.rs` (修正)

- [ ] 表示モード切替実装
  - `M`キー: ワイヤーフレーム ⇔ シェーディング
- [ ] ステージ切り替えロジック
- [ ] 状態フィルタUI（`1`=Solid, `2`=Mixed, `3`=Empty）

**キーバインド追加**:
| キー | 機能 | 説明 |
|------|------|------|
| `M` | モード切替 | ワイヤーフレーム ⇔ シェーディング |
| `1` | Solid表示切替 | Solidボクセル表示ON/OFF |
| `2` | Mixed表示切替 | Mixedボクセル表示ON/OFF |
| `3` | Empty表示切替 | Emptyボクセル表示ON/OFF |
| `L` | ライト回転 | 光源方向を回転 |

#### Day 6: パフォーマンス最適化

- [ ] メッシュ結合（隣接ボクセル）
- [ ] フラスタムカリング
- [ ] LOD実装（遠くは低解像度）
- [ ] パフォーマンス測定（10000ボクセル）

#### Day 7: ドキュメント・Phase 2完了

- [ ] Rustdoc整備
- [ ] 使用例作成（`examples/voxel_shading.rs`）
- [ ] Octree検証統合例
- [ ] スクリーンショット撮影（Before/After）
- [ ] Phase 2完了報告

---

### 📊 Phase別機能比較

| 機能 | Phase 1（ワイヤーフレーム） | Phase 2（シェーディング） |
|------|--------------------------|------------------------|
| **用途** | デバッグ・構造確認 | 切削結果検証 |
| **描画方式** | LineList（辺のみ） | TriangleList（面） |
| **頂点数** | 24/ボクセル（12辺） | 36/ボクセル（6面） |
| **シェーダ** | wireframe.wgsl | render_3d.wgsl |
| **ライティング** | なし | 方向光源 |
| **状態表示** | 深さで色分け | Solid/Mixed/Emptyで色分け |
| **パフォーマンス** | 高速 | 中速（最適化で向上） |
| **実装時間** | 1週間 | 1週間 |

## 🎯 完了条件

- [ ] `octree_to_wireframe()` 実装完了
- [ ] `OctreeStage` 実装完了
- [ ] キー入力対応完了
- [ ] 深さ0-8の表示確認
- [ ] アニメーション動作確認
- [ ] 60FPS維持（1000ノード）
- [ ] API ドキュメント整備
- [ ] スクリーンショット3枚以上

## 📚 関連情報

- **Issue**: [#207 Octree可視化](https://github.com/RedRing2020/RedRing/issues/207)
- **依存**: [#206 Octree実装](https://github.com/RedRing2020/RedRing/issues/206)
- **参考**: [#204 形状可視化](https://github.com/RedRing2020/RedRing/issues/204)
- **設計**: `OCTREE_DESIGN.md`
