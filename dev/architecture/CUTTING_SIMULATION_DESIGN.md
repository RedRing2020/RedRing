# 切削シミュレーション設計書

**作成日**: 2026年2月12日  
**最終更新**: 2026年3月31日  
**ステータス**: 設計・段階実装中  
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

### 現在地（2026年3月31日時点）

- Phase 1a のフラットエンドミル向け切削シミュレーションは `cam_sim` で実装済み
- 最小デモ操作は実装済み
- 最小デモ操作の内訳: `Shift+P` でデモ開始
- 最小デモ操作の内訳: `k` / `j` でコマ送り / 巻き戻し
- 最小デモ操作の内訳: 左上進捗バーの左ドラッグでスクラブ
- 最小デモ操作の内訳: `w` でワイヤー表示 / ソリッド表示切替
- デモ操作マニュアルは [manual/cutting_simulation_demo.md](../../manual/cutting_simulation_demo.md) に整理済み
- 一方で、Issue #214 は「ボールエンドミル対応」と「デモ開始導線の整理」が未完了のため、未クローズとする

### 残件再整理

Issue #214 の残件は、以降は次の2系列で管理する。

#### 系列A: デモ操作改善

- A1. デモ開始導線の明確化
- A1-1. アプリ起動直後に切削シミュレーションデモの開始方法を把握できるようにする
- A1-2. `Shift+P` がデモ開始であることを起動時ログまたはヘルプ表示で明示する
- A1-3. ショートカット未記憶でも開始できる最低限の導線を用意する
- A2. デモ操作語彙の統一
- A2-1. マニュアル、ヘルプログ、実装ログを同じ操作語彙に揃える
- A2-2. `ワイヤー表示` / `ソリッド表示` / `スクラブ` / `コマ送り` の表記を固定する
- A2-3. `p` と `Shift+P` の役割差を明示して誤操作を減らす
- A3. 表示状態の可観測性向上
- A3-1. 現在がワイヤー表示かソリッド表示かを判別しやすくする
- A3-2. 現在フレーム番号と総フレーム数を常に把握できる状態を維持する
- A3-3. スクラブ可能領域が画面上で認識しやすいことを確認する
- A4. デモ操作の検証整備
- A4-1. デモ開始後に最低限確認すべき観点をマニュアルへ固定する
- A4-2. `Shift+P` -> `k/j` -> `w` -> スクラブ、の一連操作で破綻しないことを確認する
- A4-3. ワイヤー表示 / ソリッド表示切替後も同一フレームを維持できることを確認する
- A5. 後続拡張の切り分け
- A5-1. 自動再生は別タスクとして切り離し、現デモの必須要件から除外する
- A5-2. 再生速度変更は別タスクとして切り離し、現デモの必須要件から除外する
- A5-3. ボールエンドミル対応と操作改善を別レビュー単位に維持する

##### A1 実装タスク分解

- A1-T1. 起動時ログの整備
- A1-T1a. アプリ起動直後に `Shift+P` で切削シミュレーションデモを開始できることをログ出力する
- A1-T1b. `p` は ToolPath のみ、`Shift+P` は切削シミュレーション全体、という差分をログで明示する
- A1-T1c. 対象ファイル: `view/app/src/app.rs`, `view/app/src/app_state.rs`, `view/app/src/app_state/input_actions.rs`

- A1-T2. ヘルプ表示の整理
- A1-T2a. `h` で表示されるヘルプ内で、切削シミュレーションの開始手順を独立したまとまりとして出す
- A1-T2b. `Shift+P` -> `k/j` -> `w` -> 左上進捗バー、の利用順を読める文面へ整理する
- A1-T2c. 対象ファイル: `view/app/src/app_state/input_actions.rs`

- A1-T3. 画面内導線の追加
- A1-T3a. デモ未開始時だけ表示される簡易案内を画面上に載せるかを判断する
- A1-T3b. 既存の snapshot overlay を拡張するか、別 overlay を追加するかを決める
- A1-T3c. 最小方針は「デモ未開始時のみ `Shift+P: 切削シミュレーション` を表示し、開始後は消す」とする
- A1-T3d. 対象ファイル候補: `view/app/src/snapshot_overlay_renderer.rs`, `view/app/src/app_renderer.rs`, `view/app/src/app_state/stage_orchestration.rs`

