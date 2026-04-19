# CAM クレート責務設計と cam_algorithms 新設案

**作成日**: 2026年3月25日
**最終更新**: 2026年3月29日
**ステータス**: 設計案更新中 - 依存例具体化、PoC選定、2D/2.5D方針追記済み
**関連Issue**: #413（親）、#426（分離）、#411、#412、#503、#504

---

## 📋 概要

現状の CAM 関連クレートは `cam_core`（データ/契約）・`cam_sim`（切削シミュレーション）・`cam_entity`（表示/属性統合）に分かれており、**経路生成・最適化・干渉回避などのアルゴリズム層の受け皿が明確でない**。

本設計では、`geo_algorithms` と対称的な責務分離を実現するため、`cam_algorithms` 新設を提案し、CAM クレート間の依存方向を確定する。

---

## 🎯 目的

1. **責務分離の明確化**: 各 CAM クレートが持つべき責務を一覧化
2. **依存方向の固定**: 許可/禁止される依存を定義し、将来実装の迷いを削減
3. **サーキュラー依存の回避**: `geo_*` 層との境界ルール確認
4. **段階導入の基盤**: 後続 Issue（アルゴリズム実装）の立案基盤

---

## 1️⃣ CAMクレート責務表（確定案）

### 1.1 cam_core（データ基盤層）

**責務**:

- ToolPath / Tool セット定義（構造体・trait）
- Artifact バイナリ I/O 契約（read/write）
- ToolPathKinematicMeta / MachineConstraint（最小型）
- ToolPose / 補間ポリシー定義

**許可される依存**:

- ✅ `geo_contracts`, `geo_primitives`, `geo_nurbs`（形状参照）
- ✅ `geo_core`（基本型）
- ✅ `geo_commons`（共通定数）

**禁止される依存**:

- ❌ `cam_algorithms`（単向依存、逆はOK）
- ❌ `cam_sim`（消費側への依存は不可）
- ❌ `cam_entity`（表示層への依存は不可）

**現行実装**:

- `model/cam_core/src/toolpath.rs`
- `model/cam_core/src/tool.rs`
- `model/cam_core/src/artifact_binary.rs`

---

### 1.2 cam_algorithms（アルゴリズム層）✨ **新設候補**

**責務**:

- ToolPath 生成・最適化アルゴリズム
- 干渉回避・衝突チェック
- 経路順序最適化
- 工具軌跡ビジュアライゼーション補助
- 機械制約の適用・検証

**許可される依存**:

- ✅ `cam_core`（データ モデル利用）
- ✅ `geo_algorithms`（交差/衝突判定）
- ✅ `geo_primitives`, `geo_nurbs`（形状演算）
- ✅ `geo_contracts`, `geo_core`, `geo_commons`

**禁止される依存**:

- ❌ `cam_sim`（シミュレーション側への依存禁止）
- ❌ `cam_entity`（表示層への依存禁止）

**位置づけ**:

- `geo_algorithms` との対称性を確保
- 形状演算よりも上位（CAM固有ロジック）

**想定実装例**:

- `cam_algorithms/src/path_optimization/` - 経路順序最適化
- `cam_algorithms/src/collision_avoidance/` - 干渉回避
- `cam_algorithms/src/machine_validation/` - 機械制約検証

---

### 1.3 cam_sim（CAM特化ユースケース実行層）

**責務**:

- ToolPath の実行シミュレーション
- フレーム進行・時刻管理
- 切削体積計算（離散幾何ベース）
- シミュレーション結果のキャッシング

**位置づけ**:

- 物理配置は `model/` 配下だが、責務としては純粋データ層というより CAM ドメイン専用の実行・ユースケース層に近い
- `geo_algorithms` より上位で、`cam_core` が持つ中立データを消費してシミュレーションという業務処理を実行する
- そのため `cam_sim` は「Model内のCAM特化アプリケーション層」と捉えるのが実態に近い

**許可される依存**:

- ✅ `cam_core`（ToolPath 読み取り）
- 🔶 `cam_algorithms`（段階的・必要時のみ）
  - 理由: アルゴリズム関数の呼び出しが必要になる場合がある
  - ただし常時依存は避ける。インターフェッシングは最小化
