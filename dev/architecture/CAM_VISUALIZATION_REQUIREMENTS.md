# CAM可視化システム 要件定義・技術調査

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**ステータス**: ドラフト - 対象範囲確認中  
**関連Issue**: #203

---

## 🎯 対象範囲と段階的実装方針

### 設計確認事項（2026年2月8日）

**質問1**: 2D CAM及び3D CAMの3軸加工が対象か？5軸加工は対象外か？  
**回答**: 
- ✅ **Phase 1**: 2D CAM（輪郭加工、ポケット加工）および3D CAM（等高線粗取り・仕上げ）の**3軸加工のみ**を対象
- ❌ **5軸加工**: Phase 4以降で検討（工具姿勢の可視化が必要となるため、大幅な拡張が必要）

**質問2**: ワーク座標はZ軸固定か？穴あけ時には副座標（subaxis）機能が必要か？  
**回答**:
- ✅ **Phase 1**: Z軸固定の3軸加工を前提、ワーク座標系はグローバル座標系と一致
- 📝 **副座標（subaxis）機能**: Phase 2で実装予定（Week 10-12）
  - 穴あけ・側面加工時のワーク座標系切り替え
  - 複数座標系の管理と可視化

**質問3**: ツールセット登録等のデータ管理は別機能か？  
**回答**:
- ✅ **Phase 1**: 工具情報（径、長さ、種別）のみ保持、DB連携なし
- 📝 **ツール管理機能**: 別Issue（#211予定）で実装
  - ツールセット登録
  - 工具ライブラリ管理
  - 工具選択UI

**質問4**: 等高線加工はツールパスを全体と等高線ごとに管理して視覚制御するか？  
**回答**:
- ✅ **Phase 1で実装**: `ContourLevelPath` 構造体により等高線ごとに管理
- ✅ **表示制御**: 等高線レベルごとの個別ON/OFF、フォーカス表示
- ✅ **アニメーション**: 等高線レベルごとの順次再生

**質問5**: エアカット、パス間の切削開始・終了（直線角度 vs 円弧接続）などの要素定義は？  
**回答**:
- ✅ **Phase 1**: `SegmentType` enumによる識別（Cutting, Approach, Retract, AirCut）
- ✅ **Phase 1**: エアカット区間の表示（グレー、半透明）
- 📝 **Phase 2**: パス接続方式（`PathConnectionType`）の詳細可視化
  - 直線角度接続（LinearAngle）
  - 円弧接続（ArcConnection）

**質問6**: ダウンカット/アップカットの識別は？  
**回答**:
- ✅ **Phase 1**: `CuttingDirection` enumによる識別（Down, Up）
- ✅ **Phase 1**: メタデータに保持、色分け表示オプションあり
  - ダウンカット: 白色
  - アップカット: オレンジ色（発泡スチロール等で使用）

**質問7**: F値（送り速度）は切削量により変動するか？  
**回答**:
- ✅ **Phase 1**: 送り速度を**セグメントごと**に保持（`PathSegment::feed_rate`）
- ✅ **Phase 1**: F値の表示オプション（デバッグ用）
- 📝 **Phase 3**: 切削量による送り速度制御の可視化（Week 15-17）
  - 色グラデーション表示
  - F値の動的調整確認

---

### Phase 1スコープ（Issue #203: 今回実装）

**対象範囲**:
- ✅ **2D CAM**: 輪郭加工、ポケット加工（Z軸固定、3軸加工）
- ✅ **3D CAM**: 等高線粗取り、等高線仕上げ（3軸加工のみ）
- ❌ **5軸加工**: 対象外（Phase 4以降で検討）
- ❌ **ツール管理**: 登録・データ管理は別機能（Issue #211で検討予定）
- ❌ **副座標（subaxis）機能**: Phase 2以降で検討

**実装する機能**:
- 工具経路の基本表示（線分、円弧）
- 切削種別の色分け（切削/早送り/アプローチ/退避）
- 送り速度（F値）の保持と表示
- ダウンカット/アップカットの識別
- エアカット区間の表示（破線表示）

**実装しない機能（将来拡張）**:
- 工具姿勢の可視化（5軸用）
- 副座標系の切り替え表示
- ツールセットDB連携
- リアルタイム切削シミュレーション

### Phase 2以降の拡張予定

**Phase 2**: 詳細パス制御（Week 10-12）
- [ ] パス接続方式（直線角度 vs 円弧接続）の可視化
- [ ] 等高線ごとのツールパス管理・制御
- [ ] 副座標（subaxis）機能の基礎

**Phase 3**: 切削最適化表示（Week 15-17）
- [ ] 切削量による送り速度制御の可視化
- [ ] ダウンカット/アップカット切り替えの詳細表示
- [ ] 過切削・削り残し検出の警告表示

**Phase 4**: 5軸加工対応（Q2以降）
- [ ] 工具姿勢（傾斜角）の可視化
- [ ] 5軸同時加工パスの表示
- [ ] 干渉チェック結果の表示

---

## 📋 目次

