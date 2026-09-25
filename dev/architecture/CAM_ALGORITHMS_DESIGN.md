# CAM クレート責務設計と cam_algorithms 新設案

**作成日**: 2026年3月25日
**最終更新**: 2026年9月25日
**ステータス**: cam_algorithms 新設済み（#684）- 逆オフセット法による最小 solver 導線を実装（ボール/フラットエンドミル）
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

### 1.2 cam_algorithms（アルゴリズム層）✅ **新設済み（#684）**

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

最小入力は `InputRef`（solver 入力を指す参照）が指す payload とし、以下を必須項目とする。

- `geometry_kind`
  - `nurbs_surface_set`（#684 で実装）
  - `triangle_mesh`（#684 で実装。離散化済みポリゴン入力、STL 等）
  - `nurbs_curve_set`（未実装。2D/2.5D 輪郭系として #212 で扱う）
  - `curve_chain_2p5d`（未実装。同上）
- `tool_id`
- `operation_id`
- `units`
- `coordinate_frame`

補足:

- solver は `InputRef` 参照先を解決して入力を復元する
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

- 状態異常（入力契約エラー）: `invalid_input`
  - 必須項目欠落、型不一致、units/frame 不整合
- 状態正常だが解なし: `no_solution`
  - 入力契約は妥当だが、幾何制約下で有効経路を構築できない
- 内部計算異常（数値解法エラー）: `convergence_failure`
  - 入力契約は妥当だが、反復解法が収束条件を満たさない

補足:

- `convergence_failure` は `invalid_input` とは別分類として扱う

いずれも Job Manager には失敗種別のみ伝達し、幾何計算の内部状態は公開しない。

### 8.6 後続Issueへの接続

- #679: `NcPostFromCam` の artifact 読込導線は本節の `ResultRef` 契約を前提とする
- #680-#683: NC post 拡張系列は、本節で固定した solver->toolpath 導線を前提とする

### 8.7 本番導線シナリオ（Job Manager 経由）

`model/application` / `model/job_runtime` / `model/cam_sim` の本番導線を対象に、以下を最小受け入れシナリオとする。

1. 正常系: CAM親ジョブ -> 切削シミュレーション子ジョブ
  - `CamProcess` が `toolpath` artifact を生成し、`CuttingSimulation` が親 `ResultRef` を参照して成功する
  - 成功時に `status=succeeded`、`result://sim/<id>/ok`、`log://sim/<id>/ok` を返す

2. 異常系: 入力契約不正（invalid_input）
  - 必須項目欠落、units/frame 不整合、`InputRef` 形式不正のいずれかで失敗する
  - Job Manager へは `invalid_input` として集約した失敗分類を返す

3. 異常系: 経路未生成（no_solution）
  - 幾何制約により有効 ToolPath を構築できない入力で失敗する
  - Job Manager へは `no_solution` として失敗分類を返す

4. 異常系: 収束失敗（convergence_failure）
  - 入力契約は妥当だが、反復解法が収束閾値を満たさない場合に失敗する
  - Job Manager へは `convergence_failure` として失敗分類を返す

5. 契約境界系: 親子制約違反
  - `CuttingSimulation` の `parent_job_id` 欠落、または親IDと `input_ref` の不一致で reject する
  - `JobType + InputRef -> ResultRef` 契約を壊さず、内部詳細は公開しない

### 8.8 実装順序（最小）

1. 入力契約モデルと失敗分類マッピングの固定
2. CAM親ジョブから `toolpath` artifact 生成導線の最小実装
3. 切削シミュレーション子ジョブでの `ResultRef` 参照実行
4. 上記 8.7 の 5 シナリオをテストで固定

### 8.9 実装（#684 完了時点）

配置:

- `model/cam_algorithms`: solver 本体（純粋計算）
  - `solver`: 入力契約 `CamSolverInput`、失敗分類 `CamSolverError`、入口 `solve_toolpath`
  - `tessellation`: NURBS 曲面集合 → 三角形メッシュ
  - `inverse_offset`: 逆オフセット法による CL 算出（`DropCutter` / `CutterShape`）
  - `scanline`: `operation_type = scanline` の経路化
- `model/cam_sim`: Job アダプタと参照解決
  - `CamSolverInputProvider`: `InputRef` → `CamSolverInput` の解決境界
  - `InMemoryCamSolverInputStore`: プロセス内 provider 実装（`input://cam/<name>` のみ受理）
  - `CamJobExecutorAdapter::with_input_provider`: provider 接続済みアダプタ

3D 経路生成方式（逆オフセット法）:

- 形状を三角形へ離散化し、各要素を工具形状で逆オフセットした面の上側包絡を工具中心高さとする
- 出力は工具先端（ToolPath 座標）の高さとする
- 凹辺のオフセットは他要素の包絡に覆われるため全辺を評価しても結果は変わらない（凸辺限定は #256 の高速化対象）

工具種別ごとの要素オフセット:

| 工具 | 頂点 | 辺 | 面 | 対応 |
|------|------|----|----|------|
| ボールエンドミル（半径 r） | 球（中心高さ − r が先端） | 円筒（円弧掃引面） | 法線方向 r オフセット平面 | #684 |
| フラットエンドミル（半径 r） | 円板（頂点高さ） | 円板掃引（水平距離 r 以内の辺上最高点） | 底面円周接触（最大傾斜方向へ r 進んだ点） | #684 |
| ラジアスエンドミル（半径 R・コーナー r） | トーラス | トーラス掃引（数値解法） | オフセット面 | #211 |

- いずれも閉形式で解けるボール/フラットを #684 の範囲とし、辺で数値解法を要するラジアスエンドミルは #211 で扱う
- 未対応工具（ラジアスエンドミル）は `invalid_input` を返す

失敗分類の決定箇所:

| 分類 | 決定条件 |
|------|----------|
| `invalid_input` | `InputRef` 形式/ドメイン不正、必須項目欠落、値域外（stepover > 工具径等）、未対応工具、provider 未登録（本番ビルド） |
| `no_solution` | 非退化三角形が存在しない、全走査ラインで工具が形状に接触しない |
| `convergence_failure` | 離散化の弦誤差が `chord_tolerance` 以内に収束しない（再分割反復上限・頂点数上限超過） |

#684 完了条件:

1. 入力契約（`CamSolverInput`）と失敗分類 3 種が実装され、Job Manager へ分類コードのみ伝達される
2. ボール/フラットエンドミルの逆オフセットが要素（頂点・辺・面）単位でテストされ、曲面に対して食い込みなし・接触ありが検証される
3. ラジアスエンドミルは `invalid_input` として明示的に拒否される
4. §8.7 の 5 シナリオが Job Manager 経由で固定され、シナリオ 2-4 は solver 実計算で再現される
5. 本番ビルドで固定 artifact に依存せず CAM 親ジョブ → 切削シミュレーション子ジョブが完走する

テスト/デバッグビルドでは provider 未登録の `InputRef` に対して従来の固定 artifact（suffix 指定の失敗分類を含む）を返す。
本番ビルドでは provider 未登録を `invalid_input` とし、固定 artifact は生成しない。


### 8.10 逆オフセット包絡面のデバッグ表示

逆オフセットの結果を目視確認するため、包絡面（CL 面）を表示するデバッグ機能を持つ。

| 層 | 配置 | 責務 |
|----|------|------|
| Model | `cam_algorithms::cl_grid`（`sample_cl_grid`） | drop-cutter を等間隔格子で評価し工具先端高さを返す |
| Application | `application::cam_inspection`（`inspect_inverse_offset`） | 形状離散化・包絡面評価を束ね、基準点高さ（ボール: 球中心、フラット: 底面中心）の格子を返す |
| ViewModel | `viewmodel::inverse_offset_converter` | 表示モードに応じてシェーディングメッシュと重畳線分へ変換する |
| View | `view/app` の `debug_scene::inverse_offset` | キー操作で表示・切替する |

- 入力形状はサンプル NURBS 曲面（`m` キー表示と同一）、弦誤差はアプリの表示トレランスを用いる
- 評価範囲は形状の XY 範囲を工具半径だけ広げた領域とし、外周で工具が接触する範囲も表示する
- 表示モード
  - 包絡面: 包絡面をシェーディング表示し、元形状を三角形エッジで重畳する
  - 格子線: 元形状をシェーディング表示し、包絡面を一定ピッチ（評価間隔 × 格子線間隔）の格子線で重畳する
- 接触境界（工具が形状に触れ得る範囲の外縁）
  - Model 層で、接触あり/なしが切り替わる格子辺ごとに二分法で境界点を求める（停止条件は既定の距離トレランス）
  - 両モードで境界を輪郭線として重畳し、境界を跨ぐ格子セル・格子線は境界点まで切り詰める
  - 形状の角の外側では、工具が触れ得るのは角頂点のみのため、包絡面は角頂点を中心とする半径 r の 1/4 円で丸まる（欠けではなく幾何的に正しい形状）
- キー操作: `i` 表示、`Shift+I` 工具切替（ボール/フラット）、`g` 表示モード切替
---

## 9️⃣ #689 工程テンプレート精度プロファイル運用契約（Step A）

