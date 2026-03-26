# Issue #214 実装準備チェックリスト

**作成日**: 2026年2月22日  
**対象Issue**: [#214](https://github.com/RedRing2020/RedRing/issues/214)  
**前提Issue**: [#246](https://github.com/RedRing2020/RedRing/issues/246)（完了）、[#260](https://github.com/RedRing2020/RedRing/issues/260)（ToolSet shank属性補完）

---

## 1. 着手ゲート確認

- [x] #246 ツールセット定義が実装済み（`ToolSet` / `Holder` / 干渉距離）
- [x] #260 shank 属性が実装済み（`shank_diameter` / `shank_length`）
- [x] #246 の定義がドキュメント化済み
- [x] #214 側に #246 前提コメント連携済み
- [x] `develop` ブランチにマージ済み

**判定**: Issue #214 実装着手可

---

## 2. 実装スコープ（今回の最初の着手対象）

Issue #214 の全体は Phase 1a〜3 だが、最初の実装PRでは **Phase 1a** に限定する。

### Phase 1a（最初のPR）

- [ ] `CuttingSimulator` 構造体の最小実装
- [ ] `SnapshotInterval` 定義（`ByDistance`, `ByAutoDistance`）
- [ ] ハイブリッド保存ロジックの骨格
- [ ] 円柱工具での材料除去（既存Octree APIに接続）
- [ ] 最小テスト（直線パス、長短セグメント混在）

---

## 3. 実装配置（cam_sim分離方針）

`CuttingSimulator` は CAM ドメイン責務のため、`geo_algorithms` 直下ではなく
新規 `cam_sim` クレートへ配置する。

- `model/cam_sim/src/lib.rs`
- `model/cam_sim/src/cutting_simulator.rs`
- `model/cam_sim/src/snapshot.rs`（必要に応じて分割）

`geo_algorithms` 側は幾何カーネルのみ保持する。

- `model/geo_algorithms/src/octree/voxel.rs`（`remove_material_*` 系）

---

## 4. 設計/依存ルール

- CAD/CAM境界ルールを維持する
- `scripts/check_architecture_dependencies_simple.ps1` は変更しない
- Phase 1a では UI 連携を入れず、計算コア実装に集中する
- 既存 `ToolPath` / `ToolSet` を入力として扱う

### 4.1 許可依存（cam_sim追加後）

- `cam_sim -> cam_core`
- `cam_sim -> geo_algorithms`
- `cam_sim -> analysis`（必要時）

### 4.2 禁止依存

- `geo_* -> cam_sim`
- `cam_core -> cam_sim`（循環依存防止）

---

## 5. 実装前チェックコマンド

- `cargo build`
- `cargo test --workspace`
- `cargo fmt --all -- --check`
- `.\scripts\check_architecture_dependencies_simple.ps1`
- `.\scripts\check_architecture_dependencies.ps1 -ExitOnError`

---

## 6. cam_sim 追加時の更新チェックリスト（漏れ防止）

### 6.1 ワークスペース

- [ ] `Cargo.toml` の `workspace.members` に `model/cam_sim` を追加
- [ ] `model/cam_sim/Cargo.toml` を作成

### 6.2 依存チェック（スクリプト）

- [ ] `scripts/check_architecture_dependencies.ps1`
  - [ ] `AllowedDependencies` に `cam_sim` を追加
  - [ ] `ForbiddenDependencies` に `cam_sim` 関連ルールを追加
  - [ ] `Get-WorkspaceCrates` の `layerMapping` に `cam_sim` を追加
  - [ ] Layer summary の Model 層に `cam_sim` を追加

### 6.3 CI

- [ ] `common_architecture_check.yml` は既存呼び出し維持（スクリプト変更で追従）
- [ ] `develop_ci.yml` / `main_ci_cd.yml` / `feature_ci.yml` で `cargo build/test --workspace` が通ること
- [ ] 必要時のみ `cargo test -p cam_sim` の明示ステップを追加

---

## 7. 最初のPR受け入れ条件（Phase 1a）

- [ ] `cargo test -p cam_sim` が成功
- [ ] 追加ユニットテストが成功
- [ ] 長い直線/短い分割線でスナップショット密度の偏りが低減されることを確認
- [ ] ドキュメントの「進捗」追記があること
- [ ] `scripts/check_architecture_dependencies.ps1 -ExitOnError` が成功

---

## 8. 次アクション

1. `model/cam_sim` クレートを追加
2. `CuttingSimulator` 最小骨格を `cam_sim` に移設
3. `geo_algorithms` は材料除去カーネルのみ保持
4. 依存チェック/CI更新を反映して検証を通す
