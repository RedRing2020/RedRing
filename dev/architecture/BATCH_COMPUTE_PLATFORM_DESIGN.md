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
- CamProcess
- CuttingSimulation

Option JobType:
- CaeSpringback (future-ready)

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

### 14.4 クレート配置方針（#298）

- 共通実行制御は `model/job_runtime` に配置する
- `job_runtime` は CAD/CAM/CAE の計算実装に依存しない
- CAM/切削の実行接続は `cam_*` 側アダプタで担保する
- 将来のNC Post/CAEジョブも同一契約へ接続できるよう、`JobType + InputRef -> ResultRef` を維持する

### 14.5 初期接続実装方針（スタブ）

- #298 の初期接続は `model/cam_sim` に `JobExecutor` アダプタを実装する
- アダプタは `JobType::CamProcess` / `JobType::CuttingSimulation` の2系統を受け付ける
- 初期段階では実計算を呼ばず、`InputRef` を検証して `ResultRef` を返すスタブ動作とする
- タイムアウト/リトライ/キャンセルは `job_runtime` 側の実行制御で検証する
- 実計算への差し替えは後続Issueで行い、同じ契約を維持したまま移行する

### 14.6 ジョブ階層・グループ管理拡張（#311）

- `JobRecord` に `parent_job_id` と `group_id` を持たせ、ツリー/一覧の両方を扱う
- 取得APIとして `list_by_parent` / `list_by_group` を追加する
- グループ単位の集約状態と進捗率を `JobGroupSummary` で提供する
- グループ進捗更新を `GroupProgressUpdated` イベントで通知する
- 互換性より基盤整理を優先し、必要に応じて破壊的変更を許容する

### 14.7 成果物有効性と再計算伝播（#311追加）

- `JobRecord` は単一 `result_ref` ではなく、成果物履歴 (`output_history`) を保持する
- 各成果物には有効状態を持たせ、`Active` / `Superseded` / `InvalidatedByDependency` を識別する
- 参照APIは分離し、現在有効成果物取得 (`active_result_ref`) と履歴取得 (`output_history`) を明示的に使い分ける
- 子ジョブ成功時は親ジョブの有効成果物を `Superseded` に更新する
- 親ジョブ再実行で新成果物が確定した場合、子孫ジョブは `NeedsRecompute` へ遷移し、既存成果物を `InvalidatedByDependency` とする

### 14.8 ワークフロー制約の追加検討（3点目）

- CAM工程における `CuttingSimulation` は 1 工程につき 1 件のみ許可する
- `CuttingSimulation` は工程の末尾にのみ配置可能とする（後続子ジョブを持たない）
- 親なし `CuttingSimulation` は禁止とし、`CamProcess` または許可された中間工程の子としてのみ投入可能とする
- 上記の業務制約は `cam_sim` 側で検証し、`job_runtime` は汎用的な関係管理・状態遷移・再計算伝播を担当する

現時点の許可パターン:

- `CamProcess -> CuttingSimulation` は許可
- `NcImport -> CuttingSimulation` は将来許可予定（`NcImport` 未実装のため現時点では未適用）
- `CuttingSimulation` の親が上記以外になる投入は拒否
- `CuttingSimulation` 自身を親にする投入は拒否（末尾制約）

実装方針（段階適用）:

- Phase A（現行）: `CamProcess -> CuttingSimulation` のみを `cam_sim` 側ファサードで強制
- Phase B（NCimport実装後）: `NcImport -> CuttingSimulation` を同ファサードの許可テーブルへ追加

受け入れ観点（3点目）:

- 同一 CAM 工程に対して 2 件目の `CuttingSimulation` 投入は拒否される
- `CuttingSimulation` の子ジョブ追加（末尾違反）は拒否される
- 親なし `CuttingSimulation` 投入は拒否される
- `NcImport` 未実装期間は `NcImport -> CuttingSimulation` を受け付けない

### 14.9 Job ViewModel 連携方針（#305）

本節は #305 の設計合意を固定する。

- 進捗はイベント値を優先しつつ、`JobStatus` からの推定を許可する
  - `ProgressUpdated` 未着時でも UI が進捗表示可能なようにする
  - 例: `Queued=0%`, `Running=推定値または直近値`, `Succeeded/Failed/Canceled=100%`
