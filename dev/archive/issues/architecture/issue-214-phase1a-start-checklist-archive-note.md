# Issue #214 Phase 1a 着手チェックリスト（1週間）

**作成日**: 2026年2月25日  
**対象Issue**: [#214](https://github.com/RedRing2020/RedRing/issues/214)  
**現行正本**: `dev/architecture/CUTTING_SIMULATION_DESIGN.md`

---

## 位置づけ

本書は Phase 1a 着手時点のチェックリストと実装メモを保持する archive note である。  
現行の設計判断・責務境界・実装計画は `dev/architecture/CUTTING_SIMULATION_DESIGN.md` を参照する。

---

## 目的

Issue #214 の初回着手を **Phase 1a（計算コア最小実装）** に限定し、
短いサイクルで検証可能な形に分割する。

---

## 本ブランチで固定する方針（実装判断の前提）

- [x] 3軸固定（工具軸は固定方向）で最小実装を行う
- [x] 工具はフラットエンドミルを対象とする
- [x] 5軸対応は本ブランチのスコープ外とし、別Issueで管理する
- [x] ただし将来拡張を阻害しないよう、型/APIは拡張余地を残す

実装メモ:
- 3軸前提で必要な入力のみを必須化する
- 5軸向け姿勢情報は「予約可能（optional）」な拡張ポイントとして設計上明記する
- 既存利用者に影響する破壊的変更は避ける

---

## スコープ（このチェックリストで扱う範囲）

- `CuttingSimulator` の最小骨格
- `SnapshotInterval`（`ByDistance`, `ByAutoDistance`）
- 距離ベース + 端点オプションのハイブリッド保存ロジック（最小）
- 円柱工具での材料除去接続（既存Octree API活用）
- フラットエンドミル形状（平端）に対応した掃引体除去への切替
- 最小テスト（直線パス、長短セグメント混在）

非スコープ:
- UI連携
- PlaybackController
- メッシュベース削り残し/削り込み判定

---

## Day 0: 着手前ゲート

- [ ] `develop` 最新を取り込み済み
- [ ] 作業ブランチ `feature/issue-214-design` で開始
- [ ] #246 完了前提（ToolSet/Holder定義）を再確認
- [ ] #260 完了前提（`shank_diameter` / `shank_length` 定義）を再確認
- [ ] `cargo test --workspace` の現状グリーンを確認
- [ ] `scripts/check_architecture_dependencies_simple.ps1` が成功

---

## Day 1-2: 構造確定

- [ ] `model/cam_sim` の公開API最小方針を決定
- [ ] `cutting_simulator.rs` と `snapshot.rs` の責務を分離
- [ ] `SnapshotInterval` と `PathPosition` の型定義を確定
- [ ] エラー型（`SimulationError`）の最小セットを定義

成果物:
- [ ] APIスケッチ（Rustシグネチャ）
- [ ] 依存方向確認メモ（循環依存なし）

### Day 1-2 具体化: APIシグネチャ草案（Phase 1a最小）

```rust
// model/cam_sim/src/snapshot.rs

#[derive(Debug, Clone)]
pub enum SnapshotInterval {
  ByDistance {
    interval_mm: f64,
    include_segment_endpoints: bool,
  },
  ByAutoDistance {
    target_count: usize,
    min_interval_mm: f64,
    include_segment_endpoints: bool,
  },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathPosition {
  pub segment_index: usize,
  pub t: f64,
  pub accumulated_distance_mm: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SnapshotConfig {
  pub actual_interval_mm: f64,
  pub estimated_count: usize,
  pub include_endpoints: bool,
}
```

```rust
// model/cam_sim/src/cutting_simulator.rs

pub struct CuttingSimulator<T: Scalar> {
  octree: VoxelOctree<T>,
  snapshots: Vec<VoxelSnapshot<T>>,
  snapshot_positions: Vec<PathPosition>,
  interval: SnapshotInterval,
}

impl<T: Scalar> CuttingSimulator<T> {
  pub fn new(octree: VoxelOctree<T>, interval: SnapshotInterval) -> Self;

  pub fn simulate(
    &mut self,
    toolpath: &ToolPath<T>,
    tool: &Tool,
  ) -> Result<(), SimulationError>;

  pub fn snapshots(&self) -> &[VoxelSnapshot<T>];
  pub fn snapshot_positions(&self) -> &[PathPosition];

  fn calculate_snapshot_config(
    &self,
    toolpath: &ToolPath<T>,
  ) -> Result<SnapshotConfig, SimulationError>;

  fn save_snapshot(&mut self, pos: PathPosition);
}
```

```rust
// model/cam_sim/src/error.rs（必要最小）

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
  #[error("empty toolpath")]
  EmptyToolpath,
  #[error("invalid snapshot interval: {0}")]
  InvalidInterval(String),
  #[error("material removal failed: {0}")]
  MaterialRemovalFailed(String),
}
```

---

## Day 3-4: ロジック最小実装

- [ ] 距離ベーススナップショット間隔計算を実装
- [ ] セグメント端点オプションの重複回避ロジックを実装
- [ ] 材料除去呼び出し点を1箇所に集約
- [ ] ログ（`tracing`）で設定値と件数を出力

品質観点:
- [ ] 長い直線（例: 100mm）で偏りがない
- [ ] 細分化パス（例: 0.5mm×多数）で過剰保存が抑制される

### Day 3-4 具体化: 実装データフロー

1. `calculate_snapshot_config` で実間隔を確定
2. 累積距離を更新しながら各セグメントを走査
3. 間隔到達点で `PathPosition` を生成
4. 端点オプションON時は端点を候補に含め、重複を抑制
5. 材料除去は `octree.remove_material_*` 呼び出しに集約
6. `save_snapshot` で状態と位置を同時記録

### Phase 1a 追記（2026-02-25）: 除去カーネルの整合

- 現行の `remove_material_capsule` は球状近似（カプセル）であり、フラット端工具の端部形状とは一致しない
- `cam_sim` は線分抽出と進行管理に責務を限定し、形状判定は `geo_algorithms::octree::VoxelOctree` 側で行う
- Phase 1a では `remove_material_swept_cylinder`（半球キャップなしの平端掃引体）を追加し、`cam_sim` から利用する

---

## Day 5: テスト・判定

- [ ] ユニットテスト（最小3ケース）
  - [ ] 固定距離モード
  - [ ] 自動距離モード
  - [ ] 端点含有ON/OFF差分
- [ ] `cargo test -p cam_sim`（または該当クレート）成功
- [ ] `cargo test --workspace` 成功
- [ ] `scripts/check_architecture_dependencies_simple.ps1` 成功

### Day 5 具体化: 最低受け入れ基準

- [ ] 距離100mm直線 + `interval_mm=10` で、保存間隔が概ね10mm刻み（許容差 ±1e-6）
- [ ] 同一総距離で「長セグメント1本」と「短セグメント多数」に対し、スナップショット数差が小さい
- [ ] `include_segment_endpoints=true/false` で件数差分が説明可能
- [ ] 例外系（空経路、不正間隔）で `SimulationError` が返る

---

## 完了判定（Phase 1a 開始完了）

- [ ] Phase 1a の最小機能がコード化されている
- [ ] 主要テストが通過している
- [ ] 依存ルール違反がない
- [ ] 次PRで拡張する境界が明記されている

---

## 次ステップ（引き継ぎ）

1. ボールエンドミル対応（Phase 1b）
2. 動作シミュレーション（Phase 1c）
3. メッシュ比較ベース判定（Phase 2）

---

## 実装進捗メモ（2026-02-25）

- [x] `geo_algorithms::VoxelOctree` に平端掃引円柱除去 `remove_material_swept_cylinder` を追加
- [x] `cam_sim::CuttingSimulator` の切削呼び出しを `remove_material_swept_cylinder` へ切替
- [x] `geo_algorithms` 側でカプセルとの差分比較テストを追加

### 視覚的シミュレーション確認に向けた次段

1. `cam_sim` 実行後に `collect_solid_voxel_bounds()` を利用し、可視化入力へボクセル境界群を渡す