1. [要件定義](#要件定義)
2. [データモデル詳細設計](#データモデル詳細設計)
3. [技術調査](#技術調査)
4. [アーキテクチャ設計](#アーキテクチャ設計)
5. [実装計画](#実装計画)
6. [テスト戦略](#テスト戦略)

---

## 要件定義

### 1. 表示対象

#### 1.1 工具経路（ToolPath）表示

**必須機能**:
- [ ] 線分ベースのパス表示
  - 切削送り（G01, G02, G03）
  - 早送り（G00）
- [ ] 色分け表示
  - 早送り: 青色（RGB: 0.2, 0.5, 1.0）
  - 切削送り: 白色（RGB: 1.0, 1.0, 1.0）
  - アプローチ: 緑色（RGB: 0.2, 1.0, 0.2）
  - 退避: 黄色（RGB: 1.0, 1.0, 0.2）
- [ ] 線の太さ調整（1-5ピクセル）

**将来機能**:
- [ ] 工具補正表示（左/右補正の可視化）
- [ ] 送り速度による色グラデーション
- [ ] 加工順序の番号表示

#### 1.2 オフセット結果表示

**2D輪郭オフセット**:
- [ ] オリジナル輪郭（グレー）
- [ ] オフセット輪郭（赤色）
- [ ] オフセット方向矢印表示

**3Dメッシュオフセット**:
- [ ] オリジナルメッシュ（ワイヤーフレーム、グレー）
- [ ] オフセットメッシュ（ソリッド、緑色）
- [ ] 法線ベクトル表示（オプション）

#### 1.3 工具形状表示

**Phase 3機能（後回し）**:
- [ ] 円筒工具（エンドミル）
  - 工具径の円筒表示
  - 工具長さ表示
- [ ] ボールエンドミル
  - 半球形状表示
- [ ] 工具姿勢（傾斜角）の可視化

#### 1.4 加工シミュレーション結果

**Phase 4機能（将来）**:
- [ ] 削り残しメッシュ表示
- [ ] 過切削領域の警告表示（赤色）
- [ ] ワーク形状の透過表示

---

### 2. 表示モード

#### 2.1 ワイヤーフレームモード
- 線分のみ表示
- 背景透過
- 軽量・高速描画

#### 2.2 ソリッドカラーモード
- メッシュの塗りつぶし表示
- 単色または色分け

#### 2.3 工具径表示モード
- 工具中心パスに加えて工具外形を表示
- 円筒プリミティブによる可視化

---

### 3. インタラクション

#### 3.1 パス上の任意点選択
- [ ] マウスクリックでパス上の点を選択
- [ ] 選択点の座標表示
- [ ] 選択点のパラメータ表示（t値、送り速度など）

#### 3.2 パラメータ値表示（HUD）
- [ ] 工具経路長さ
- [ ] 推定加工時間
- [ ] 最大/最小送り速度
- [ ] 最大/最小Z高さ

#### 3.3 アニメーション再生
- [ ] 再生/一時停止/停止
- [ ] 再生速度調整（0.1x - 10x）
- [ ] スライダーによる位置指定
- [ ] ステップ実行（1セグメントずつ進む）

---

## 技術調査

### 1. 大量パス表示の最適化

#### 1.1 インスタンシング

**調査結果**:
- wgpu の `draw_indexed()` でインスタンシング対応
- 同一形状を複数描画する際に有効
- 工具形状の大量描画に適用可能

**実装方針**:
```rust
// インスタンスデータをGPUバッファに格納
struct InstanceData {
    position: [f32; 3],
    rotation: [f32; 4], // クォータニオン
    scale: f32,
}

// シェーダ側でインスタンスごとに変換
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) instance_pos: vec3<f32>,
    @location(2) instance_rot: vec4<f32>,
    @location(3) instance_scale: f32,
) -> VertexOutput {
    // インスタンスごとの変換を適用
}
```

**メリット**:
- GPU側で並列処理
- CPUからGPUへのデータ転送量削減
- 1000個以上の工具形状を60FPS以上で描画可能

**デメリット**:
- 全インスタンスが同一形状である必要がある
- 動的な形状変更には不向き

#### 1.2 LOD（Level of Detail）

**調査結果**:
- カメラ距離に応じて詳細度を変更
- 遠方のパスは低解像度、近傍は高解像度

**実装方針**:
```rust
enum LODLevel {
    High,    // カメラ距離 < 100
    Medium,  // 100 <= 距離 < 500
    Low,     // 距離 >= 500
}

impl ToolPath {
    fn tessellate_with_lod(&self, camera_pos: Point3D, level: LODLevel) -> Vec<Vertex> {
        match level {
            LODLevel::High => self.tessellate(64),   // 円弧を64分割
            LODLevel::Medium => self.tessellate(32), // 32分割
            LODLevel::Low => self.tessellate(16),    // 16分割
        }
    }
}
```

**メリット**:
- 描画負荷の削減
- 遠方の細かいディテールは視覚的に不要

**デメリット**:
- LOD切り替え時のポッピング（突然の見た目変化）
- 実装の複雑化

**推奨**:
- Phase 1では LOD なし
- パフォーマンス問題が発生した場合に Phase 2 で導入

---

### 2. 工具形状の効率的な描画

#### 2.1 円筒プリミティブの描画

**調査項目**:
- [ ] プリミティブメッシュの事前生成
  - 円筒メッシュ（側面）
  - 円盤メッシュ（上下面）
- [ ] インスタンシングによる大量描画
- [ ] シェーダによる手続き的生成（Procedural）

**実装方針（推奨）**:
```rust
// モデル層で円筒メッシュを事前生成
pub struct CylinderMesh {
    radius: f32,
    height: f32,
    segments: u32, // 円周分割数
}

impl CylinderMesh {
    pub fn generate_vertices(&self) -> Vec<Vertex3D> {
        // 円筒メッシュ生成
        // segments 個の四角形で側面を構成
    }
}

// View層でインスタンシング描画
pub struct ToolShapeResources {
    cylinder_mesh: CylinderMesh,
    instance_buffer: wgpu::Buffer,
}
```

**パフォーマンス目標**:
- 1000個の円筒工具を60FPS以上で描画

#### 2.2 GPUインスタンシングの活用

**wgpu実装例**:
```rust
// インスタンスバッファの作成
let instance_data: Vec<InstanceData> = tool_positions
    .iter()
    .map(|pos| InstanceData {
        position: [pos.x as f32, pos.y as f32, pos.z as f32],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: tool_radius as f32,
    })
    .collect();

let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Instance Buffer"),
    contents: bytemuck::cast_slice(&instance_data),
    usage: wgpu::BufferUsages::VERTEX,
});

// 描画時
render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
render_pass.draw(0..vertex_count, 0..instance_count);
```

---

### 3. アニメーション制御のアーキテクチャ

#### 3.1 タイムライン管理

**設計方針**:
```rust
pub struct AnimationController {
    current_time: f64,        // 現在時刻（秒）
    total_duration: f64,      // 総時間（秒）
    playback_speed: f64,      // 再生速度（1.0 = 等倍）
    is_playing: bool,         // 再生中フラグ
    loop_enabled: bool,       // ループ再生フラグ
}

impl AnimationController {
    pub fn update(&mut self, delta_time: f64) {
        if self.is_playing {
            self.current_time += delta_time * self.playback_speed;
            
            if self.current_time >= self.total_duration {
                if self.loop_enabled {
                    self.current_time = 0.0;
                } else {
                    self.is_playing = false;
                    self.current_time = self.total_duration;
                }
            }
        }
    }
    
    pub fn get_normalized_time(&self) -> f64 {
        self.current_time / self.total_duration
    }
}
```

#### 3.2 再生速度制御

**速度オプション**:
- 0.1x（スロー再生、詳細確認用）
- 0.5x
- 1.0x（等倍、デフォルト）
- 2.0x
- 5.0x
- 10.0x（高速プレビュー）

**UI制御**:
- スライダーによる連続調整
- プリセットボタン（0.1x, 1x, 5x, 10x）

---

## データモデル詳細設計

### Phase 1データモデル（3軸加工対応）

#### ワーク座標系の定義

**Phase 1の方針**:
- Z軸固定の3軸加工を前提
- ワーク座標系はグローバル座標系と一致
- 副座標（subaxis）機能はPhase 2で実装

```rust
/// ワーク座標系（Phase 1: 簡易版）
#[derive(Debug, Clone)]
pub struct WorkCoordinateSystem<T: Scalar> {
    /// 原点位置
    pub origin: Point3D<T>,
    
    /// Z軸方向（固定）
    pub z_axis: Vector3D<T>,
}
```

#### 工具定義（Phase 1: 最小限）

**Phase 1の方針**:
- 工具径、工具長のみ保持
- ツールセットDB連携はPhase 2以降
- ツール登録・管理機能は別Issue（#211予定）で実装

```rust
/// 工具情報（Phase 1: 簡易版）
#[derive(Debug, Clone)]
pub struct Tool<T: Scalar> {
    /// 工具番号
    pub tool_number: u32,
    
    /// 工具径
    pub diameter: T,
    
    /// 工具長
    pub length: T,
    
    /// 工具種別
    pub tool_type: ToolType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    /// エンドミル（フラット）
    FlatEndMill,
    
    /// ボールエンドミル
    BallEndMill,
    
    /// ドリル
    Drill,
}
```

---

## アーキテクチャ設計

### 1. データモデル（Model層）

#### 1.1 ToolPath構造体（Phase 1拡張版）

```rust
// model/geo_algorithms/src/toolpath.rs

use geo_primitives::{LineSegment3D, Arc3D, Point3D};
use analysis::Scalar;

/// 工具経路セグメント
#[derive(Debug, Clone)]
pub enum PathSegment<T: Scalar> {
    /// 直線補間（G01）
    Linear {
        segment: LineSegment3D<T>,
        segment_type: SegmentType,
        feed_rate: T, // mm/min（切削量により変動）
    },
    
    /// 円弧補間（G02/G03）
    Arc {
        arc: Arc3D<T>,
        direction: ArcDirection,
        segment_type: SegmentType,
        feed_rate: T,
    },
    
    /// 早送り（G00）
    Rapid {
        segment: LineSegment3D<T>,
    },
}

/// セグメント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    /// 切削送り（通常）
    Cutting,
    
    /// アプローチ（切削開始）
    Approach,
    
    /// 退避（切削終了）
    Retract,
    
    /// エアカット（材料に接触しない移動）
    AirCut,
}

/// 円弧方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcDirection {
    /// 時計回り（G02）
    Clockwise,
    
    /// 反時計回り（G03）
    CounterClockwise,
}

/// カッティング方向（Phase 1: 識別のみ）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuttingDirection {
    /// ダウンカット（順送り、基本）
    Down,
    
    /// アップカット（逆送り、発泡スチロール等）
    Up,
}

/// パス接続方式（Phase 2実装予定）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathConnectionType {
    /// 直線角度で接続
    LinearAngle { angle_degrees: f64 },
    
    /// 円弧で接続
    ArcConnection { radius: f64 },
}

impl<T: Scalar> PathSegment<T> {
    /// セグメントの長さを取得
    pub fn length(&self) -> T {
        match self {
            Self::Linear { segment, .. } => segment.length(),
            Self::Arc { arc, .. } => arc.arc_length(),
            Self::Rapid { segment } => segment.length(),
        }
    }
    
    /// パラメータ t (0.0-1.0) の位置を取得
    pub fn point_at(&self, t: T) -> Point3D<T> {
        match self {
            Self::Linear { segment, .. } => segment.point_at_parameter(t),
            Self::Arc { arc, .. } => arc.point_at_parameter(t),
            Self::Rapid { segment } => segment.point_at_parameter(t),
        }
    }
    
    /// セグメント種別を取得
    pub fn segment_type(&self) -> Option<SegmentType> {
        match self {
            Self::Linear { segment_type, .. } => Some(*segment_type),
            Self::Arc { segment_type, .. } => Some(*segment_type),
            Self::Rapid { .. } => None,
        }
    }
    
    /// 送り速度を取得（mm/min）
    pub fn feed_rate(&self) -> Option<T> {
        match self {
            Self::Linear { feed_rate, .. } => Some(*feed_rate),
            Self::Arc { feed_rate, .. } => Some(*feed_rate),
            Self::Rapid { .. } => None,
        }
    }
}

/// 工具経路メタデータ
#[derive(Debug, Clone)]
pub struct PathMetadata {
    /// 加工番号（オペレーション番号）
    pub operation_id: u32,
    
    /// 説明
    pub description: String,
    
    /// 推定加工時間（秒）
    pub estimated_time: f64,
    
    /// カッティング方向
    pub cutting_direction: CuttingDirection,
}

/// 等高線パス（等高線加工用、Phase 1で基本対応）
#[derive(Debug, Clone)]
pub struct ContourLevelPath<T: Scalar> {
    /// Z高さ
    pub z_level: T,
    
    /// このレベルのパスセグメント
    pub segments: Vec<PathSegment<T>>,
    
    /// レベル番号（0が最上層）
    pub level_index: usize,
}

/// 工具経路全体
#[derive(Debug, Clone)]
pub struct ToolPath<T: Scalar> {
    /// パスセグメント列
    segments: Vec<PathSegment<T>>,
    
    /// 工具情報
    tool: Tool<T>,
    
    /// メタデータ
    metadata: PathMetadata,
    
    /// 等高線パス（等高線加工の場合）
    contour_levels: Option<Vec<ContourLevelPath<T>>>,
}

impl<T: Scalar> ToolPath<T> {
    /// 新規作成
    pub fn new(
        segments: Vec<PathSegment<T>>,
        tool: Tool<T>,
        metadata: PathMetadata,
    ) -> Self {
        Self {
            segments,
            tool,
            metadata,
            contour_levels: None,
        }
    }
    
    /// 等高線加工パスとして作成
    pub fn with_contour_levels(
        segments: Vec<PathSegment<T>>,
        tool: Tool<T>,
        metadata: PathMetadata,
        contour_levels: Vec<ContourLevelPath<T>>,
    ) -> Self {
        Self {
            segments,
            tool,
            metadata,
            contour_levels: Some(contour_levels),
        }
    }
    
    /// 総パス長さを計算
    pub fn total_length(&self) -> T {
        self.segments.iter().map(|seg| seg.length()).sum()
    }
    
    /// セグメント数
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    
    /// セグメント参照
    pub fn segments(&self) -> &[PathSegment<T>] {
        &self.segments
    }
    
    /// 等高線パス参照
    pub fn contour_levels(&self) -> Option<&[ContourLevelPath<T>]> {
        self.contour_levels.as_deref()
    }
    
    /// 工具情報参照
    pub fn tool(&self) -> &Tool<T> {
        &self.tool
    }
}
```

#### 1.2 OffsetResult構造体

```rust
// model/geo_algorithms/src/offset.rs

/// 2D輪郭オフセット結果
#[derive(Debug, Clone)]
pub struct OffsetResult2D<T: Scalar> {
    /// オリジナル輪郭
    original: Vec<LineSegment2D<T>>,
    
    /// オフセット輪郭
    offset: Vec<LineSegment2D<T>>,
    
    /// オフセット距離
    distance: T,
    
    /// 内部島（ポケット加工時）
    islands: Vec<Vec<LineSegment2D<T>>>,
}

/// 3Dメッシュオフセット結果
#[derive(Debug, Clone)]
pub struct OffsetResult3D<T: Scalar> {
    /// オリジナルメッシュ
    original: TriangleMesh3D<T>,
    
    /// オフセットメッシュ
    offset: TriangleMesh3D<T>,
    
    /// オフセット距離
    distance: T,
}
```

---

### 2. ViewModel層変換（ViewModel層）

#### 2.1 ToolPath変換（Phase 1拡張版）

```rust
// viewmodel/converter/src/toolpath_converter.rs

use model_geo_algorithms::toolpath::{ToolPath, PathSegment, SegmentType, CuttingDirection};
use view_render::vertex_3d::Vertex3D;

/// 可視化オプション
#[derive(Debug, Clone)]
pub struct VisualizationOptions {
    /// 早送りを表示するか
    pub show_rapid_moves: bool,
    
    /// エアカットを表示するか
    pub show_air_cuts: bool,
    
    /// 工具表示モード
    pub tool_display: ToolDisplayMode,
    
    /// パスの線の太さ（ピクセル）
    pub path_thickness: f32,
    
    /// テセレーション品質（円弧の分割数）
    pub arc_segments: u32,
    
    /// 送り速度の表示
    pub show_feed_rates: bool,
    
    /// カッティング方向の色分け
    pub color_by_cutting_direction: bool,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            show_rapid_moves: true,
            show_air_cuts: true,
            tool_display: ToolDisplayMode::None,
            path_thickness: 2.0,
            arc_segments: 32,
            show_feed_rates: false,
            color_by_cutting_direction: false,
        }
    }
}

/// 工具表示モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDisplayMode {
    /// 表示しない
    None,
    
    /// 簡易表示（点のみ）
    Simple,
    
    /// 円筒表示
    Cylinder,
}

/// セグメント種別ごとの色定義
pub mod colors {
    /// 切削送り（白色）
    pub const CUTTING: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    
    /// 早送り（青色）
    pub const RAPID: [f32; 4] = [0.2, 0.5, 1.0, 1.0];
    
    /// アプローチ（緑色）
    pub const APPROACH: [f32; 4] = [0.2, 1.0, 0.2, 1.0];
    
    /// 退避（黄色）
    pub const RETRACT: [f32; 4] = [1.0, 1.0, 0.2, 1.0];
    
    /// エアカット（グレー、破線表示用）
    pub const AIR_CUT: [f32; 4] = [0.5, 0.5, 0.5, 0.7];
    
    /// ダウンカット（デフォルト白色）
    pub const DOWN_CUT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    
    /// アップカット（オレンジ色）
    pub const UP_CUT: [f32; 4] = [1.0, 0.6, 0.2, 1.0];
}

/// 工具経路頂点データ
#[derive(Debug)]
pub struct ToolPathVertices {
    /// パス線分の頂点
    pub path_vertices: Vec<Vertex3D>,
    
    /// 工具形状の頂点（オプション）
    pub tool_vertices: Option<Vec<Vertex3D>>,
    
    /// 色情報（各頂点ごと）
    pub vertex_colors: Vec<[f32; 4]>,
    
    /// 送り速度情報（デバッグ用、オプション）
    pub feed_rates: Option<Vec<f32>>,
}

/// 工具経路をGPU頂点データに変換
pub fn toolpath_to_vertices(
    path: &ToolPath<f64>,
    options: &VisualizationOptions,
) -> ToolPathVertices {
    let mut path_vertices = Vec::new();
    let mut vertex_colors = Vec::new();
    let mut feed_rates = Vec::new();
    
    for segment in path.segments() {
        let (vertices, color, feed_rate) = match segment {
            PathSegment::Linear { segment, segment_type, feed_rate } => {
                // セグメント種別による色分け
                let color = match segment_type {
                    SegmentType::Cutting => colors::CUTTING,
                    SegmentType::Approach => colors::APPROACH,
                    SegmentType::Retract => colors::RETRACT,
                    SegmentType::AirCut => {
                        if !options.show_air_cuts {
                            continue;
                        }
                        colors::AIR_CUT
                    },
                };
                
                let v = vec![
                    vertex_from_point(segment.start_position()),
                    vertex_from_point(segment.end_position()),
                ];
                (v, color, Some(*feed_rate as f32))
            },
            
            PathSegment::Arc { arc, segment_type, feed_rate, .. } => {
                let color = match segment_type {
                    SegmentType::Cutting => colors::CUTTING,
                    SegmentType::Approach => colors::APPROACH,
                    SegmentType::Retract => colors::RETRACT,
                    SegmentType::AirCut => {
                        if !options.show_air_cuts {
                            continue;
                        }
                        colors::AIR_CUT
                    },
                };
                
                let v = tessellate_arc(arc, options.arc_segments);
                (v, color, Some(*feed_rate as f32))
            },
            
            PathSegment::Rapid { segment } => {
                if !options.show_rapid_moves {
                    continue;
                }
                let v = vec![
                    vertex_from_point(segment.start_position()),
                    vertex_from_point(segment.end_position()),
                ];
                (v, colors::RAPID, None)
            },
        };
        
        // カッティング方向による色上書き（オプション）
        let final_color = if options.color_by_cutting_direction {
            match path.metadata().cutting_direction {
                CuttingDirection::Down => colors::DOWN_CUT,
                CuttingDirection::Up => colors::UP_CUT,
            }
        } else {
            color
        };
        
        // 頂点と色を追加
        for vertex in vertices {
            path_vertices.push(vertex);
            vertex_colors.push(final_color);
            if let Some(fr) = feed_rate {
                feed_rates.push(fr);
            }
        }
    }
    
    // 工具形状の生成（オプション）
    let tool_vertices = match options.tool_display {
        ToolDisplayMode::None => None,
        ToolDisplayMode::Simple => Some(generate_tool_points(path)),
        ToolDisplayMode::Cylinder => Some(generate_tool_cylinders(path)),
    };
    
    ToolPathVertices {
        path_vertices,
        tool_vertices,
        vertex_colors,
        feed_rates: if options.show_feed_rates && !feed_rates.is_empty() {
            Some(feed_rates)
        } else {
            None
        },
    }
}

fn vertex_from_point(p: &Point3D<f64>) -> Vertex3D {
    Vertex3D {
        position: [p.x() as f32, p.y() as f32, p.z() as f32],
    }
}

fn tessellate_arc(arc: &Arc3D<f64>, segments: u32) -> Vec<Vertex3D> {
    // 円弧を線分に分割
    (0..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            vertex_from_point(&arc.point_at_parameter(t))
        })
        .collect()
}

fn generate_tool_points(path: &ToolPath<f64>) -> Vec<Vertex3D> {
    // Phase 3実装予定
    Vec::new()
}

fn generate_tool_cylinders(path: &ToolPath<f64>) -> Vec<Vertex3D> {
    // Phase 3実装予定
    Vec::new()
}
```

/// 工具経路頂点データ
#[derive(Debug)]
pub struct ToolPathVertices {
    /// パス線分の頂点
    pub path_vertices: Vec<Vertex3D>,
    
    /// 工具形状の頂点（オプション）
    pub tool_vertices: Option<Vec<Vertex3D>>,
    
    /// 色情報（セグメント種別ごと）
    pub colors: Vec<[f32; 4]>,
}

/// 工具経路をGPU頂点データに変換
pub fn toolpath_to_vertices(
    path: &ToolPath<f64>,
    options: &VisualizationOptions,
) -> ToolPathVertices {
    let mut path_vertices = Vec::new();
    let mut colors = Vec::new();
    
    for segment in path.segments() {
        let (vertices, color) = match segment {
            PathSegment::Linear(seg) => {
                let v = vec![
                    vertex_from_point(seg.start_position()),
                    vertex_from_point(seg.end_position()),
                ];
                (v, [1.0, 1.0, 1.0, 1.0]) // 白色
            },
            PathSegment::Arc(arc) => {
                let v = tessellate_arc(arc, options.arc_segments);
                (v, [1.0, 1.0, 1.0, 1.0]) // 白色
            },
            PathSegment::Rapid(seg) => {
                if !options.show_rapid_moves {
                    continue;
                }
                let v = vec![
                    vertex_from_point(seg.start_position()),
                    vertex_from_point(seg.end_position()),
                ];
                (v, [0.2, 0.5, 1.0, 1.0]) // 青色
            },
        };
        
        path_vertices.extend(vertices);
        colors.push(color);
    }
    
    // 工具形状の生成（オプション）
    let tool_vertices = match options.tool_display {
        ToolDisplayMode::None => None,
        ToolDisplayMode::Simple => Some(generate_tool_points(path)),
        ToolDisplayMode::Cylinder => Some(generate_tool_cylinders(path)),
    };
    
    ToolPathVertices {
        path_vertices,
        tool_vertices,
        colors,
    }
}

fn vertex_from_point(p: &Point3D<f64>) -> Vertex3D {
    Vertex3D {
        position: [p.x() as f32, p.y() as f32, p.z() as f32],
    }
}

fn tessellate_arc(arc: &Arc3D<f64>, segments: u32) -> Vec<Vertex3D> {
    // 円弧を線分に分割
    (0..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            vertex_from_point(&arc.point_at_parameter(t))
        })
        .collect()
}
```

---

### 3. View層レンダリング（View層）

#### 3.1 ToolPathResources

```rust
// view/render/src/toolpath.rs

use wgpu;
use std::sync::Arc;

pub struct ToolPathResources {
    device: Arc<wgpu::Device>,
    
    /// パス描画パイプライン
    path_pipeline: wgpu::RenderPipeline,
    
    /// 工具描画パイプライン（オプション）
    tool_pipeline: Option<wgpu::RenderPipeline>,
    
    /// 頂点バッファ
    vertex_buffer: Option<wgpu::Buffer>,
    
    /// 頂点数
    vertex_count: u32,
    
    /// ユニフォームバッファ（カメラ行列）
    uniform_buffer: wgpu::Buffer,
    
    /// バインドグループ
    bind_group: wgpu::BindGroup,
}

impl ToolPathResources {
    pub fn new(device: &Arc<wgpu::Device>, format: wgpu::TextureFormat) -> Self {
        // パイプライン構築
        let path_pipeline = create_toolpath_pipeline(device, format);
        
        // ユニフォームバッファ作成
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ToolPath Uniform Buffer"),
            size: 64, // mat4x4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // バインドグループ作成
        let bind_group_layout = path_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ToolPath Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        Self {
            device: Arc::clone(device),
            path_pipeline,
            tool_pipeline: None,
            vertex_buffer: None,
            vertex_count: 0,
            uniform_buffer,
            bind_group,
        }
    }
    
    /// パスデータを更新
    pub fn update_path_data(&mut self, vertices: &[Vertex3D]) {
        if vertices.is_empty() {
            self.vertex_buffer = None;
            self.vertex_count = 0;
            return;
        }
        
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ToolPath Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.vertex_buffer = Some(buffer);
        self.vertex_count = vertices.len() as u32;
    }
    
    /// カメラ行列を更新
    pub fn update_camera(&mut self, queue: &wgpu::Queue, view_proj_matrix: &[[f32; 4]; 4]) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(view_proj_matrix));
    }
    
    /// 描画
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if self.vertex_buffer.is_none() || self.vertex_count == 0 {
            return;
        }
        
        render_pass.set_pipeline(&self.path_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}

fn create_toolpath_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    // シェーダモジュールのロード
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ToolPath Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/toolpath.wgsl").into()),
    });
    
    // パイプライン構築
    // （詳細は省略）
    todo!()
}
```

#### 3.2 toolpath.wgsl シェーダ

```wgsl
// view/render/shaders/toolpath.wgsl

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    output.color = vec4<f32>(1.0, 1.0, 1.0, 1.0); // デフォルト白色
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
```

---

### 4. Stage管理（View層）

```rust
// view/stage/src/toolpath_stage.rs

use view_render::toolpath::ToolPathResources;
use super::render_stage::RenderStage;

pub struct ToolPathStage {
    resources: ToolPathResources,
    animation_controller: AnimationController,
}

impl ToolPathStage {
    pub fn new(device: &Arc<wgpu::Device>, format: wgpu::TextureFormat) -> Self {
        Self {
            resources: ToolPathResources::new(device, format),
            animation_controller: AnimationController::default(),
        }
    }
    
    pub fn set_toolpath(&mut self, vertices: &[Vertex3D]) {
        self.resources.update_path_data(vertices);
    }
    
    pub fn update_camera(&mut self, queue: &wgpu::Queue, view_proj: &[[f32; 4]; 4]) {
        self.resources.update_camera(queue, view_proj);
    }
}

impl RenderStage for ToolPathStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ToolPath Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        self.resources.render(&mut render_pass);
    }
    
    fn update(&mut self) {
        // アニメーション更新などをここで実行
        self.animation_controller.update(0.016); // 60FPS想定
    }
}