- 成果物参照は生文字列を直接UIへ渡さず、`Option<ArtifactRefDto>` へ包んで受け渡す
  - `ArtifactRefDto` は `result_ref` / `log_ref` / `validity` を保持する
  - `None` は成果物未生成・参照不可を表現する
- `JobError` の生メッセージを直接表示しない
  - ViewModel でエラーコードへ正規化し、UI向け文言テーブルで解決する
  - 文言は多言語対応を前提とし、表示時にロケールで選択する

多言語対応基盤（最小要件）:

- 文言キー形式: `job.error.<code>` / `job.status.<status>`
- ViewModel は `message_key + args` を返し、最終文字列解決はUI層で実施
- 英語・日本語を初期サポート対象とし、将来ロケール追加可能な構造にする

受け入れ観点（#305追加合意）:

- `ProgressUpdated` が無いジョブでも `JobStatus` 由来の進捗表示が可能
- 成果物参照は `Option<ArtifactRefDto>` で受け渡される
- 失敗表示は `JobError` 直出しではなく、文言キー解決経由になる

### 14.10 Job管理ドメイン抽象化の段階移行（#315 Phase 0）

本節は、破壊的変更を許可したうえで責務境界を再整理するための設計固定である。

現状課題:

- 依存境界維持のため、`cam_sim` に bridge/投入制約が一時的に集約されている
- この状態で機能追加を続けると、`cam_sim` の責務肥大化と境界劣化が進みやすい

目標責務:

- `job_runtime`: 実行基盤（状態遷移/イベント/再試行/履歴）
- `job_domain`（新規想定）: 投入ポリシー/親子制約/ユースケース/境界DTO
- `cam_sim`: 計算接続アダプタ
- `viewmodel`: 表示用DTO変換と文言解決

段階移行計画（破壊的変更許可）:

- Phase 0: 設計固定（本節）
- Phase 1: `job_domain` 最小導入（互換レイヤは作らない）
  - `cam_sim::workflow` / `cam_sim::job_view_bridge` を新設計へ置換する前提で進める
- Phase 2: 投入制約とユースケースを `job_domain` へ移設
  - `cam_sim` は計算接続アダプタ責務のみに縮退する
- Phase 3: 境界DTOとメッセージ解決責務を最終配置へ再編
  - `viewmodel` は表示変換に集中し、ドメイン投入制約を持たない
- Phase 4: 旧経路の削除と依存ルール最終固定
  - 旧API/旧bridge/暫定コードを削除し、アーキテクチャチェックを最終形へ更新

優先順（冗長化を先に潰す）:

1. `cam_sim` の責務集中解消（workflow移設）
2. bridge再配置（`cam_sim` から切り離す）
3. ViewModel文言基盤の汎用化（#314 と連携）

Phaseごとの必須ゲート:

- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- アーキテクチャ依存チェックの通過

運用ルール:

- 各Phaseは小さなPRに分割してレビューする
- 互換性維持より責務分離を優先し、不要コードは早期に削除する

Phase 2 実装方針（2026-03-08 更新）:

- `job_domain` に CAM ワークフローポリシー（SIM末尾制約/親種別制約/重複SIM制約）を実装する
- `cam_sim::workflow` は `JobDomainService<CamWorkflowPolicy>` を呼び出す薄いファサードへ変更する
- `cam_sim` 側は `job_runtime::JobType` と `job_domain` の文字列表現を相互変換する責務のみを持つ
- 既存の `CamWorkflowError` 契約は維持し、`DomainRuleViolation` を同等エラーへマッピングする

Phase 3 / #314 実装方針（2026-03-08 更新）:

- `viewmodel/converter` に `MessageCatalog` trait を導入し、`message_key + args` 文字列解決を独立モジュール化する
- `job_converter` は DTO 変換と message key 生成に集中し、テンプレート解決ロジックを持たない
- `JobError` 文字列の `message_key` 正規化は `job_message_mapper` へ分離する
- `ja/en` テンプレートは `job_message_catalog` として独立管理し、将来のドメイン拡張で差し替え可能にする