- A1-T4. ウィンドウタイトルの扱い整理
- A1-T4a. デモ未開始時タイトルと、デモ開始後タイトルの責務を分ける
- A1-T4b. デモ開始後は現在の `Snapshot x/N` 表示を維持し、未開始時は開始方法を示せるか検討する
- A1-T4c. 対象ファイル: `view/app/src/app_state/snapshot_playback.rs`, `view/app/src/app_state.rs`

- A1-T5. 動作確認手順の固定
- A1-T5a. 起動直後にログまたは画面から `Shift+P` が分かることを確認する
- A1-T5b. `Shift+P` 実行後に進捗バー、タイトル、初期フレーム表示が揃うことを確認する
- A1-T5c. `p` と `Shift+P` の挙動差が誤認されないことを確認する
- A1-T5d. 対象ファイル: `manual/cutting_simulation_demo.md`, `view/app/src/app_state/debug_scene/toolpath.rs`

##### A1 実装順の推奨

- Step 1. `A1-T1` と `A1-T2` を先に実施し、起動直後にログだけで辿れる状態を作る
- Step 2. その後 `A1-T4` を調整し、タイトルの責務を明確化する
- Step 3. なお導線不足が残る場合のみ `A1-T3` の画面内案内を追加する
- Step 4. 最後に `A1-T5` でマニュアルと実装の一致を確認する

#### 系列B: 工具形状拡張

- B1. ボールエンドミル除去の `cam_sim` 統合
- B2. ボールエンドミル用サンプルデータとデモ確認手順の追加
- B3. フラットエンドミルとの差分検証

#### 補足: フラットエンドミル円弧パス除去改善の実現性

- 実現は可能
- ただし、現状の `cam_sim` は `PathGeometry::Arc` を `UnsupportedGeometry` として拒否している
- 一方で `geo_algorithms::octree::voxel` には `remove_material_arc_polyline()` があり、円弧を線分群へ分割して除去する近似 API は既に存在する
- したがって最小実装としては、`cam_sim` 側で Arc セグメントを受け付け、円弧長ベースで分割数を決めて `remove_material_arc_polyline()` へ接続する方針が最短である
- ご提示の「円柱と平面を同心円の円弧、端部を工具系の円弧としたループ面で除去する」方式は、フラットエンドミルの円弧掃引体をより幾何学的に近く表現する拡張として考えられる
- この方式は線分近似より高精度化が見込めるが、Voxel との交差判定に専用の掃引体判定を追加する必要があり、実装コストは高い
- 優先順位としては以下を推奨する
- 優先1: Arc セグメントを polyline 近似で `cam_sim` に統合する
- 優先2: 必要精度を測定し、過剰除去が問題になる場合だけ専用の円弧掃引体を検討する
- 優先3: 専用掃引体を導入する場合は、円柱側面、端面円板、始終端の工具輪郭をどう閉じるかを先に数式と判定条件へ落とす

自動再生・一時停止・再生速度変更は、現時点では必須残件に含めない。現在のデモは手動コマ送り前提で成立しており、優先度は起動導線整理とボールエンドミル対応より下位とする。

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

- [ ] `cam_sim` で `BallEndMill` を受け付ける
- [ ] ボール先端を考慮した除去形状を `simulate()` に統合する
- [ ] ボールエンドミル用のサンプルデータをデモへ追加する

#### Phase 1b 詳細化（2026年3月31日）

##### 既存実装との比較

- Phase 1a はフラットエンドミル限定で、`CuttingSimulator::simulate()` が `FlatEndMill` 以外を拒否している
- Phase 1a の除去本体は `simulate_segments_with_flags()` 内の `remove_material_swept_cylinder()` 呼び出しであり、平底工具の掃引体を前提にしている
- `cam_core::Tool` には `BallEndMill` と `ToolReferencePoint` / `convert_z()` が既に定義されている
- `VoxelOctree` には `remove_material_capsule()` があり、球を線分に沿って掃引した近似除去に利用できる
- 未実装なのは、工具種別に応じた分岐、参照点補正、デモデータ差し替え、比較検証、およびデモ画面への加工サマリ表示である

##### 実装選択肢

- 選択肢1: 最小実装として、ボールエンドミルを「球の掃引」として扱い、工具先端経路を球中心経路へ補正して `remove_material_capsule()` に接続する
- 選択肢2: ボール先端に加えて同径シャンク部も含む厳密な掃引形状を導入する

