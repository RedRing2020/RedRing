# バッチ計算基盤設計（Dockerヘッドレス + Kubernetes）

**作成日**: 2026年3月8日  
**最終更新**: 2026年3月8日（補足反映）  
**ステータス**: 設計提案（実装前）

---

## 1. 背景

本ドキュメントは、CAD UIとは分離した夜間バッチ計算基盤の要件を定義する。
対象業務は以下。

- 4mクラス大型ワークを含むCAM計算（荒加工・中加工・仕上げ）
- 切削シミュレーション（複数条件の比較）
- NURBS面を含むシートのCAE系計算（例: スプリングバック、モーフィング、オプション検討）

想定運用は、クライアントが夜間に数十ジョブを投入し、翌日に結果確認する方式。

---

## 2. 方針（結論）

- **必須基盤1**: Dockerベースのヘッドレス計算実行環境
- **必須基盤2**: KubernetesによるCAM/切削シミュレーションの夜間バッチ実行・再実行・スケーリング管理
- **オプション基盤**: スプリングバック/モーフィング計算のKubernetesバッチ化（ゴール明確化後に判断）
- **分離原則**: CADのインタラクティブUI処理と、重計算ジョブを明確に分離する

この方針は、既存のCAD/CAM境界ルールと整合する。

---

## 3. 期待効果（CAD/CAM/CAE観点）

### 3.1 Dockerヘッドレス基盤

- 実行再現性の確保（依存ライブラリ、Rustツールチェーン、解析ライブラリ差異の吸収）
- ローカル/CI/本番での挙動差を縮小
- ジョブ単位のバージョン固定（イメージタグで追跡）
- GUIレス実行による安定運用（夜間連続処理に適合）

### 3.2 Kubernetesバッチ基盤

- Job/CronJobでの夜間実行自動化
- 失敗時リトライ、ノード障害時の再スケジュール
- 複数ジョブの並列実行（荒・中・仕上げ、条件分岐ケース）
- リソース要求/制限の明示（CPU/メモリ/GPU）
- 優先度制御（翌朝必要なジョブを先行）

---

## 4. スコープ定義

### 4.1 In Scope

- CAM計算の夜間バッチ
- 切削シミュレーションの夜間バッチ
- ジョブ履歴、成果物、失敗ログの永続化

### 4.2 Out of Scope（本ドキュメント時点）

- CAD UIのコンテナ実行最適化
- リアルタイム共同編集
- 5軸加工の専用最適化詳細

### 4.3 Option Scope（評価ゲート付き）

- NURBSを含むシートCAEジョブ（スプリングバック/モーフィング）のK8sバッチ化
- 判定条件: 計算時間、Undo/Redoデータ量、対話操作性、運用コストが閾値を超える場合のみ導入

---

## 5. 論理アーキテクチャ

```text
Client CAD UI
  -> Batch API (submit/cancel/status/result)
  -> Job Metadata Store
  -> Object Storage (input/output artifacts)

Batch API
  -> Kubernetes Job Controller

Kubernetes Cluster
  - cam-batch-worker (toolpath/cam planning)
  - sim-batch-worker (cutting simulation)
  - cae-batch-worker (springback/morphing; optional)

Observability
  - logs/metrics/traces
  - alerting (nightly failure summary)
```

---

## 6. ジョブモデル（最小）

```text
JobType:
- CamProcessBatch
- CuttingSimulationBatch

Option JobType:
- CaeSpringbackBatch (future-ready)

必須メタデータ:
- job_id, tenant_id, project_id
- requested_at, deadline_at
- priority
- input_artifact_refs
- compute_profile (cpu/memory/gpu)
- worker_image_tag
- retry_policy
```

---

## 7. 計算効率化の具体策

- ケース分割実行: 加工工程ごと（荒/中/仕上げ）にJob分離
- パラメータスイープ: 切削条件を並列探索
- データ局所化: 入力形状と中間結果を同一ストレージ階層に配置
- 差分計算: 変更のない中間結果を再利用（再メッシュ回避）
- 優先度キュー: 翌朝確認対象ジョブを高優先度化
- GPUノード選別: CAEをK8s化する場合のみGPUノードにスケジュール

---

## 8. 非機能要件（初期値）

- 夜間ジョブ成功率: 99%以上
- 再実行完了率（自動リトライ後）: 99.5%以上
- 翌朝開始時刻までの主要ジョブ完了率: 95%以上
- ジョブ追跡可能期間: 30日以上
- 成果物保管期間: 90日以上（運用で調整）

---

## 9. 対応可否判断（現時点）

