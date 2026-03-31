# ViewModel-Model導線再定義設計

**作成日**: 2026年3月31日
**関連Issue**: #500
**関連Issue（実装/デモ）**: #502, #214, #215
**選択方針**: A（Application最小port追加）

## 1. 目的

Entity/Topology追加後の責務境界を明確化し、ViewModelからModelへの最小導線を定義する。

本設計のゴールは以下。
- ViewModel -> Modelの呼び出し境界を固定する
- Entity/Topologyの生成・更新責務をModel側へ集約する
- 非ECS実装からECS実装への差し替え点を先に定義する

## 2. 前提と非スコープ

前提:
- #502 で操作1本の最小E2Eデモを成立させる
- #214（切削シミュレーション）をデモ対象文脈として扱う
- #215（ECS）は #502 のデモリリース後に着手する

非スコープ:
- ECS本実装
- 大規模なクレート再編

## 3. 選択方針A（採用）

方針A:
- ApplicationにViewModel向け最小port traitを1本追加する
- ViewModelは操作コマンドを渡し、結果DTOと失敗DTOを受ける
- Entity/Topology更新はModel側ユースケースが実施する

採用理由:
- #502向けに最短でE2E導線を固定できる
- 既存構成への変更を最小化できる
- #215でbackend差し替え（ECS化）しやすい

### 3.1 導線確保の具体的手法（明示）

本設計で採用する導線確保手法は、**Ports and Adapters（Hexagonal Architecture）** とする。

実装ルール（今後の追加導線でも共通）:
- Port trait定義はApplication層に置く
- Port trait実装はApplication層のorchestration/adapterに置く
- Port実装の内部でModel（geo_entity / geo_topology等）のAPIを呼ぶ
- ViewModelはPort呼び出しにのみ依存し、Model詳細に依存しない

禁止事項:
- Modelクレート側にApplicationのPort trait実装を置かない
- ViewModelからModelクレートの具象型/具象処理へ直接依存しない

このルールにより、導線追加時の判断は「PortをApplicationに追加し、実装をApplicationに置く」を標準手順とする。

## 4. 責務マップ

ViewModel:
- UIイベントの受理
- Application port呼び出し
- 成功/失敗のUI反映

具体説明:
- 入力: ユーザー操作（コマンド、選択、パラメータ）
- 出力: Application向けrequest DTO、View向け表示状態更新
- やること: 入力妥当性のUIレベル検証、表示用メッセージキー解決、画面状態遷移
- やらないこと: 幾何計算、Entity/Topologyの直接更新、永続化判断

Application:
- ユースケース境界
- Entity/Topology更新オーケストレーション
- 返却DTO正規化

具体説明:
- 入力: ViewModelからのrequest DTO
- 出力: 成功response DTOまたはApplicationError
- やること: ユースケース単位のトランザクション境界管理、Model呼び出し順序制御、失敗の正規化
- やらないこと: 画面表示文言の最終決定、GPU描画データ詳細生成

Model（geo_entity / geo_topology）:
- Entity/Topologyの生成・更新・参照
- 一貫性ルールの維持

具体説明:
- 入力: Applicationが渡す操作パラメータ（ドメイン型）
- 出力: 更新済みドメイン状態、ドメインエラー
- やること: ID整合性維持、参照整合性維持、ドメイン不変条件の検証
- やらないこと: UI都合の変換、操作フローの分岐制御

View:
- 受け取った描画データの反映

具体説明:
- 入力: ViewModelが整形した表示データ
- 出力: 画面描画、ユーザーへの視覚フィードバック
- やること: レンダリング更新、選択/ハイライト反映
- やらないこと: ドメイン状態更新、ユースケース判断

### 4.1 判断ルール（迷ったときの基準）

- その処理が「業務ルール/幾何整合性」に関与するならModelへ置く
- その処理が「操作手順の編成」に関与するならApplicationへ置く
- その処理が「表示都合の整形」に関与するならViewModelへ置く
- その処理が「GPU反映/描画」に関与するならViewへ置く

補助チェック:
- ViewModel変更だけでユースケース成立条件が変わるなら設計誤配置の可能性が高い
- Model変更でUI文言だけが壊れるなら責務混在の可能性が高い

## 5. 最小導線（非ECS）

1. ViewModelが操作コマンドDTOを生成
2. Application portへ渡す
3. Application実装が既存の管理構造を用いてEntity/Topologyを更新
4. ApplicationがRender反映用DTOと更新結果DTOを返す
5. ViewModelが成功/失敗をUIへ反映

## 6. 将来置換方針（ECS）

- 置換対象はApplication portの実装のみ
- ViewModel側の呼び出し契約は維持する
- 非ECS backendとECS backendで同一port traitを満たす

## 7. エラー境界方針

- Application側でドメイン固有エラーをApplicationErrorへ正規化
- ViewModelで表示キーへ変換
- converterごとの個別エラー列挙を段階的に収束させる

### 7.1 ID境界方針（Phase2追記）

- Entityの内部同一性はModel側の型付きID（`geo_entity::EntityId`）を正本とする
- Application境界DTOではIDを文字列表現で返却してよい
- ただし、フィーチャ再実行でIDが変化しないことを優先し、`new_random` の常用は避ける
- フィーチャ管理対象の生成では `from_feature_output(feature_id, output_index, local_key)` を優先する

補助ルール:
- 画面表示・ログ・外部I/Oは文字列IDを用いる
- ドメイン内部比較・整合性検証は型付きIDで行う

## 8. 受け入れ条件（#500）

- ViewModel -> Model最小導線が文章定義されている
- Entity/Topology責務境界が明文化されている
- #500完了後に着手する実装Issueが分解されている

## 9. 実装Issue分解案

Phase1:
- 最小port trait/DTO導入

Phase2:
- Entity/Topology更新ユースケース実装
- 成功/失敗境界の統一

Phase3:
- #502の操作1本E2Eデモ実装
- 成功/失敗確認手順を整備

## 10. 進め方

1. 本ドキュメントを#500の設計正本としてレビュー
2. 合意後に実装Issueへ分解して起票
3. #502を先行実施（最小実装 + デモリリース）
4. 実測結果を入力に #215 のECS評価/実装へ進む

## 11. Phase3 最小E2Eデモ手順（#509）

デモ対象操作:
- `p` キーで `load_sample_toolpath` を実行し、ViewModel -> Application -> Model更新結果DTO -> View反映を確認する

成功ケース確認:
1. アプリ起動後に `p` キーを押す
2. CAMシミュレーション可視化が表示されることを確認する
3. ログに `entity_id=` を含む完了メッセージが出ることを確認する

失敗ケース確認:
1. `cargo test --workspace cam_sim_visualization_converter::tests::test_create_cam_simulation_visualization_bundle_for_demo_failure_empty_toolpath`
2. 空ToolPathシナリオで `ApplicationError::Simulation` が返ることを確認する

観測ポイント:
- 成功時: `CamSimulationVisualizationBundle.tool_entity_id` が非空
- 失敗時: Application境界で失敗が正規化され、ViewModelで `CamSimulationVisualizationError::Application` として観測できる