- ✅ `geo_primitives`, `geo_algorithms`（幾何演算）

**禁止される依存**:

- ❌ `cam_entity`（表示層への直接依存は避ける）

**現行実装**:

- `model/cam_sim/src/`

**`cam_algorithms` を参照する具体例**:

- シミュレーション開始前の ToolPath 正規化
  - 例: `cam_algorithms::path_optimization` が近接セグメント統合や順序調整を行い、`cam_sim` は最適化後の経路だけを実行する
- ステップ実行中の機械制約再検証
  - 例: ロータリ軸角度や姿勢補間結果が `MachineConstraint` を満たすかを `cam_algorithms::machine_validation` に問い合わせる
- 再生品質向上のための事前干渉チェック
  - 例: シミュレーション本体は切削除去量を担当し、ホルダ干渉や退避不足の判定だけを `cam_algorithms::collision_avoidance` に委譲する

**設計上の整理**:

- `cam_sim` が直接持つべきなのは時間進行、除去量更新、結果キャッシュ
- 経路の改善、実行前検証、干渉判定ロジックは `cam_algorithms` に集約する
- そのため `cam_sim -> cam_algorithms` は「実行前後の問い合わせ」に限定し、シミュレーション状態そのものは渡さない

---

### 1.4 cam_entity（表示/統合層）

**責務**:

- UI エンティティ（ToolPanel、PathPanel 等）
- Tool/ToolPath の属性統合
- ビューバインディング
- 表示不要な内部状態に関する ID・メタデータ

**許可される依存**:

- ✅ `cam_core`, `cam_sim`（データ読み取り）
- ✅ `geo_entity`（表示用形状エンティティ）

**禁止される依存**:

- ❌ `cam_algorithms`（消費側はアルゴリズムに直依存する設計は避ける）

**現行実装**:

- `model/cam_entity/src/`

---

## 2️⃣ 依存方針（確定案）

### 依存グラフ（許可関係）

```
┌─────────────────────────────────────────────────────┐
│  geo_* 層（CADベース幾何演算）                      │
│  ├─ geo_core                                        │
│  ├─ geo_primitives / geo_nurbs                      │
│  └─ geo_algorithms（交差/衝突等）                   │
└───────────────┬─────────────────────────────────────┘
                │（依存許可）
                ↓
┌─────────────────────────────────────────────────────┐
│  CAM層（CAMアルゴリズム・実行）                     │
│                                                     │
│  ┌─────────────────────────────────────────┐       │
│  │ cam_core（データ基盤）                  │       │
│  │ - ToolPath/Tool 定義                    │       │
│  │ - Artifact I/O 契約                     │       │
│  └─────────────────────────────────────────┘       │
│              ↑        ↑         ↑                   │
│              │        │         │(依存)             │
│         (逆依存禁止) (逆依存禁止) (逆依存禁止)      │
│              │        │         │                   │
│  ┌───────────┴──┐  ┌──┴─────┐  ┌──┴──────┐        │
│  │cam_algorithms│  │cam_sim  │  │cam_entity│       │
│  │(新設)        │  │         │  │（UI層）  │       │
│  └───────────┬──┘  └──┬─────┘  └──┬──────┘        │
│              │(依存OK) │(段階的)    │(許可)        │
│              └────┬────┘         ┌─┘              │
│                   ↓              ↓                │
│              [アルゴリズム]    [表示]            │
└─────────────────────────────────────────────────────┘
```

### 依存方向の規則表

| From | To | 許可 | 備考 |
|------|-----|-----|------|
| `cam_core` | `cam_algorithms` | ❌ | データ基盤は逆依存のみ |
| `cam_core` | `cam_sim` | ❌ | シミュレーション層への依存不可 |
| `cam_core` | `cam_entity` | ❌ | 表示層への依存不可 |
| `cam_algorithms` | `cam_core` | ✅ | **正向依存OK** |
| `cam_algorithms` | `geo_algorithms` | ✅ | 幾何アルゴリズムへの依存OK |
| `cam_algorithms` | `cam_sim` | ❌ | シミュレーション層への依存不可 |
| `cam_algorithms` | `cam_entity` | ❌ | 表示層への依存不可 |
| `cam_sim` | `cam_core` | ✅ | データ読み取りOK |
| `cam_sim` | `cam_algorithms` | 🔶 | **限定的 OK**（*1） |
| `cam_sim` | `cam_entity` | ❌ | 直接依存は避ける |
| `cam_entity` | `cam_core` | ✅ | データ読み取りOK |
| `cam_entity` | `cam_algorithms` | ❌ | 存在しない関数呼び出し |
| `cam_entity` | `cam_sim` | ✅ | シミュレーション読み取りOK |