struct AnimationController {
    current_time: f64,
    total_duration: f64,
    playback_speed: f64,
    is_playing: bool,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            total_duration: 1.0,
            playback_speed: 1.0,
            is_playing: false,
        }
    }
}

impl AnimationController {
    fn update(&mut self, delta_time: f64) {
        if self.is_playing {
            self.current_time += delta_time * self.playback_speed;
            if self.current_time >= self.total_duration {
                self.current_time = self.total_duration;
                self.is_playing = false;
            }
        }
    }
}
```

---

## 実装計画

### Phase 1実装範囲の再確認

**実装する機能**:
- ✅ `PathSegment` enum拡張（SegmentType, FeedRate, ArcDirection）
- ✅ セグメント種別による色分け（切削/早送り/アプローチ/退避/エアカット）
- ✅ 送り速度（F値）の保持（セグメントごと）
- ✅ ダウンカット/アップカット識別（メタデータ）
- ✅ 等高線パスの基本管理（ContourLevelPath）
- ✅ 等高線レベルごとの表示切り替え

**実装しない機能（Phase 2以降）**:
- ❌ パス接続方式（直線角度 vs 円弧接続）の詳細可視化
- ❌ 切削量による送り速度制御の可視化
- ❌ 5軸加工対応
- ❌ 副座標（subaxis）機能
- ❌ ツール管理DB連携

### Phase 1: 基本的な工具経路表示（2-3日）

**Day 1: データモデル実装**:
- [ ] `model/geo_algorithms/src/toolpath.rs` 実装
  - [ ] `PathSegment` enum（SegmentType, FeedRate付き）
  - [ ] `SegmentType`, `ArcDirection`, `CuttingDirection` enum
  - [ ] `Tool` 構造体（簡易版）
  - [ ] `PathMetadata` 構造体
  - [ ] `ContourLevelPath` 構造体
  - [ ] `ToolPath` 構造体
- [ ] 単体テスト作成
  - [ ] パス長計算
  - [ ] セグメント種別の識別
  - [ ] 送り速度の取得

**Day 2: ViewModel変換実装**:
- [ ] `viewmodel/converter/src/toolpath_converter.rs` 実装
  - [ ] `VisualizationOptions` 構造体
  - [ ] `colors` モジュール（色定義）
  - [ ] `toolpath_to_vertices()` 関数
  - [ ] セグメント種別ごとの色分けロジック
- [ ] `viewmodel/converter/src/contour_level_manager.rs` 実装
  - [ ] `ContourLevelVisibility` 構造体
  - [ ] `contour_levels_to_vertices()` 関数
- [ ] 単体テスト
  - [ ] 頂点変換の正確性
  - [ ] 色分けの確認

**Day 3: View層レンダリング実装**:
- [ ] `view/render/src/toolpath.rs` 実装
  - [ ] `ToolPathResources` 構造体
  - [ ] パイプライン構築
  - [ ] 描画メソッド
- [ ] `view/render/shaders/toolpath.wgsl` 実装
  - [ ] 頂点シェーダ
  - [ ] フラグメントシェーダ（頂点カラー対応）
- [ ] `view/stage/src/toolpath_stage.rs` 実装
  - [ ] `ToolPathStage` 構造体
  - [ ] `RenderStage` トレイト実装
- [ ] 統合テスト
  - [ ] サンプル経路の描画確認

### Phase 2: 詳細パス制御（将来実装）

**Week 10-12実施予定**:
- [ ] パス接続方式の可視化
- [ ] 等高線ごとの詳細制御（個別ON/OFF、アニメーション）
- [ ] 副座標（subaxis）機能の基礎
- [ ] エアカットの破線表示

---

## テスト戦略

### 単体テスト

**Model層**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_toolpath_length() {
        let seg1 = PathSegment::Linear(LineSegment3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        ));
        let seg2 = PathSegment::Rapid(LineSegment3D::new(
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
        ));
        
        let path = ToolPath::new(
            vec![seg1, seg2],
            5.0,
            1000.0,
            PathMetadata::default(),
        );
        
        assert_eq!(path.total_length(), 20.0);
    }
}
```