現時点の推奨は選択肢1とする。理由は、既存 API を活用して最小差分で `BallEndMill` を `cam_sim` に統合でき、Phase 1b のレビュー単位を適切な大きさに保てるためである。

##### 実装が必要なファイル

- `model/cam_sim/src/simulator.rs` - `BallEndMill` の受け入れと参照点補正ヘルパー追加
- `model/cam_sim/src/simulator/engine.rs` - 工具種別ごとの除去分岐
- `model/geo_algorithms/src/octree/voxel/tree_impl.rs` - 既存 `remove_material_capsule()` の利用方針確認、必要ならラッパー追加
- `model/geo_algorithms/src/octree/voxel/node_impl.rs` - 新規除去 API が必要な場合のみ更新
- `viewmodel/converter/src/toolpath_converter.rs` - ボールエンドミル用サンプル ToolPath / サンプル工具定義
- `viewmodel/converter/src/cam_sim_visualization_converter.rs` - 工具線オーバーレイと参照点整合の確認
- `view/app/src/app_state/debug_scene/toolpath.rs` - デモ起動時に使用するサンプルの切替点
- `view/app/src/app_state/snapshot_playback.rs` - 画面表示する加工サマリの更新点
- `view/app/src/snapshot_overlay_renderer.rs` - 加工サマリ表示を overlay へ出す場合の描画拡張候補
- `manual/cutting_simulation_demo.md` - 対応工具と表示項目の更新

##### タスク分解

- B1. 工具種別分岐の追加
- B1-1. `FlatEndMill` と `BallEndMill` を `simulate()` で受け付ける
- B1-2. `RadiusEndMill` は引き続き未対応として明示的に拒否する
- B1-3. フラット側の既存動作を回帰させないことを確認する

- B2. 参照点の正規化
- B2-1. 現行デモの ToolPath 座標は実質的に工具先端基準として扱う
- B2-2. `BallEndMill` では `Tool::convert_z(..., Tip, Center)` 相当の補正を用いて、除去計算に渡す中心経路を生成する
- B2-3. 参照点補正はまず Z 軸方向のみを対象とし、5軸の姿勢付き ToolPath への一般化は後続タスクとする

- B3. 除去モデルの統合
- B3-1. フラットエンドミルは従来どおり `remove_material_swept_cylinder()` を使う
- B3-2. ボールエンドミルは中心経路に対して `remove_material_capsule()` を使う
- B3-3. シャンク・ホルダー干渉やラジアスエンドミルの扱いは Phase 1b の対象外とする

- B4. デモデータ整備
- B4-1. ボールエンドミル用のサンプル工具をデモ経路へ接続する
- B4-2. フラットとの差が視認できるよう、少なくとも1本は Z 変化を含むセグメントを持つサンプル経路を用意する
- B4-3. 既存フラットサンプルは比較用として維持する

- B5. 画面表示する加工サマリ
- B5-1. 全体の距離
- B5-2. 切削時間（現時点では固定送り前提の単純方式）
- B5-3. 各軸の Min / Max（X / Y / Z）
- B5-4. 5軸時の回転軸 Min / Max（A / B / C など存在する軸のみ）
- B5-5. 現在フレームの累積時間と残り推定時間
- B5-6. 現在セグメント種別（Cutting / Rapid / Approach / Retract / PassRetract）
- B5-7. 工具接触中フラグ（切削中か空走中か）
- B5-8. 除去体積率（初期体積比の進捗）

- B6. テスト整備
- B6-1. `BallEndMill` が `UnsupportedToolType` にならないこと
- B6-2. 先端基準の水平経路に対し、中心補正後の除去が実行されること
- B6-3. 同一経路・同一半径で、フラットとボールの残存体積差が観測できること
- B6-4. 既存フラット経路のスナップショット件数と進捗表示が回帰しないこと

##### 実装順の推奨

- Step 1. `B1` と `B2` を先に実装し、`cam_sim` 単体で `BallEndMill` を受け付ける状態を作る
- Step 2. 続けて `B3` を実装し、フラット / ボールで除去 API が分岐する状態を作る
- Step 3. その後 `B4` を実施し、比較しやすいデモ経路とサンプル工具を用意する
- Step 4. 次に `B5` を実施し、デモ画面で加工サマリを確認できるようにする
- Step 5. 最後に `B6` と `manual/cutting_simulation_demo.md` 更新を行い、比較観点と制約を固定する