本節は Issue #689 の設計固定を目的とし、加工ステージ別オペレーションと精度プロファイル適用契約を定義する。

### 9.1 精度プロファイル

- `press_rough`: 0.001mm
- `mold_finish`: 0.0001mm

`tolerance_profile` の適用優先順位:

1. オペレーション定義の `tolerance_profile` 明示指定
2. 工程テンプレートの `tolerance_profile` 既定値
3. solver の `tolerance_profile` 既定値

### 9.2 `operation_type` 決定規約

`operation_type` で受理する canonical token は以下に固定する。
実装・保存・artifact 追跡では、別名や自然言語ではなくこれらの token を用いる。

- `contour_offset`
  - 等高線オフセット加工。荒加工での外周からの段階切込み、およびストック入力ありの等高線オフセット加工を含む
- `rest_machining`
  - 等高残加工。前工程や大径工具で残った未加工領域を小径工具などで追い込む加工
- `scanline`
  - スキャン加工。一定方向の往復または片方向走査で面を仕上げる加工
- `surface_follow`
  - 面沿い加工。対象面の法線・曲率・パラメトリック流れに追従して経路を生成する加工

`operation_type` の適用優先順位:

1. `operation_type` 明示指定
2. 工程テンプレートの `operation_type` 既定値
3. solver の `operation_type` 既定値

### 9.3 加工ステージ別オペレーション

`machining_stage` は実行順序を強制するための状態ではなく、工程テンプレート上のタグ分類として扱う。

- 荒加工（rough）
  - 等高線オフセット加工（`contour_offset`）
- 中加工（semi_finish）
  - ストック入力あり等高線オフセット加工（`contour_offset`）
  - 等高残加工（`rest_machining`）
- 仕上げ（finish）
  - スキャン加工（`scanline`）
  - 面沿い加工（`surface_follow`）
  - 小径工具による等高残加工（`rest_machining`、等高中加工後の追い込み用途）

補足:

- 等高残加工は `semi_finish` を基本配置とするが、小径工具での追い込み時は `finish` でも許容する
- `machining_stage` タグと `operation_type` の組み合わせで運用し、単純な前後関係だけで reject しない
- `machining_stage` と `operation_type` は直交する属性とし、`finish` だから常に `surface_follow` になる、といった自動推論は行わない

### 9.4 加工範囲指定方式

- `edge_projected_2d`
  - エッジ指示を加工方向へ投影した2D境界を使用
- `rectangle`
  - 矩形座標値（min/max）を直接指定

入力契約（最小）:

- `operation_type`
- `machining_stage`
- `boundary_mode`
- `machining_direction`
- `tolerance_profile`
- `stock_ref`（`machining_stage = semi_finish` かつ `operation_type = rest_machining` の時に必須）

必須フィールド制約:

- `machining_direction`
  - canonical token: `+X` / `-X` / `+Y` / `-Y` / `+Z` / `-Z`
  - 単位なしの軸方向指定として扱い、角度値や任意ベクトルは受理しない
- `boundary_mode = edge_projected_2d`
  - `edge_refs`（1件以上）を必須とする
  - `edge_refs` を `machining_direction` へ投影して2D境界を構築できない場合は失敗分類とする
- `boundary_mode = rectangle`
  - `rect_min=(x_min,y_min)` と `rect_max=(x_max,y_max)` を必須とする
  - 座標系はワーク局所座標、単位は mm 固定とする
  - `x_min < x_max` かつ `y_min < y_max` を満たさない場合は失敗分類とする
- `tolerance_profile`
  - 許容値は `press_rough` / `mold_finish` のみとする
  - 未指定は `missing_tolerance_profile`、未知値は `invalid_tolerance_profile` として失敗分類する
- `operation_type`
  - 許容値は `contour_offset` / `rest_machining` / `scanline` / `surface_follow` のみとする
  - 別名 token の導入は許可しない

### 9.5 失敗分類（初期・内部分類）

- `invalid_tolerance_profile`
- `missing_tolerance_profile`
- `unit_mismatch`
- `boundary_projection_failed`
- `empty_projected_boundary`
- `operation_boundary_out_of_domain`
- `stock_required_but_missing`

失敗分類マッピング:

- 本節の列挙はテンプレート適用段階の内部分類であり、Job Manager へ直接公開する分類コードではない
- Job Manager へ返す失敗分類は #684 で定義済みの 3 分類（`invalid_input` / `no_solution` / `convergence_failure`）に統一する
- #689 で追加した内部分類は以下へ集約する
  - `invalid_tolerance_profile`
  - `missing_tolerance_profile`
  - `unit_mismatch`
  - `boundary_projection_failed`
  - `empty_projected_boundary`
  - `operation_boundary_out_of_domain`
  - `stock_required_but_missing`
  - 上記はすべて `invalid_input` へマップする