**ViewModel層**:
```rust
#[test]
fn test_toolpath_conversion() {
    let path = create_sample_toolpath();
    let options = VisualizationOptions::default();
    
    let vertices = toolpath_to_vertices(&path, &options);
    
    assert!(!vertices.path_vertices.is_empty());
    assert_eq!(vertices.colors.len(), path.segment_count());
}
```

### 統合テスト

**サンプルデータ**:
- [ ] 単純パス（直線のみ、10セグメント）
- [ ] 複雑パス（円弧含む、100セグメント）
- [ ] 大規模パス（1000セグメント以上）

**検証項目**:
- [ ] 描画の正確性（目視確認）
- [ ] FPS測定（60FPS以上）
- [ ] メモリ使用量確認

---

## 次のアクション

1. **即座に着手**:
   - [ ] GitHub Issue作成
   - [ ] ToolPath データ構造実装開始

2. **今週中に完了**:
   - [ ] Phase 1 実装完了
   - [ ] 基本的な工具経路表示の動作確認

3. **来週着手**:
   - [ ] オフセット結果表示の実装
   - [ ] パフォーマンステスト

**次回レビュー**: Phase 1完了時（2026年2月12日予定）

---

## 今後のIssue計画

### Issue #211: ツール管理システム（Phase 2, Week 10-12）