*1: `cam_sim -> cam_algorithms` について

- **許可理由**: ツールパス再検証、干渉チェック再実行など、シミュレ中にアルゴリズムが必要な場合がある
- **制約**: 逆依存（`cam_algorithms -> cam_sim`）は厳禁
- **運用**: インターフェイスは最小化。FAT ファンクション は避ける

---

## 3️⃣ CAM層と geo_* 層の境界ルール

### 原則

```
┌──────────────────────────────────┐
│  geo_* 層                        │
│  （汎用幾何演算・アルゴリズム）  │
└──────────────────────────────────┘
         ↑ 依存 (OK)
         │
┌──────────────────────────────────┐
│  cam_* 層                        │
│  （CAM固有ロジック）            │
└──────────────────────────────────┘
         │ 依存 OK
         ↓
[アプリケーション層：viewmodel etc]
```

### 具体規則

| 事項 | ルール |
|------|--------|
| **Shape 型参照** | CAM層は `geo_primitives` / `geo_nurbs` の形状を自由に使用可 |
| **アルゴリズム呼び出し** | `geo_algorithms` の関数を `cam_algorithms` から呼び出し可。`cam_core` からは呼び出し不可 |
| **Trait 実装** | CAM固有のtrait（例: `CamSpecificProperty`）は `cam_core` / `cam_algorithms` 側で定義。geo側は拡張不可 |
| **逆依存** | **厳禁**: `geo_*` 層から `cam_*` への依存は一切不可 |

---

## 4️⃣ アーキテクチャスクリプト対応

### 現状の制約スクリプト

ファイル: `scripts/check_architecture_dependencies_simple.ps1`

**拡張対象**:

- `cam_algorithms` を新しいレイヤーとして追加
- 上記「依存方向の規則表」をスクリプトで検証可能な形に記述

**単なる許可表追加以外に必要な作業**:

- `Cargo.toml` の workspace members に `model/cam_algorithms` を追加する
- `scripts/_arch_rules_data.ps1` の `ARCH_LAYER_MAPPING` に `cam_algorithms` を追加する
- `scripts/_arch_rules_data.ps1` の `ARCH_LAYERS` に `cam_algorithms` を Model 層として追加する
- `scripts/_arch_rules_data.ps1` の `ARCH_ALLOWED_DEPS` に `cam_algorithms` 自身の許可依存と、既存クレート側の `cam_algorithms` 許可有無を反映する
- `scripts/_arch_rules_data.ps1` の `ARCH_FORBIDDEN_DEPS` に `cam_algorithms` を含む禁止方向を反映する
- 必要なら `ARCH_REQUIRED_MODEL_CRATES` に `cam_algorithms` を追加し、新設後は存在必須クレートとして扱う
- `dev/architecture/ARCHITECTURE.md` 側の層構成説明も同時更新する

**重要な整理**:

- `check_architecture_dependencies_simple.ps1` 本体は共有ルールデータを読むだけなので、主変更点は `scripts/_arch_rules_data.ps1`
- つまり「cam_algorithms 追加時はスクリプト反映が必須でセット実施」という認識で正しい
- さらに実運用では、workspace 参加、設計ドキュメント更新、依存ルール更新を同一変更セットに含める必要がある

**実装例** (pseudo-code):

```ps1
$allowed = @(
    "cam_algorithms -> cam_core",
    "cam_algorithms -> geo_algorithms",
    "cam_sim -> cam_core",
    "cam_sim -> cam_algorithms",  # 限定的（注記）
    "cam_entity -> cam_core",
    "cam_entity -> cam_sim",
)

$forbidden = @(
    "cam_core -> cam_algorithms",
    "cam_core -> cam_sim",
    "cam_core -> cam_entity",
)
```

---