### 9.6 責務境界

- Job Manager はテンプレート本体を解釈せず、`InputRef` / `ResultRef` 契約を維持する
- Model/CAM 側がテンプレートを解決し、profile と operation を solver へ適用する

### 9.7 工程順序ポリシー

- 工程順序チェックは設定可能な警告として扱う（既定は warning、hard error にはしない）
- 例: `finish` が `semi_finish` より先行する場合、設定有効時に警告を出す
- 警告出力の有無はテンプレート設定で制御し、`JobType + InputRef -> ResultRef` 契約は維持する

### 9.8 実装（Step B/C）

配置:

- `cam_core::tolerance`: 精度プロファイル `ToleranceProfile`（`press_rough` / `mold_finish`）と加工トレランス定数
  - 数値カーネルの判定トレランスではなく CAM の加工精度のため、既存の CAM トレランス（`CamTolerance`）と同じ場所に置く
- `cam_algorithms::process_template`: 工程テンプレートの解決と solver への適用
  - `resolve_operation(template, operation, defaults)`: 優先順位に従い `tolerance_profile` / `operation_type` / `machining_stage` / 加工範囲を解決
  - `build_solver_input(resolved, ...)`: 解決結果を `CamSolverInput` へ適用（`chord_tolerance` = プロファイルの加工トレランス）
- `CamSolverInput.boundary`: 加工範囲（未指定なら形状の XY 範囲全体）

優先順位の解決規則:

- 明示指定 → 工程テンプレート既定値 → solver 既定値（`SolverDefaults`）の順に最初の指定を採用する
- 採用した token が未知の場合は下位の既定値へ落とさずに失敗する
- `SolverDefaults` の初期値は未設定とし、どこにも指定がなければ `missing_tolerance_profile` / `missing_operation_type` とする（精度の暗黙適用を防ぐ）

内部分類の追加（§9.5 の列挙に対する実装上の追加。いずれも `invalid_input` へ集約）:

| 内部分類 | 条件 |
|----------|------|
| `invalid_operation_type` | `operation_type` が canonical token 以外 |
| `missing_operation_type` | `operation_type` がどこにも指定されていない |
| `invalid_machining_stage` | `machining_stage` が `rough` / `semi_finish` / `finish` 以外 |
| `invalid_rectangle_boundary` | 矩形が `x_min < x_max` かつ `y_min < y_max`（有限値）を満たさない |
| `unsupported_operation_type` | token は正しいが solver が未対応（現状 `scanline` 以外） |

`invalid_input` の reason は先頭を `<内部分類>` とし、詳細がある場合のみ `<内部分類>: <詳細>` とする（`missing_*` や `operation_boundary_out_of_domain` のように詳細を持たない分類もある）。判定は先頭の内部分類で行い、Job Manager へは分類コード `invalid_input` のみを伝達する。

矩形加工範囲（`boundary_mode = rectangle`）:

- 「形状の XY 範囲を工具半径だけ広げた領域」（reach）を工具が形状に触れ得る範囲とする
- 矩形を reach でクリップした結果が面積を持たない場合（重ならない、または辺・角で接するだけ。幅・高さが距離トレランス以下）は `operation_boundary_out_of_domain`（経路生成まで進めて `no_solution` にしない）
- 重なる場合は矩形を reach でクリップした範囲を走査する（形状外でも工具半径以内なら接触し得るため形状範囲では切らない。reach 外は接触し得ないため、過大な矩形でもサンプル数を形状規模に抑える）

後続 Step とする項目:

- `edge_projected_2d` と `machining_direction`（投影方向の解釈を含む）
- `stock_ref` と `stock_required_but_missing`（`rest_machining` の solver 対応と合わせる）
- 工程順序の警告（§9.7）
- `succeeded` 時の適用 profile / tolerance / operation_type の追跡メタデータ

計測（40mm 角の双二次ドーム、ボール R3、release ビルド）:

| プロファイル | 頂点数 | 三角形数 | 離散化 | 経路生成 |
|--------------|--------|----------|--------|----------|
| `press_rough` (0.001mm) | 28,561 | 56,448 | 80ms | 97ms |
| `mold_finish` (0.0001mm) | 263,169 | 524,288 | 633ms | 875ms |

`mold_finish` は頂点数・時間とも約 9 倍となる。経路生成は工具半径内の三角形数が支配的であり、#256（凸辺限定・要素絞り込み）の判断材料とする。計測は `profile_load_measurement`（`#[ignore]`）で再現できる。