**テスト**:

- ボールエンドミル特有の削り残し検証
- フラットエンドミルとの比較

**補足**:

- `SphericalSolid3D` / `TorusSolid3D` などの幾何プリミティブ自体は別途存在する
- 未完了なのは、それらを切削シミュレーションへ接続する工程

### Phase 1c: 動作シミュレーション（2週間）

**実装ファイル**:

- `view/app/src/app_state/debug_scene/toolpath.rs` - デモ開始
- `view/app/src/app_state/snapshot_playback.rs` - コマ送り / 巻き戻し / スクラブ
- `view/app/src/app_state/display_controls.rs` - ワイヤー表示 / ソリッド表示切替
- `view/app/src/app_state/mouse_actions.rs` - スクラブ入力
- `view/app/src/snapshot_overlay_renderer.rs` - 左上進捗バー描画

**実装項目**:

- [x] コマ送り / 巻き戻し
- [x] スライダーによる任意時点移動
- [x] ワイヤー表示 / ソリッド表示切替
- [x] スナップショット進捗バー表示
- [ ] ショートカット依存を弱めた起動導線の整理
- [ ] 自動再生
- [ ] 再生速度調整（0.1x～10x）
- [ ] スナップショット間の補間（スムーズ再生）
- [ ] 距離計算支援UI（IntervalRecommender）

**UI機能**:

- 左上進捗バー
- ウィンドウタイトルへの現在フレーム表示
- ワイヤー表示 / ソリッド表示切替
- 将来拡張候補: スナップショット間隔設定ダイアログ、推奨値自動計算、メモリ使用量表示

**テスト**:

- コマ送り / 巻き戻しで同一系列を往復できること
- スクラブ時に任意フレームへ移動できること
- ワイヤー表示 / ソリッド表示切替時に同一フレームを維持できること

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
- シャンク段定義（多段、円柱/テーパー）
- シャンク干渉距離（側面/底面）

### Issue #503 反映事項（2026-03）

- `ToolSet` は `ShankSegmentKind (Cylinder/Taper)` と `ShankSegment` を保持
- シャンク専用干渉距離 `ShankInterferenceOffset { side, bottom }` を保持
- シャンク全長が `stickout_length` 以上の場合は無効
- シャンク底面クリアランス（bottom）は以下条件で自動無効化する
    - 最下段がテーパーの場合
    - 工具径 > 最下段シャンク径 の場合

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

### シャンク形状定義

シャンクは上端→下端順の多段形状として定義する。
各段は以下の情報を持つ。

- 種別: 円柱またはテーパー
- 段長さ
- 上端径
- 下端径

### 干渉距離定義

ホルダー干渉距離は用途別に分離して定義する。

- 側面干渉距離（半径方向オフセット）
- 底面干渉距離（軸方向先端側オフセット）

多段ホルダーにおける底面干渉距離は、ホルダー最下面（工具側先端の再底面）のみに適用する。
中間段の段端面には適用しない。

シャンク干渉距離も同様に side / bottom で定義する。
ただしシャンク bottom は、過剰判定と無効領域の発生を防ぐため条件付き適用とする。

### 干渉判定時の形状生成ルール

切削シミュレーションおよびCAM干渉チェックでは、実形状ではなく干渉オフセット形状を用いる。

- 側面: 各段の外径に側面干渉距離を反映する
- 底面: 最下面のみ底面干渉距離を反映する
- 段間R: 側面干渉適用後の連続性を維持する（負値や不連続は禁止）

シャンク bottom の適用条件:

- 最下段が円柱であること
- 工具径 <= 最下段シャンク径

上記を満たさない場合、シャンク bottom は 0 として扱う（自動無効化）。

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
| 2026-03-31 | Issue #503 反映（シャンク多段化、シャンク bottom 条件付き自動無効化） | AI開発者 |
| 2026-03-26 | Issue #260 反映（shank_length 追加、shank属性の検証規約を追記） | AI開発者 |
| 2026-02-22 | Issue #246 ツールセット定義（ホルダー多段、参照点、干渉距離）を追記 | AI開発者 |
| 2026-02-12 | 初版作成（距離ベーススナップショット設計） | AI開発者 |