Phase 4 後方整理（2026-03-08 更新）:

- `cam_sim` に残っていた旧互換公開を削除する
  - `CamJob*` の再公開を廃止（境界DTOは `job_domain` を正規公開先に統一）
  - `cam_sim::cutting_simulator` 互換モジュールを削除
- 依存ルールは現行最終形を維持し、`converter` は `job_domain` を直接参照する

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

---

## 16. Docker実行イメージ設計（#297向け）

本章は、Job Manager が実行するコンテナの最小構成とセキュリティ基準を定義する。

### 16.1 設計方針（最小構成）

- 目的は「ヘッドレス計算の再現実行」であり、開発ツール一式を本番実行イメージに含めない
- イメージは `builder` と `runtime` の multi-stage を前提とする
- runtime は必要最小限の依存のみを含み、シェルやパッケージマネージャを極力持たせない
- 1コンテナ1プロセスを原則とし、Job Manager は実行と監視に専念する

最小ランタイム構成（論理）:

- `redring-batch-runner`（CAM/切削共通ランナー）
- 読み取り専用の実行バイナリ
- 入出力マウントポイント（`/work/input`, `/work/output`, `/work/logs`）
- 実行時設定は環境変数ではなく、可能な限りジョブ定義（InputRef）経由で受け渡す

### 16.2 非root実行とコンテナセキュリティ基準

必須:

- コンテナプロセスは root 以外の固定UID/GIDで実行する
- `allowPrivilegeEscalation: false` を前提とする
- Linux capabilities は `drop: ["ALL"]` を基本とする
- ルートファイルシステムは read-only を基本とし、書き込みは `/work/output` と `/work/logs` のみ
- 機密情報はイメージに埋め込まず、実行基盤のシークレット参照で注入する
- イメージは digest pin（`image@sha256:...`）で実行し、タグ参照のみの実行を禁止する

推奨:

- ベースイメージはLTS系に限定し、定期スキャン（CVE High/Critical）をCIゲート化する
- `seccomp` は default 以上、可能なら RuntimeDefault を強制する
- 依存ライブラリSBOMを生成し、アーティファクトとして保存する

### 16.3 Job Manager との責務接続

- Job Manager は「どのイメージを、どの入力参照で実行するか」を決定する
- コンテナ内の計算ロジック詳細（CAMアルゴリズム実装）は Model 側責務とし、Job Manager は介入しない
- 契約は既存方針どおり `JobType + InputRef -> ResultRef` を維持する
- 実行結果は終了コード + 成果物参照 + 構造化ログで返却し、Job Manager が状態遷移に反映する

### 16.4 Docker対象の実施環境定義

1. ローカル検証環境（Developer Local）
- 目的: 再現確認、最小ジョブの手動実行
- 要件: 同一イメージdigestでの起動、入力/出力ディレクトリの明示マウント

2. CI検証環境（GitHub Actions等）
- 目的: build/test/run の自動検証
- 要件: イメージビルド、最小ジョブ実行、ログ/成果物保存、脆弱性スキャン

3. 本番バッチ環境（Kubernetes）
- 目的: 夜間バッチの実運用
- 要件: 非root実行強制、read-only rootfs、リソース制限、再実行ポリシー、監査ログ保持

環境ごとの差分は「リソース量」「並列度」「資格情報の供給方法」に限定し、
実行イメージとジョブ契約は共通化する。

### 16.5 受け入れ条件（#297設計観点）

- [ ] 非root固定UID/GIDで CAM/切削の最小ジョブが両方成功する
- [ ] 同一入力で local/CI/K8s の結果差異が許容範囲内である
- [ ] High/Critical 脆弱性がCIで検知された場合にリリースを停止できる
- [ ] 実行イメージのdigest、SBOM、実行ログを追跡可能である

---

## 17. 実装準備設計（ドラフト）

本章は、#297を着手するための具体的な実装ドラフトを示す。

### 17.1 Dockerfile構成案（multi-stage）

`builder` ステージ:

- Rust stable でワークスペースビルド
- テストに必要な最小アセットのみ同梱
- 出力バイナリを `redring-batch-runner` として配置

`runtime` ステージ:

- 最小ベースイメージ（LTS）
- 固定 UID/GID（例: 10001:10001）ユーザー作成
- `USER 10001:10001` で実行
- `WORKDIR /work`
- `/work/output` と `/work/logs` のみ書き込み可能
- エントリポイントはバッチランナー固定（シェル起動を前提にしない）

初期実装で避けるもの:

- Docker-in-Docker
- 特権モード
- ルート権限での暫定運用
- 実行時 `apt install` のような可変依存解決

### 17.2 CIワークフロー構成案（Docker専用ジョブ）

既存 `develop_ci.yml` に追加するジョブ候補:

1. `docker-build-batch-runner`
- Dockerイメージをbuild
- イメージdigestを出力

2. `docker-smoke-test-nonroot`
- 非rootで最小CAMジョブを1件実行
- 非rootで最小切削シミュレーションジョブを1件実行
- `/work/output` と `/work/logs` に成果物が生成されることを検証

3. `docker-security-scan`
- イメージ脆弱性スキャン（High/Criticalでfail）
- SBOM生成とアーティファクト保存

4. `docker-archive-artifacts`
- 実行ログ
- 成果物ハッシュ一覧
- イメージdigest

依存関係:

- `docker-smoke-test-nonroot` は `docker-build-batch-runner` 成功後
- `docker-security-scan` は `docker-build-batch-runner` 成功後
- どれか失敗時は develop CI 全体を fail にする

### 17.3 Job Manager が扱う最小実行契約

最小ジョブ入力:

```text
JobType: CamProcess | CuttingSimulation
ImageRef: ghcr.io/redring2020/redring-batch-runner@sha256:...
InputRef: object storage path or immutable artifact id
OutputRef: destination path
TimeoutSec: integer
RetryPolicy: maxRetries + backoff
```

最小ジョブ出力:

```text
Status: succeeded | failed | canceled
ExitCode: integer
ResultRef: artifact reference
LogRef: structured log reference
Digest: executed image digest
```

### 17.6 Artifact Manifest 契約（#302）

`JobType + InputRef -> ResultRef` を実運用するため、artifact本体とは別に
`artifact_manifest.json` を共通契約として扱う。

最小スキーマ（v1）:

```json
{
  "artifact_type": "toolpath|interference|generic",
  "format": "binary|json|custom",
  "format_version": "v1",
  "producer_job_id": 123,
  "image_digest": "sha256:...",
  "sha256": "...",
  "size_bytes": 1024,
  "created_at_utc": "2026-03-08T09:00:00Z"
}
```

責務分離:

- Job Manager: メタデータ契約の検証のみを行う
  - 必須項目の欠落
  - digest/hash/size の形式検証
  - `ResultRef` 不在時の reject
- Worker / Domain: artifact本体生成と `artifact_manifest.json` 作成を担う
- ViewModel / View: `ResultRef` / `LogRef` / manifest由来メタデータの表示に専念する

運用ルール:

- `format_version` 不一致は reject（非互換）を基本方針とする
- hash不一致は改ざんまたは破損として即失敗扱い
- 将来のNC/CAE拡張時も同一manifest契約を再利用する

### 17.4 #297着手時の実施順序（推奨）

1. Dockerfile作成（非root + multi-stage）
2. ローカルで最小ジョブ2種の動作確認（CAM/切削）
3. develop CIへDockerジョブ追加
4. 脆弱性スキャンとSBOMをCIゲート化
5. 受け入れ条件チェックリストをIssue #297に反映

### 17.5 設計上の保留事項

- ランナー実体を既存クレートに置くか、新規バッチ用クレートを作るか
- GPU依存ジョブを将来導入する場合のイメージ分離方針
- CIの実行時間増加に対するキャッシュ戦略

---

## 18. 参考スニペット（実装時の叩き台）

以下は設計意図を示す参考スニペットであり、そのまま本番適用する前に監査する。

### 18.1 Dockerfile（非root + multi-stage）