- Dockerヘッドレス化: **対応可（高）**
- K8sバッチオーケストレーション（CAM/切削）: **対応可（中〜高）**
- CAD UIまで含む全面コンテナ化: **段階導入推奨（中）**
- スプリングバック/モーフィングのK8s化: **オプション（ゴール未確定のため要評価）**

総合判断: **Docker + K8s(CAM/切削)は有効かつ推奨**。CAEはDB基盤案と比較して段階判断する。

---

## 10. 段階導入計画

### Phase A: Dockerヘッドレス標準化

- CAM/切削シミュレーション実行をコンテナ化
- 入出力アーティファクト仕様を固定
- ローカル・CIで同一イメージを使用

### Phase B: K8s夜間バッチ運用開始

- Kubernetes Jobで夜間実行
- 失敗時リトライと通知を実装
- ジョブ実行履歴を可視化

### Phase C: CAEジョブ統合（オプション）

- `CaeSpringbackBatch` を追加
- GPUスケジューリング/ノード分離を導入
- モーフィング計算の中間成果物を再利用

実施条件:
- 1ジョブ当たり計算時間が対話許容時間を継続的に超過
- DBベースUndo/Redoでの保守コストまたは容量コストが閾値超過
- 夜間一括処理需要が継続的に存在

### Phase C'（代替）: DB中心の対話型CAE運用

- スプリングバック/モーフィングをバッチ化せず、CAD機能として対話実行
- 巨大Undo/RedoデータをDBで管理
- 途中状態をスナップショット化し、差分保存と圧縮で保持
- 必要に応じて一部のみ非同期ジョブ化（ハイブリッド）

---

## 11. リスクと対策

- リスク: 大容量形状データI/Oが律速
- 対策: アーティファクト分割、圧縮、差分保存

- リスク: GPUノード不足
- 対策: CAE専用キューと優先度制御

- リスク: CAEのゴール不明確なままK8s実装が先行
- 対策: DB中心運用とK8s化を比較する評価ゲートを先に定義

- リスク: 夜間失敗の発見遅延
- 対策: 朝会前サマリー通知（失敗ジョブ一覧、再実行結果）

---

## 12. RedRing既存方針との整合

- CAD/CAM境界ルールを維持（`geo_* -> cam_*` 禁止）
- バッチ実行責務はCAM/CAE側に集約
- 依存境界違反は既存スクリプトで継続検証

関連:
- `dev/architecture/ARCHITECTURE.md`
- `dev/architecture/CAM_CRATE_DESIGN_PROPOSAL.md`
- `dev/architecture/CUTTING_SIMULATION_DESIGN.md`

---

## 13. 補足メモ（2026年3月8日）

過去CAD開発では、スプリングバックのモーフィング計算はフィーチャ対応のためバッチ化せず、
DB基盤で巨大Undo/Redoを扱う方式を採用していた。
RedRingでも同方式は有効な代替案とし、K8s化は明確なゴール定義後に判断する。

---

## 14. ジョブマネージャー責務境界（追記）

本基盤では、`Job Manager` と `Model` の責務を以下で分離する。

### 14.1 Job Manager の責務

- ジョブ受付（submit/cancel/retry）
- キュー管理、優先度制御、スケジューリング
- 実行状態管理（queued/running/succeeded/failed/canceled）
- タイムアウト、再実行、失敗分類
- 実行ログ収集と成果物参照先（artifact ref）の管理

### 14.2 Model の責務

- CAM/切削/CAEの計算ロジック本体
- 入力から出力を生成する決定的な計算API
- 幾何・CAM境界ルールを守る依存構造
- 計算途中状態の生成（必要に応じてスナップショット）

### 14.3 インターフェース境界

- Job Manager は Model の内部実装に依存しない
- 契約は `JobType + InputRef -> ResultRef` を基本とする
- 進捗はイベントで通知（例: `ProgressUpdated`, `ArtifactReady`, `Completed`）
- モジュール境界として、実行制御と計算ロジックを同一クレートに混在させない

---

## 15. Issue分割案（本ドキュメント起点）

質問の認識どおり、まずは次の2点を最優先Issueとして切るのが妥当。

1. **Docker CI基盤整備**
- ヘッドレス計算イメージ定義
- CI上でのビルド/テスト/実行再現
- イメージタグ戦略と成果物保存ルール

2. **ジョブマネージャー基盤実装**
- `JobType`/`JobStatus`/`RetryPolicy` の共通モデル
- submit/status/cancel/retry API
- CAM/切削ジョブの実行アダプタ接続

推奨（3点目を切る場合）:

3. **運用可観測性基盤（ログ・メトリクス・通知）**
- 夜間失敗サマリー
- ジョブ遅延/失敗率の可視化
- 翌朝確認を前提とした通知フロー
