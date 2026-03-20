# Octreeトレランス設計メモ（#206）

**作成日**: 2026年2月21日  
**ステータス**: 実装反映済み（運用中）  
**対象**: `model/geo_algorithms/src/octree/*`

---

## 1. 結論

Octree のトレランスは、幾何判定トレランスや CAM トレランスと**分離**して扱う。

- **Octreeトレランス**: 空間インデックスの安定化（AABB境界・セル分割・探索枝刈り）
- **Geometryトレランス**: 距離/角度/交差判定の数学的許容誤差
- **CAMトレランス**: 加工品質・機械精度・クリアランス

1つの値で統一すると、検索漏れ/過検出/性能悪化のトレードオフが隠れて破綻しやすい。

---

## 2. 役割分担

### 2.1 Octreeトレランス（インデックス層）

主な用途:
- 点データを AABB 化するときの半径（最小厚み）
- `query_region` の境界膨張
- 近傍探索時の枝刈り安全余白
- 分割/統合の閾値ヒステリシス

性質:
- 演算誤差吸収より「探索の安定性」を優先
- やや保守的（少し大きめ）でもよい

### 2.2 Geometryトレランス（幾何演算層）

主な用途:
- 接触判定、共線/平行判定、距離比較
- 交差アルゴリズムの収束判定

性質:
- 数学的一貫性を優先
- 小さすぎると不安定、大きすぎると誤判定

### 2.3 CAMトレランス（工程層）

主な用途:
- 閉曲線判定、機械精度、工具クリアランス
- 加工シミュレーション品質の制約

性質:
- 機械・工程要件に依存
- Geometry/Octree より大きい値を取ることがある

---

## 3. 現在コードへの適用状況

以下は #206 ブランチで適用済み:

1. `geo_algorithms::octree` に `OctreeTolerance<T>` を導入（`src/octree/tolerance.rs`）
2. `Octree::new(...)` と併置で `Octree::with_tolerance(...)` を追加
3. `query_region` / `nearest` / 例・統合テスト側が `OctreeTolerance` を参照
4. 既定値は `ToleranceSettings::<T>::relaxed()` を起点とし、Octree用途へ写像

---

## 4. 変換ルール（推奨）

基準:
- `geo_contracts::ToleranceSettings.distance_tolerance` を基準値 `d_geo` とする
- Octree側は用途係数で派生

推奨初期値:
- `point_aabb_half_extent = 1.0 * d_geo`
- `query_expand = 1.0 * d_geo`
- `nearest_prune_margin = 0.5 * d_geo`

CAM 連携時:
- `d_geo = min(cam.machine_accuracy, cam.closure_tolerance)` を候補
- 用途別に上記係数で Octree 値へ変換

---

## 5. 破綻を防ぐ運用ルール

- マジックナンバー禁止（`0.001` 直書き禁止）
- トレランス値は型名付き設定経由のみ参照
- どの層の値かを API 名で明示（`OctreeTolerance` / `ToleranceSettings` / `CamTolerance`）
- 変換は境界（Adapter）で実施し、アルゴリズム本体に混在させない

---

## 6. テスト戦略

最低限の回帰観点:

1. **単調性**: Octreeトレランス増加で候補数が減らない
2. **一致性**: 逐次/並列で結果が一致
3. **境界安定性**: セル境界上の点で探索漏れが発生しない
4. **変換妥当性**: CAM→Octree 変換後も加工領域の漏れがない

---

## 7. 直近アクション（#206ブランチ）

- [x] `OctreeTolerance<T>` 型の導入
- [x] `Octree::with_tolerance(...)` 実装
- [x] 既存テストのトレランス指定を `OctreeTolerance` ベースへ移行
- [x] `OCTREE_DESIGN.md` から本メモへリンク追加

次アクション（任意）:

- [ ] CAM/Geometry 設定からの明示的 Adapter 実装（`From`/`TryFrom` もしくは専用ビルダ）
- [ ] トレランス係数の性能計測（候補数・探索時間・漏れ率）

---

## 8. 補足

本メモは「分離して管理する」方針を定義するもの。
値の最適化（係数チューニング）は、性能計測とCAM精度要件を見ながら別タスクで確定する。