## 5️⃣ 最小 PoC（実装候補）

新規 `cam_algorithms` 初版での最初の実装候補：

### PoC-1: ToolPath 順序最適化（選定済み）

**目的**: リコンフィグ移動時間を最小化
**スコープ**: 等高線レベル / Segment の順序入替え最適化
**難易度**: 中（組み合わせ最適化基盤実装）
**期間**: 1〜1.5週間
**関連Issue**: 後続の Issue #2XX として起票

**選定理由**:

- `cam_core` の既存 ToolPath 表現をそのまま入力に使える
- `cam_sim` や UI に依存せず、`cam_algorithms` 単体の責務を検証しやすい
- #426 の機械制約拡張より前でも着手でき、後続の制約検証とも競合しにくい

### PoC-2: 機械制約検証

**目的**: ToolPath の実行可能性を事前判定
**スコープ**: 回転軸範囲チェック（#426 に関連）
**難易度**: 低（単純な値チェック）
**期間**: 3〜5日

### PoC-3: 干渉回避（Pre）

**目的**: 工具干渉チェック（全形状ペア）
**スコープ**: #338 / #350 等の collision 実装完了後
**難易度**: 高（Phase C に依存）
**期間**: TBD（後続スケジュール次第）

---

## 6️⃣ 後続実装 Issue への分割

本設計確定後、以下の Issue を起票予定：

| # | Title | 内容 | 依存 | 工数 |
|---|-------|------|------|------|
| TBD | [Phase X] cam_algorithms新設と最小実装 | クレート作成、カーゴ化、PoC-1 | #413 | 1-1.5w |
| #426 | MachineConstraint拡張 | 直動軸・速度/加速度 | #413 | 1.5-2w |
| TBD | [Phase X] ToolPath順序最適化アルゴリズム | 組み合わせ最適化実装 | 上記 | 1-1.5w |
| TBD | [Phase X] 機械制約検証API | constraint check インターフェース | #426 | 0.5-1w |

---

## ✅ 受け入れ条件（本Issue #413 の完了条件）

- [x] `cam_algorithms` 新設の採否が明文化される（✅ 採択）
- [x] CAM クレート責務表（core/algorithms/sim/entity）が確定する（本文 Section 1）
- [x] 依存方向の許可/禁止が文書化される（本文 Section 2）
- [x] `scripts/check_architecture_dependencies_simple.ps1` の既存方針に反しない導入手順が定義される（本文 Section 4）
- [x] 最小 PoC の対象が1件以上選定される（本文 Section 5 - PoC-1を選定）
- [ ] 後続実装 Issue を起票できる粒度でスコープ分割される（本文 Section 6）

---

## 📎 関連ドキュメント

- `dev/architecture/ARCHITECTURE.md` - 全体レイヤー構成
- `dev/architecture/CAM_CRATE_DESIGN_PROPOSAL.md` - 旧設計（参考）
- `dev/archive/issues/issue-257-five-axis-toolpath-archive-note.md` - 5軸表現設計（完了）
- Issue #411 - `read_artifact_v1` 導線統一
- Issue #412 - artifact API 命名整理
- Issue #426 - MachineConstraint拡張（本設計に依存）

---

## 📝 ステータス

- **作成日**: 2026年3月25日
- **設計段階**: 🔶 主要方針は承認済み、依存例を具体化して継続整理中
- **実装開始**: ⏳ #413 完了 → 後続 Issue で実施

---

## 7️⃣ 2D/2.5D優先のToolPath方針（2026年3月29日合意）

本節は、Issue #504（PathGeometry拡張）に着手する前に固定すべき前提を記録する。

### 7.1 前提

- 2D/2.5D加工は、3D/5軸より幾何表現が単純でも、加工意図・固定サイクル・制御機差分の取り扱いが複雑になりやすい
- 仕様変更は CAM計算と切削シミュレーションの両方へ波及するため、実装着手前の方針固定を必須とする
- v0（未リリース）運用のため、現時点では破壊的変更を許容する

### 7.2 先に固定する責務分離

ToolPathの複雑化抑制のため、以下を分離する。

- OperationKind（加工意図）
  - 例: Contour, Pocket, DrillCycle, FaceCycle, DeepDrillCycle, ThreadCycle