**概要**: ツールセット登録・管理機能の実装

**機能**:
- [ ] ツールライブラリDB（工具径、長さ、種別、メーカー情報）
- [ ] ツールセット登録UI
- [ ] 工具選択・割り当て機能
- [ ] 工具寿命管理（使用時間、摩耗状態）

**依存関係**: Issue #203完了後

---

### Issue #212: 副座標（subaxis）機能（Phase 2, Week 10-12）

**概要**: 穴あけ・側面加工時のワーク座標系切り替え機能

**機能**:
- [ ] 複数ワーク座標系の定義
- [ ] 座標系切り替えコマンド（G54-G59）
- [ ] 座標系ごとの工具経路表示
- [ ] 座標系原点の可視化

**依存関係**: Issue #203完了後

---

### Issue #213: パス接続詳細表示（Phase 2, Week 10-12）

**概要**: パス間の切削開始・終了方式の詳細可視化

**機能**:
- [ ] 直線角度接続の可視化
- [ ] 円弧接続の可視化
- [ ] 接続パラメータ（角度、半径）の表示
- [ ] 接続方式の編集・プレビュー

**依存関係**: Issue #203完了後

---

### Issue #214: 切削最適化表示（Phase 3, Week 15-17）

**概要**: 切削量による送り速度制御の可視化

