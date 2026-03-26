# 切削シミュレーション設計書

**作成日**: 2026年2月12日  
**最終更新**: 2026年2月22日  
**ステータス**: 設計フェーズ  
**関連Issue**: [#214](https://github.com/RedRing2020/RedRing/issues/214), [#246](https://github.com/RedRing2020/RedRing/issues/246)

---

## 目次

1. [概要](#概要)
2. [背景と課題](#背景と課題)
3. [ハイブリッドスナップショット方式](#ハイブリッドスナップショット方式)
4. [距離計算支援機能](#距離計算支援機能)
5. [実装計画](#実装計画)
6. [技術仕様](#技術仕様)
7. [パフォーマンス目標](#パフォーマンス目標)

---

## 概要

VoxelOctreeを用いたCAM工具経路の切削シミュレーション機能を実装します。
本設計では、**距離ベーススナップショット方式**により均一な切削状況の記録を実現します。

### 主要機能

1. **VoxelOctree による材料除去シミュレーション**
   - 円柱工具（フラットエンドミル）
   - ボールエンドミル（半球形状）
   
2. **距離ベーススナップショット**
   - セグメント端点オプション（ハイブリッド方式）
   - 自動距離計算支援（総距離ベース最適化）
   
3. **動作シミュレーション**
   - 再生・巻き戻し
   - スライダーによる任意時点への移動
   - 速度調整可能な再生

4. **切削状況の可視化**
   - 削り残し検出（メッシュベース比較）
   - 送り速度による色分け
   - 工具突き出し長警告表示

---

## 背景と課題

### 従来の問題点（セグメント数ベース）

#### 問題1: 長いセグメントでの情報欠落

```
平面加工の例:
Segment 1: (0, 0, 0) → (100, 0, 0)  // 100mm の直線
Segment 2: (100, 0, 0) → (100, 100, 0)  // 100mm の直線

セグメント端点のみ保存 → 途中99mmの切削状況が不明
```

#### 問題2: 短いセグメントでの過剰記録

```
円弧近似の例:
Segment 1-200: 各0.5mm × 200 = 100mm の円弧

200個すべての端点を保存 → メモリ無駄遣い
```

#### 問題3: 不均一性

```
ToolPath全体:
- 長い直線: 1セグメント = 100mm
- 細かい円弧: 200セグメント = 100mm

同じ長さなのに記録密度が200倍異なる
```

### 解決策: 距離ベーススナップショット

**等距離間隔**で切削状況を記録することで、移動距離に対して**均一な**情報密度を実現します。

---

## ハイブリッドスナップショット方式

### 設計方針

距離ベースの均一記録に、セグメント端点の確実な記録を組み合わせます。

### データ構造

```rust
/// スナップショット保存間隔の指定方式
#[derive(Debug, Clone)]
pub enum SnapshotInterval {
    /// 固定距離間隔（mm）+ オプション設定
    ByDistance {
        /// 基本間隔距離（mm）
        interval_mm: f64,
        
        /// セグメント端点を必ず含めるか
        include_segment_endpoints: bool,
    },
    
    /// 自動等分割（総距離から計算）
    ByAutoDistance {
        /// 目標スナップショット数
        target_count: usize,
        
        /// 最小間隔距離（mm、これ以下にはしない）
        min_interval_mm: f64,
        
        /// セグメント端点を必ず含めるか
        include_segment_endpoints: bool,
    },
    
    /// セグメント数ベース（後方互換性）
    #[deprecated(note = "距離ベース方式の使用を推奨")]
    BySegmentCount(usize),
}

impl Default for SnapshotInterval {
    fn default() -> Self {
        Self::ByDistance {
            interval_mm: 10.0,
            include_segment_endpoints: true,
        }
    }
}
```

### アルゴリズム: ハイブリッド方式

```rust
pub struct CuttingSimulator<T: Scalar> {
    octree: VoxelOctree<T>,
    snapshots: Vec<VoxelSnapshot<T>>,
    snapshot_positions: Vec<PathPosition>,
    interval: SnapshotInterval,
}

#[derive(Debug, Clone)]
pub struct PathPosition {
    /// セグメントインデックス
    segment_index: usize,
    
    /// セグメント内の補間パラメータ (0.0 = 開始点, 1.0 = 終了点)
    t: f64,
    
    /// ToolPath開始からの累積距離（mm）
    accumulated_distance: f64,
}

impl<T: Scalar> CuttingSimulator<T> {
    /// 工具経路をシミュレーション実行
    pub fn simulate(&mut self, toolpath: &ToolPath, tool: &Tool) -> Result<(), SimulationError> {
        let config = self.calculate_snapshot_config(toolpath)?;
        
        tracing::info!(
            "Snapshot設定: 間隔 {:.2}mm, 予測数: {}",
            config.actual_interval_mm,
            config.estimated_count
        );
        
        let mut accumulated_distance = 0.0;
        let mut next_snapshot_distance = 0.0;
        
        // セグメント端点をセットに登録（重複防止）
        let mut endpoint_distances = HashSet::new();
        if config.include_endpoints {
            let mut dist = 0.0;
            for segment in toolpath.all_segments() {
                endpoint_distances.insert(OrderedFloat(dist));
                dist += segment.length();
                endpoint_distances.insert(OrderedFloat(dist));
            }
        }
        
        for (seg_idx, segment) in toolpath.all_segments().iter().enumerate() {
            let segment_length = segment.length();
            let segment_start_dist = accumulated_distance;
            
            // セグメント開始点でスナップショット（端点モード時）
            if config.include_endpoints && accumulated_distance == next_snapshot_distance {
                self.save_snapshot(PathPosition {
                    segment_index: seg_idx,
                    t: 0.0,
                    accumulated_distance,
                });
                next_snapshot_distance += config.actual_interval_mm;
            }
            
            // 材料除去
            self.octree.remove_material(&segment.to_swept_solid(tool))?;
            accumulated_distance += segment_length;
            
            // セグメント内部の距離ベーススナップショット
            while next_snapshot_distance < accumulated_distance {
                let dist_in_segment = next_snapshot_distance - segment_start_dist;
                let t = dist_in_segment / segment_length;
                
                // 端点モード時は端点と重複しないか確認
                let is_duplicate = config.include_endpoints 
                    && endpoint_distances.contains(&OrderedFloat(next_snapshot_distance));
                
                if !is_duplicate {
                    self.save_snapshot(PathPosition {
                        segment_index: seg_idx,
                        t,
                        accumulated_distance: next_snapshot_distance,
                    });
                }
                
                next_snapshot_distance += config.actual_interval_mm;
            }
            
            // セグメント終了点でスナップショット（端点モード時）
            if config.include_endpoints {
                let is_at_interval = (accumulated_distance - next_snapshot_distance + config.actual_interval_mm).abs() < 1e-6;
                
                if !is_at_interval {
                    self.save_snapshot(PathPosition {
                        segment_index: seg_idx,
                        t: 1.0,
                        accumulated_distance,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// スナップショット設定を計算
    fn calculate_snapshot_config(&self, toolpath: &ToolPath) -> Result<SnapshotConfig, SimulationError> {
        match &self.interval {
            SnapshotInterval::ByDistance { interval_mm, include_segment_endpoints } => {
                let total_distance = toolpath.total_length();
                let estimated_count = (total_distance / interval_mm).ceil() as usize;
                
                Ok(SnapshotConfig {
                    actual_interval_mm: *interval_mm,
                    estimated_count,
                    include_endpoints: *include_segment_endpoints,
                })
            }
            
            SnapshotInterval::ByAutoDistance { target_count, min_interval_mm, include_segment_endpoints } => {
                let total_distance = toolpath.total_length();
                let ideal_interval = total_distance / (*target_count as f64);
                let actual_interval = ideal_interval.max(*min_interval_mm);
                let estimated_count = (total_distance / actual_interval).ceil() as usize;
                
                tracing::info!(
                    "自動計算: 総距離 {:.2}mm ÷ 目標{}個 = {:.2}mm → 最小制約適用後 {:.2}mm",
                    total_distance, target_count, ideal_interval, actual_interval
                );
                
                Ok(SnapshotConfig {
                    actual_interval_mm: actual_interval,
                    estimated_count,
                    include_endpoints: *include_segment_endpoints,
                })
            }
            
            #[allow(deprecated)]
            SnapshotInterval::BySegmentCount(count) => {
                let total_segments = toolpath.segment_count();
                let total_distance = toolpath.total_length();
                let interval = total_distance / (*count as f64);
                
                tracing::warn!(
                    "非推奨: セグメント数ベース使用中。距離ベース方式への移行を推奨。"
                );
                
                Ok(SnapshotConfig {
                    actual_interval_mm: interval,
                    estimated_count: *count,
                    include_endpoints: false,
                })
            }
        }
    }
    
    fn save_snapshot(&mut self, position: PathPosition) {
        let snapshot = self.octree.create_snapshot();
        self.snapshots.push(snapshot);
        self.snapshot_positions.push(position);
        
        tracing::debug!(
            "Snapshot #{}: セグメント {}、t={:.3}、累積距離={:.2}mm",
            self.snapshots.len(),
            position.segment_index,
            position.t,
            position.accumulated_distance
        );
    }
}

#[derive(Debug, Clone)]
struct SnapshotConfig {
    actual_interval_mm: f64,
    estimated_count: usize,
    include_endpoints: bool,
}

/// 順序付き浮動小数点（HashSet用）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OrderedFloat(u64);

impl OrderedFloat {
    fn new(f: f64) -> Self {
        Self(f.to_bits())
    }
}
```

### 使用例

```rust
// 例1: 固定10mm間隔、セグメント端点も含める（推奨）
let simulator = CuttingSimulator::new(
    workpiece_bbox,
    octree_depth,
    SnapshotInterval::ByDistance {
        interval_mm: 10.0,
        include_segment_endpoints: true,
    }
);

// 例2: 自動計算（100個のスナップショット目標）
let simulator = CuttingSimulator::new(
    workpiece_bbox,
    octree_depth,
    SnapshotInterval::ByAutoDistance {
        target_count: 100,
        min_interval_mm: 5.0,  // 最低5mm間隔
        include_segment_endpoints: true,
    }
);

// 例3: 距離のみ、端点は含めない（高速）
let simulator = CuttingSimulator::new(
    workpiece_bbox,
    octree_depth,
    SnapshotInterval::ByDistance {
        interval_mm: 10.0,
        include_segment_endpoints: false,
    }
);
```

---

## 距離計算支援機能

### UI設計

ユーザーが最適な間隔を設定できるよう、自動計算と編集機能を提供します。

```rust
/// 距離間隔推奨値の計算
pub struct IntervalRecommender;

impl IntervalRecommender {
    /// 総距離から推奨間隔を計算
    pub fn recommend(total_distance_mm: f64) -> IntervalRecommendation {
        let recommendations = vec![
            // (目標スナップショット数, 説明)
            (50, "粗い（高速、メモリ節約）"),
            (100, "標準（推奨）"),
            (200, "詳細（より正確）"),
            (500, "非常に詳細（重い）"),
        ];
        
        let options = recommendations.iter().map(|(count, desc)| {
            let interval = total_distance_mm / (*count as f64);
            
            IntervalOption {
                interval_mm: interval,
                target_count: *count,
                description: desc.to_string(),
                memory_estimate_mb: Self::estimate_memory(*count),
            }
        }).collect();
        
        IntervalRecommendation {
            total_distance_mm,
            options,
            custom_interval_mm: None,
        }
    }
    
    /// メモリ使用量の概算（MB）
    fn estimate_memory(snapshot_count: usize) -> f64 {
        const BYTES_PER_SNAPSHOT: usize = 1024 * 1024; // 1MB（深さ8の場合の概算）
        (snapshot_count * BYTES_PER_SNAPSHOT) as f64 / (1024.0 * 1024.0)
    }
    
    /// カスタム間隔の妥当性チェック
    pub fn validate_custom_interval(
        interval_mm: f64,
        total_distance_mm: f64
    ) -> Result<ValidationResult, IntervalError> {
        if interval_mm <= 0.0 {
            return Err(IntervalError::NonPositiveInterval);
        }
        
        if interval_mm < 1.0 {
            return Ok(ValidationResult::Warning(
                "1mm未満の間隔は非常に詳細でメモリを大量消費します。"
            ));
        }
        
        let snapshot_count = (total_distance_mm / interval_mm).ceil() as usize;
        let memory_mb = Self::estimate_memory(snapshot_count);
        
        if memory_mb > 2048.0 {
            // 2GB以上
            return Ok(ValidationResult::Warning(
                format!("メモリ使用量が推定 {:.0}MB と大きくなります。", memory_mb)
            ));
        }
        
        Ok(ValidationResult::Ok {
            snapshot_count,
            memory_mb,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IntervalRecommendation {
    pub total_distance_mm: f64,
    pub options: Vec<IntervalOption>,
    pub custom_interval_mm: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct IntervalOption {
    pub interval_mm: f64,
    pub target_count: usize,
    pub description: String,
    pub memory_estimate_mb: f64,
}

#[derive(Debug, Clone)]
pub enum ValidationResult {
    Ok {
        snapshot_count: usize,
        memory_mb: f64,
    },
    Warning(String),
}

#[derive(Debug, Clone)]
pub enum IntervalError {
    NonPositiveInterval,
}
```

### UI例（疑似コード）

```rust
// App層での使用例
impl AppState {
    fn show_simulation_settings(&mut self, ui: &mut Ui, toolpath: &ToolPath) {
        let total_distance = toolpath.total_length();
        let recommendation = IntervalRecommender::recommend(total_distance);
        
        ui.label(format!("工具経路総距離: {:.2} mm", total_distance));
        ui.separator();
        
        // 推奨オプション
        ui.label("推奨間隔:");
        for option in &recommendation.options {
            if ui.button(format!(
                "{}: {:.2}mm間隔（約{}個、~{:.0}MB）",
                option.description,
                option.interval_mm,
                option.target_count,
                option.memory_estimate_mb
            )).clicked() {
                self.simulation_interval = option.interval_mm;
            }
        }
        
        ui.separator();
        
        // カスタム入力
        ui.label("カスタム間隔:");
        ui.add(egui::Slider::new(&mut self.simulation_interval, 1.0..=100.0)
            .text("mm")
            .step_by(1.0));
        
        // 妥当性チェック結果表示
        match IntervalRecommender::validate_custom_interval(self.simulation_interval, total_distance) {
            Ok(ValidationResult::Ok { snapshot_count, memory_mb }) => {
                ui.label(format!(
                    "✓ 約{}個のスナップショット、メモリ ~{:.0}MB",
                    snapshot_count, memory_mb
                ));
            }
            Ok(ValidationResult::Warning(msg)) => {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {}", msg));
            }
            Err(e) => {
                ui.colored_label(egui::Color32::RED, format!("✗ {:?}", e));
            }
        }
        
        // セグメント端点オプション
        ui.checkbox(&mut self.include_segment_endpoints, "セグメント端点も含める（推奨）");
        
        if ui.button("シミュレーション開始").clicked() {
            self.start_cutting_simulation();
        }
    }
}
```

---

## 実装計画

### 前提条件

- ✅ Octree基本実装完了（Issue #207）
- ✅ VoxelOctree実装完了
- ⬜ 本Issue作成（Octree完了後）

### Phase 1a: VoxelOctree + 円柱工具除去（1週間）

**実装ファイル**:
- `model/geo_algorithms/src/octree/voxel.rs` - VoxelOctree
- `model/geo_algorithms/src/octree/cutting_simulator.rs` - シミュレータ本体

**実装項目**:
- [x] `CuttingSimulator` 構造体
- [x] `SnapshotInterval` 列挙型
- [x] `simulate()` - ハイブリッド方式実装
- [x] `PathPosition` - 位置記録
- [x] 円柱工具による材料除去

**テスト**:
- 単純直線パスでの距離ベース記録確認
- セグメント端点の確実な記録確認
- 長短セグメント混在時の均一性確認

### Phase 1b: ボールエンドミル対応（1週間）

**実装項目**:
- [x] `SphericalSolid3D` - 球形状（半球除去用）
- [x] `TorusSolid3D` - トーラス形状（角のフィレット用）
- [x] ツール形状の統一インターフェース

**テスト**:
- ボールエンドミル特有の削り残し検証
- フラットエンドミルとの比較

### Phase 1c: 動作シミュレーション（2週間）

**実装ファイル**:
- `model/geo_algorithms/src/octree/playback.rs` - 再生制御
- `viewmodel/graphics/src/cutting_simulator_view.rs` - ViewModel変換
- `view/stage/src/cutting_simulation_stage.rs` - レンダリング

**実装項目**:
- [x] `PlaybackController` - 再生/一時停止/巻き戻し
- [x] スライダーによる任意時点移動
- [x] 速度調整（0.1x～10x）
- [x] スナップショット間の補間（スムーズ再生）
- [x] 距離計算支援UI（IntervalRecommender）

**UI機能**:
- スナップショット間隔設定ダイアログ
- 推奨値の自動計算・表示
- カスタム値の妥当性チェック
- メモリ使用量の予測表示

**テスト**:
- 再生速度の正確性
- スナップショット間補間の滑らかさ
- メモリ使用量の実測

### Phase 2: メッシュベース削り込み判定（2週間）

**実装ファイル**:
- `model/geo_algorithms/src/mesh_comparison.rs` - メッシュ比較
- `model/geo_io/src/nurbs_to_mesh.rs` - NURBS→メッシュ変換

**実装項目**:
- [x] NURBS→三角形メッシュ変換（CAMトレランスベース）
- [x] メッシュとVoxelOctreeの距離計算
- [x] 削り残し領域の検出
- [x] 削り込み領域の検出

**テスト**:
- 単純形状での精度検証
- 複雑NURBS形状での性能測定
- トレランス設定による精度とパフォーマンスのトレードオフ確認

### Phase 3: 色分け可視化（3日）

**実装項目**:
- [x] 送り速度による色分け（青→緑→黄→赤）
- [x] 工具突き出し長による警告表示（オレンジ点滅）
- [x] 削り残し/削り込みの色分け表示

**カラーマップ**:
```rust
送り速度:
  0-25%:   青 (0.0, 0.5, 1.0)    // 超低速
  25-50%:  緑 (0.0, 1.0, 0.0)    // 標準
  50-75%:  黄 (1.0, 1.0, 0.0)    // 高速
  75-100%: 赤 (1.0, 0.0, 0.0)    // 急送

工具突き出し長:
  安全範囲:   グレー (0.7, 0.7, 0.7)
  警告範囲:   オレンジ (1.0, 0.5, 0.0)
  危険範囲:   赤点滅 (1.0, 0.0, 0.0)

削り込み状況:
  削り残し:   マゼンタ (1.0, 0.0, 1.0)
  適正:       透明
  削り込み:   シアン (0.0, 1.0, 1.0)
```

---

## 技術仕様

### セグメント長の計算

```rust
impl PathSegment {
    /// セグメントの実距離を計算
    pub fn length(&self) -> f64 {
        match self.segment_type {
            SegmentType::Linear => {
                // 直線: ユークリッド距離
                (self.end - self.start).norm()
            }
            
            SegmentType::Arc { center, radius, .. } => {
                // 円弧: 半径 × 中心角
                let start_vec = self.start - center;
                let end_vec = self.end - center;
                let angle = start_vec.angle_to(&end_vec);
                radius * angle.abs()
            }
            
            SegmentType::Helix { pitch, revolutions, radius } => {
                // 螺旋: √(円周長² + ピッチ²)
                let circumference = 2.0 * std::f64::consts::PI * radius * revolutions;
                let height = pitch * revolutions;
                (circumference.powi(2) + height.powi(2)).sqrt()
            }
            
            // ... 他のセグメントタイプ
        }
    }
    
    /// セグメント上の補間点を計算
    pub fn interpolate(&self, t: f64) -> Point3D<f64> {
        debug_assert!((0.0..=1.0).contains(&t));
        
        match self.segment_type {
            SegmentType::Linear => {
                Point3D::lerp(&self.start, &self.end, t)
            }
            
            SegmentType::Arc { center, start_angle, end_angle, .. } => {
                let angle = start_angle + t * (end_angle - start_angle);
                center + Vector3D::from_polar(radius, angle)
            }
            
            // ... 他のセグメントタイプ
        }
    }
}
```

### VoxelSnapshot構造

```rust
/// Octree状態のスナップショット
#[derive(Clone)]
pub struct VoxelSnapshot<T: Scalar> {
    /// ノード状態の圧縮表現
    compressed_state: CompressedOctree,
    
    /// 残存体積（高速計算用キャッシュ）
    remaining_volume: T,
    
    /// タイムスタンプ（ミリ秒）
    timestamp_ms: u64,
}

/// Octree圧縮表現（メモリ節約）
#[derive(Clone)]
struct CompressedOctree {
    /// ノード状態のビット配列（Empty=0, Solid=1, Mixed=2）
    node_states: BitVec,
    
    /// 深さごとのノード数
    depth_counts: Vec<usize>,
}

impl<T: Scalar> VoxelSnapshot<T> {
    /// スナップショットのメモリサイズを取得
    pub fn memory_size(&self) -> usize {
        self.compressed_state.node_states.len() / 8  // バイト単位
            + std::mem::size_of::<T>()
            + std::mem::size_of::<u64>()
    }
}
```

### ToolPath拡張メソッド

```rust
impl ToolPath {
    /// 全セグメントをフラット化
    pub fn all_segments(&self) -> Vec<&PathSegment> {
        let mut segments = Vec::new();
        
        // Approach
        for path in &self.approach {
            segments.extend(&path.segments);
        }
        
        // Cutting（全周回）
        for level in &self.cutting {
            for path in &level.paths {
                segments.extend(&path.segments);
            }
        }
        
        // Retract
        for path in &self.retract {
            segments.extend(&path.segments);
        }
        
        segments
    }
    
    /// 総距離を計算
    pub fn total_length(&self) -> f64 {
        self.all_segments().iter().map(|seg| seg.length()).sum()
    }
    
    /// セグメント総数
    pub fn segment_count(&self) -> usize {
        self.all_segments().len()
    }
}
```

---

## パフォーマンス目標

### メモリ使用量

| 設定 | スナップショット数 | メモリ使用量 | 用途 |
|------|-------------------|-------------|------|
| 粗い | 50 | ~50MB | プレビュー、高速確認 |
| 標準 | 100 | ~100MB | 一般的な加工（推奨） |
| 詳細 | 200 | ~200MB | 精密加工、品質重視 |
| 非常に詳細 | 500 | ~500MB | 超精密加工、研究用 |

**計算根拠**:
- Octree深さ8の場合、1スナップショット ≈ 1MB
- 総距離1000mmの場合:
  - 10mm間隔 → 100スナップショット → ~100MB
  - 5mm間隔 → 200スナップショット → ~200MB

### 処理時間目標

| Phase | 処理内容 | 目標時間 | 備考 |
|-------|---------|---------|------|
| Phase 1a | VoxelOctree構築 + 材料除去 | < 1秒 | 1000セグメント |
| Phase 1b | ボールエンドミル除去 | < 2秒 | 複雑判定のため遅い |
| Phase 1c | スナップショット保存 | < 0.1秒 | 1スナップショットあたり |
| Phase 2 | メッシュ比較 | < 5秒 | 10000三角形メッシュ |
| Phase 3 | 色分け可視化 | リアルタイム | 60FPS維持 |

### スケーラビリティ

```
小規模加工:
  総距離: 100mm
  セグメント: 50個
  スナップショット: 10個（10mm間隔）
  メモリ: ~10MB
  処理時間: <0.5秒

中規模加工（標準）:
  総距離: 1000mm
  セグメント: 500個
  スナップショット: 100個（10mm間隔）
  メモリ: ~100MB
  処理時間: <1秒

大規模加工:
  総距離: 10000mm
  セグメント: 5000個
  スナップショット: 1000個（10mm間隔）
  メモリ: ~1GB
  処理時間: <10秒

超大規模加工（5軸複雑パス）:
  総距離: 100000mm
  セグメント: 50000個
  スナップショット: 5000個（20mm間隔）
  メモリ: ~5GB
  処理時間: <60秒
```

---

## 関連ドキュメント

- [OCTREE_DESIGN.md](./OCTREE_DESIGN.md) - Octree基本設計
- [CAM_VISUALIZATION_REQUIREMENTS.md](./CAM_VISUALIZATION_REQUIREMENTS.md) - CAM可視化要求
- [CAM_CRATE_DESIGN_PROPOSAL.md](./CAM_CRATE_DESIGN_PROPOSAL.md) - cam_core設計
- [ISSUE_214_IMPLEMENTATION_PREP.md](./ISSUE_214_IMPLEMENTATION_PREP.md) - #214着手準備チェックリスト

---

## Issue #246: ツールセット定義（工具 + ホルダー）

関連Issue: [#246](https://github.com/RedRing2020/RedRing/issues/246), [#260](https://github.com/RedRing2020/RedRing/issues/260)

### 目的

CAM計算および切削シミュレーションで使用するツールセット（工具 + ホルダー）の定義を統一し、
干渉判定の計算条件を明確化する。

### ツールセット構成要素

- 工具（Tool）
- ホルダー（Holder）
- 参照点定義（Tool Reference Point）
- 干渉距離定義（Interference Offset）

### 共通属性（最低限）

工具およびツールセットは、以下の共通属性を最低限保持する。

- 工具径
- 刃長
- 全長
- 突き出し長
- シャンク径
- シャンク長（`0` は未指定）

### Issue #260 反映事項（2026-03）

- `ToolSet` に `shank_diameter` / `shank_length` を保持
- `shank_diameter` / `shank_length` は `0` を未指定として扱う
- `shank_length > 0` かつ `shank_length >= stickout_length` は無効
- 本時点では `ToolSet` の専用シリアライズ境界（読込/書込/round-trip）は未導入

### 用語定義

#### 工具原点定義（Tool Origin）

工具・ホルダー形状を配置するための座標系基準点。
形状定義および姿勢計算の基準として使用する。

#### 参照点定義（Tool Reference Point）

ツールパス上の座標値が工具上のどの点を指すかを示す定義（例: Tip / Center / Gauge）。

#### 用語運用ルール

参照点定義と工具原点定義は役割が異なるため、独立した定義として扱う。

### ホルダー形状定義

ホルダーは軸方向に連続する多段形状として定義する。
各段は以下の情報を持つ。

- 種別: 円柱またはテーパー
- 段長さ
- 上端径
- 下端径
- 段間R（必要に応じて定義）

### 干渉距離定義

ホルダー干渉距離は用途別に分離して定義する。

- 側面干渉距離（半径方向オフセット）
- 底面干渉距離（軸方向先端側オフセット）

多段ホルダーにおける底面干渉距離は、ホルダー最下面（工具側先端の再底面）のみに適用する。
中間段の段端面には適用しない。

### 干渉判定時の形状生成ルール

切削シミュレーションおよびCAM干渉チェックでは、実形状ではなく干渉オフセット形状を用いる。

- 側面: 各段の外径に側面干渉距離を反映する
- 底面: 最下面のみ底面干渉距離を反映する
- 段間R: 側面干渉適用後の連続性を維持する（負値や不連続は禁止）

### 最小バリデーション要件

以下を満たさない定義は無効とする。

- 段長さ > 0
- 上端径 > 0、下端径 > 0
- 干渉距離 >= 0
- 段接続が軸方向に連続
- 最下面が一意に定まること

### 設定例

- 側面干渉距離: 10.0 mm
- 底面干渉距離: 0.0 mm

上記設定時、側面は10.0 mm拡張し、底面は拡張しない。

---

## 変更履歴

| 日付 | 変更内容 | 担当 |
|------|---------|------|
| 2026-03-26 | Issue #260 反映（shank_length 追加、ToolSet I/O境界未導入を明記） | AI開発者 |
| 2026-02-22 | Issue #246 ツールセット定義（ホルダー多段、参照点、干渉距離）を追記 | AI開発者 |
| 2026-02-12 | 初版作成（距離ベーススナップショット設計） | AI開発者 |