- PathGeometry（幾何表現）
  - 例: Line, Arc, Helix, PlanarSpiral
- PostMapping（制御機依存写像）
  - FixedCycleKind/variant を CNCコントローラー向け G-code へ変換

### 7.3 PathGeometry分類ルール（誤正規化の防止）

- Arc: 半径一定、中心一定の円弧
- Helix: 半径一定 + Z連続変化
- PlanarSpiral: 半径が連続変化する平面螺旋（Z一定を許容）

注意:

- PlanarSpiral は Arc へ正規化しない
- `delta_z == 0` は Arc と同値ではない
- XY平面で穴径を広げる螺旋切削は PlanarSpiral として保持する

### 7.4 固定サイクルの位置づけ（PoC対象）

- 固定サイクルはポスト処理に直結するが、アプリケーション層/モデル層では制御機非依存の正規化表現を保持する
- Fanuc系の呼び分け（例: G181/G182/G183/G184、variant）を想定しつつ、cam_core側は canonical な種別を優先する
- マクロ挿入は商用要件として認識するが、当面は凍結する

### 7.5 #504 PoCの最小受け入れ条件

- [ ] OperationKind と PathGeometry を分離した最小データ構造を定義できる
- [ ] PathGeometry に Arc/Helix/PlanarSpiral を独立表現として追加できる
- [ ] 固定サイクルの canonical 種別（最低4種）を保持できる
  - Drill
  - Face
  - DeepDrill
  - Thread
- [ ] controller profile を切り替えて fixed cycle を写像できる設計を示せる
- [ ] マクロ挿入を凍結したまま将来拡張点を確保できる

### 7.6 実装前に決める最終論点

- fixed cycle の初期採用種別（4種で開始するか）
- パラメータ必須項目（R/Z/Q/P/F 等）の最小セット
- variant の表現（String許容か、enum化するか）

上記3点の合意を、Issue #504 の実装開始条件とする。

---

## 8️⃣ #684 CAMソルバー最小導線契約（Option A / Step A）

本節は Issue #684 の設計固定を目的とし、実装前提となる最小契約を定義する。

### 8.1 目的

- 形状データ（NURBSを含む）入力から ToolPath artifact を生成し、`NcPostFromCam` へ接続可能な最小導線を固定する
- `JobType + InputRef -> ResultRef` 契約を維持したまま、solver 導線を明文化する

### 8.2 入力モデル（最小）

最小入力は `SolverInputRef` が指す payload とし、以下を必須項目とする。

- `geometry_kind`
  - `nurbs_surface_set`
  - `nurbs_curve_set`
  - `curve_chain_2p5d`
- `tool_id`
- `operation_id`
- `units`
- `coordinate_frame`

補足:

- Solver は `InputRef` 参照先を解決して入力を復元する
- `cam_sim` 側で参照解決するが、Job Manager は payload 本体を解釈しない

### 8.3 solver 最小責務

- 入力契約の検証
- ToolPath 生成（最小形）
- 失敗分類の返却
  - `invalid_input`
  - `no_solution`
  - `convergence_failure`
- 生成結果を `toolpath` artifact binary v0.1 に接続

非責務（本Issueの非スコープ）:

- 加工時間最小化などの高度最適化
- コントローラ固有 post 最適化
- UI編集機能

### 8.4 最小導線シーケンス

1. `JobType::CamProcess` で `InputRef` を受理
2. `InputRef` から solver 入力を復元
3. solver を実行して ToolPath を生成
4. `toolpath` artifact binary v0.1 として永続化
5. `ResultRef` を返却し、後続 `NcPostFromCam` が参照する

### 8.5 失敗分類と契約境界

- `invalid_input`
  - 必須項目欠落、型不一致、units/frame 不整合
- `no_solution`
  - 幾何制約下で有効経路を構築できない
- `convergence_failure`
  - 反復解法が収束条件を満たさない

いずれも Job Manager には失敗種別のみ伝達し、幾何計算の内部状態は公開しない。

### 8.6 後続Issueへの接続

- #679: `NcPostFromCam` の artifact 読込導線は本節の `ResultRef` 契約を前提とする
- #680-#683: NC post 拡張系列は、本節で固定した solver->toolpath 導線を前提とする