**機能**:
- [ ] F値の色グラデーション表示
- [ ] 切削負荷の計算・表示
- [ ] 送り速度の動的調整確認
- [ ] 過切削・削り残しの警告表示

**依存関係**: Issue #203, #206（Octree）完了後

---

### Issue #215: 5軸加工対応（Phase 4, Q2以降）

**概要**: 5軸同時加工の工具経路可視化

**機能**:
- [ ] 工具姿勢（A軸、B軸、C軸）の可視化
- [ ] 5軸同時加工パスの表示
- [ ] 干渉チェック結果の表示
- [ ] 機械座標系との対応表示

**依存関係**: Issue #203, #211, #212完了後

---

## 設計方針のまとめ

### ✅ Phase 1で確実に実装する機能

1. **データモデル**:
   - `PathSegment` enum（SegmentType, FeedRate, ArcDirection）
   - `ToolPath`, `ContourLevelPath`, `Tool` 構造体
   - `CuttingDirection` enum（ダウンカット/アップカット）

2. **可視化**:
   - セグメント種別による色分け（切削/早送り/アプローチ/退避/エアカット）
   - 等高線レベルごとの表示制御
   - 送り速度（F値）の保持と表示

3. **対象範囲**:
   - 2D CAM（輪郭加工、ポケット加工）
   - 3D CAM（等高線粗取り、等高線仕上げ）
   - 3軸加工のみ

### 📝 Phase 2以降で実装する機能

1. **ツール管理**: Issue #211（ツール登録、DB連携）
2. **副座標**: Issue #212（ワーク座標系切り替え）
3. **パス接続**: Issue #213（直線角度 vs 円弧接続の詳細可視化）
4. **切削最適化**: Issue #214（F値の動的調整、切削負荷表示）
5. **5軸加工**: Issue #215（工具姿勢、干渉チェック）

### 🎯 実装優先順位

**Week 3（今週）**: Issue #203 Phase 1実装  
**Week 10-12**: Issue #211, #212, #213（Phase 2機能）  
**Week 15-17**: Issue #214（Phase 3機能）  
**Q2以降**: Issue #215（Phase 4機能）

---

**最終更新**: 2026年2月8日  
**次回更新予定**: Phase 1実装開始時