```dockerfile
FROM rust:1.86-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --workspace --release

FROM debian:bookworm-slim AS runtime
RUN groupadd -g 10001 redring && useradd -u 10001 -g 10001 -m -s /usr/sbin/nologin redring
WORKDIR /work
COPY --from=builder /src/target/release/redring-batch-runner /usr/local/bin/redring-batch-runner
RUN mkdir -p /work/output /work/logs && chown -R 10001:10001 /work
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/redring-batch-runner"]
```

注記:

- 現在ワークスペースに `redring-batch-runner` バイナリは未定義のため、実体配置は別途決定する
- rootfs read-only は実行基盤側（K8s/CI）で強制する

### 18.2 GitHub Actions ジョブ断片（非rootスモークテスト）

```yaml
docker-build-batch-runner:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Build image
      run: docker build -t redring-batch-runner:ci .

docker-smoke-test-nonroot:
  runs-on: ubuntu-latest
  needs: [docker-build-batch-runner]
  steps:
    - uses: actions/checkout@v4
    - name: CAM smoke test
      run: |
        docker run --rm \
          --user 10001:10001 \
          --read-only \
          -v ${{ github.workspace }}/tmp/input:/work/input:ro \
          -v ${{ github.workspace }}/tmp/output:/work/output \
          -v ${{ github.workspace }}/tmp/logs:/work/logs \
          redring-batch-runner:ci \
          --job-type cam --input /work/input/min_cam.json --output /work/output
```

### 18.3 Kubernetes securityContext 断片

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop: ["ALL"]
```

運用ルール:

- securityContext の緩和は例外申請制とし、恒久設定にしない
- 例外を入れる場合は理由・期間・代替策をIssueで明文化する

---

## 19. 最小版実装（現ブランチ）

本設計に対応する最小版として、以下を追加した。

- `Dockerfile.batch`
- `.dockerignore`
- `scripts/batch_runner_stub.sh`

ローカル確認例:

```bash
docker build -f Dockerfile.batch -t redring-batch-runner:local .

mkdir -p tmp/input tmp/output tmp/logs
echo '{"example":true}' > tmp/input/min_cam.json

docker run --rm \
  --user 10001:10001 \
  --read-only \
  -v "$PWD/tmp/input:/work/input:ro" \
  -v "$PWD/tmp/output:/work/output" \
  -v "$PWD/tmp/logs:/work/logs" \
  redring-batch-runner:local \
  --job-type cam --input /work/input/min_cam.json --output /work/output
```

確認ポイント:

- 非root UID/GID で実行されること
- `/work/output` に結果JSONが出力されること
- `/work/logs` に実行ログが出力されること

---

## 20. Dockerイメージタグ運用ルール（#297）

### 20.1 タグ体系

運用タグは次の3種類を使用する。

- `main-<short_sha>`
  - `main` ブランチ由来の継続タグ
  - 例: `main-4ee01ba`
- `develop-<short_sha>`
  - `develop` ブランチ由来の検証タグ
  - 例: `develop-4ee01ba`
- `release-<version>`
  - リリース固定タグ
  - 例: `release-v0.1.0`

補助タグ:

- `pr-<number>-<short_sha>`
  - PR検証専用の短期タグ
  - 例: `pr-299-77e2bde`

`latest` は再現性を下げるため運用しない。

### 20.2 digest pin原則

- 実行時はタグ参照ではなく `image@sha256:<digest>` を使用する
- Job Manager は `ImageRef` に digest を保持する
- タグは人間向けの識別子、実行同定は digest を正とする

### 20.3 ブランチ別運用

1. develop CI
- イメージをビルド
- `develop-<short_sha>` を付与
- digest をアーティファクト保存

2. main CI
- イメージをビルド
- `main-<short_sha>` を付与
- digest をアーティファクト保存

3. release作業
- `release-<version>` を付与
- リリースノートに digest を記録

### 20.4 保持・クリーンアップ

- `pr-*` タグ: 14日保持
- `develop-*` タグ: 30日保持
- `main-*` タグ: 90日保持
- `release-*` タグ: 恒久保持

削除時も digest とジョブ履歴の参照情報は監査期間中保持する。

### 20.5 監査・追跡

- すべてのジョブ結果に `image_digest` を記録する
- ログ、結果アーティファクト、digest を同一 `job_id` に紐づける
- digest 未記録ジョブは失敗扱いとして再投入対象にする
