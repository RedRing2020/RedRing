# RedRing アーキテクチャ構成

**最終更新日**: 2026年10月12日

RedRing のレイヤー構成、クレートの責務と依存の規則を定義する。クレートの依存の検査は `scripts/check_architecture_dependencies.ps1` と、そのルール `scripts/_arch_rules_data.ps1` で行う。

## 🧱 レイヤー構成

| ディレクトリ | レイヤー | 役割 |
| --- | --- | --- |
| `foundation/` | 基盤 | ドメインに依存しない数値計算・ログ・ローカライズ |
| `model/` | モデル | 幾何（`geo_*`）・CAM（`cam_*`）・ジョブ（`job_*`）・ユースケースの orchestration（`application`） |
| `viewmodel/` | ビューモデル | モデルのデータを描画用のデータに変換する |
| `view/` | ビュー | GPU 描画・ステージ・アプリケーション |

```text
view（render / stage / app）
  ↓
viewmodel（converter / graphics / cam_demo）
  ↓
model
  application → cam_sim → cam_algorithms → cam_core
  application → geo_algorithms → geo_primitives / geo_nurbs → geo_contracts → geo_core
  application → geo_topology → geo_primitives / geo_nurbs
  geo_primitives → geo_commons
  geo_entity / geo_io → geo_primitives
  cam_sim / application → job_domain → job_runtime
  ↓
foundation（analysis / logging_foundation / i18n_foundation）
```

各クレートに許可された依存の正本は `scripts/_arch_rules_data.ps1` とする。

### クレート責務定義

- **`foundation/analysis`**: 線形代数などの純粋な数値解析・抽象型・許容誤差
- **`foundation/logging_foundation`** / **`foundation/i18n_foundation`**: ログの初期化・レート制御 / ローカライズのメッセージカタログ
- **`geo_contracts`**: 幾何の trait定義の正本（抽象化）。形状ごとの capability（core）・形状に共通の capability（foundation）・形状をまたぐ演算（operations）の公開境界を定義し、`geo_core` の値型で型付けし、その値型を再エクスポートする。形状の具体型と実装を持たない
- **`geo_commons`**: `analysis` のみに依存する、形状の型に依存しない共通の数値計算（楕円・楕円体の最近点と距離等）
- **`geo_core`**: 最下位の幾何の値型（点・ベクトル・方向・相似変換・AABB・点の分類）と、許容誤差の既定値・変換のエラー。`analysis` のみに依存し、trait定義を持たず、操作は固有メソッドで提供する
- **`geo_primitives`**: 解析形状の実装。`geo_contracts` の trait定義を実装し、必要に応じて `geo_commons` の計算へ委譲する
- **`geo_nurbs`**: NURBS 曲線・曲面の実装。`geo_contracts` の trait定義を実装し、`geo_core` の型を使う
- **`geo_topology`**: 位相構造（エッジ・ワイヤ・複合曲線等）
- **`geo_algorithms`**: `geo_primitives` / `geo_nurbs` を使う、形状をまたぐ演算（距離・交差・衝突・テセレーション）
- **`geo_entity`**: 業務に依らない CAD のエンティティ基盤（識別子・属性・関係）。業務（CAM 等）の属性・表示の定義は持たない
- **`geo_io`**: ファイル I/O（STL / OBJ / PLY 等）
- **`cam_core`**: CAM の中立データ基盤（ToolPath / Tool / artifact I/O / 最小の機械制約）
- **`cam_algorithms`**: CAM 固有のアルゴリズム（経路生成、順序最適化、干渉回避、機械制約の検証）
- **`cam_sim`**: CAM 特化の実行・ユースケース（ToolPath の実行、除去量の更新、結果のキャッシュ）
- **`cam_entity`**: CAM 向けの表示・属性の統合（CAM の業務の属性は、ここで定義する）
- **`job_runtime`** / **`job_domain`**: CAD / CAM / CAE に依存しない実行制御 / ジョブ投入のポリシー
- **`application`**: テセレーション・シミュレーション・ジョブ投入などのユースケースの orchestration（`APPLICATION_ORCHESTRATION_LAYER_DESIGN.md`）
- **`test_support`**（`redring_test_support`）: 共有のテスト補助
- **`viewmodel/*`**: モデルのデータの描画用への変換（`converter`）、カメラ等のグラフィックス制御（`graphics`）、CAM のデモ導線（`cam_demo`）
- **`view/*`**: GPU 描画基盤（`render`）、レンダリングステージ（`stage`）、メインアプリケーション（`app`）

### レイヤー設計の重要ポイント

- **`geo_contracts` は境界定義層**: 形状の trait定義の公開面は `geo_contracts` を正本とし、具体型のクレートは trait定義を再エクスポートしない
- **shape 意味論の正本は別文書**: 個別の形状の意味と API の意図は `GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` を正本とし、trait定義の構成（`GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md`）とは分けて管理する
- **API は最低限から拡張**: 呼び出し元が必要になった時点で API を追加し、呼び出し元のない API・別名・簡易実装を置かない（`GEO_API_ADDITION_RULES.md`）
- **`geo_commons` は存続対象**: `analysis` のみに依存する共通の数値計算の置き場として維持する。`geo_algorithms` の代替でも、廃止前提の移行対象でもない
- **抽象化と実装の分離**: `geo_contracts`（抽象化）→ `geo_core`（値型の実装）の向きに依存し、形状のクレートが `geo_core` の値型で `geo_contracts` の trait定義を実装する。`geo_core` の型の直接 import は許可する（`TYPED_GEO_CONTRACTS_DESIGN.md`）
- **`geo_primitives` / `geo_nurbs` の役割**: 自クレートの型に対する trait定義の実装の入口を担い、重い自由関数のアルゴリズムは `geo_algorithms`、形状の型に依存しない数値計算は `geo_commons` へ分離する
- **依存と import の区別**: `geo_algorithms -> geo_primitives/geo_nurbs` の依存は許可だが、`geo_algorithms` の実装ファイルでの `use geo_primitives::...` の直接 import は禁止（`use crate::...` の再エクスポート経由を使う）
- **`geo_topology` の曲線依存**: `geo_topology -> geo_primitives` を許可し、曲線のエッジの段階導入に限って `geo_topology -> geo_nurbs` も許可する。`geo_nurbs -> geo_topology` の逆依存は禁止する
- **循環依存の回避**: `geo_primitives` と `geo_nurbs` は互いに直接依存しない（形状をまたぐ処理は `geo_algorithms` に集約する）
- **上位責務の分離**: テセレーション・シミュレーション・ジョブ管理の orchestration（処理の順序と実行方式の選択）は `geo_algorithms` より上位の application 層へ集約する。ViewModel が描画用の変換で計算を直接呼ぶことは含まない（下記の ViewModel → Model の経路）
- **Application Layer の役割**: 非同期の実装詳細そのものは持たず、同期 / 非同期 / ジョブ投入の実行方式の境界だけを管理する
- **CAM の責務分離**: `cam_core` は中立データ、`cam_algorithms` は計算ロジック、`cam_sim` は CAM 特化のユースケースの実行として分離する
- **`cam_sim` の位置づけ**: 物理配置は `model/` 配下だが、責務としては純粋なデータ層ではなく CAM ドメイン専用の application 層に近い
- **エンティティ層の業務分離**: 依存は `geo_*` → `geo_entity` → 業務別の entity（`cam_entity` 等）の向きとする。属性コードのドメインは、業務に依らないもの（メタ・寸法等）を `geo_entity`、業務のもの（CAM）を業務層が所有する（`ENTITY_ATTRIBUTE_HIERARCHICAL_ID_PROPOSAL.md`）
- **ViewModel → Model の経路**: 更新とユースケースの実行は application 経由、描画用の変換と表示確認用のデータは Model の値型・描画用の計算を直接使ってよい（`VIEWMODEL_MODEL_ROUTE_REDEFINITION_DESIGN.md` §3.4.1）
- **ビューはモデルに直接依存しない**: `view/*` はモデルのクレートに依存せず、描画に必要なデータは `viewmodel/*` で変換したものを受け取る

### CAD/CAM 境界ルール（2026年3月更新）

- **原則**: `model/geo_*` から `model/cam_*` への依存は禁止
- **CAM内部の基準依存**: `cam_algorithms -> cam_core` は許可、`cam_core -> cam_algorithms` は禁止
- **CAM内部の実行依存**: `cam_sim -> cam_core` は許可、`cam_sim -> cam_algorithms` は限定的に許可
- **CAM内部の表示依存**: `cam_entity -> cam_core` は許可、`cam_entity -> cam_algorithms` は禁止
- **禁止**: `cam_core -> geo_entity`
- **禁止**: `geo_core -> cam_entity`

#### CAMクレート責務の補足

- **`cam_core`**: 他CAMクレートから参照される中立データのみを保持する
- **`cam_algorithms`**: 生成・最適化・検証・干渉回避などの CAM 計算を集約する
- **`cam_sim`**: シミュレーション実行を担う。`cam_algorithms` への依存は実行前後の問い合わせに限定する
- **`cam_entity`**: UI/可視化向け属性統合を担い、計算ロジックは保持しない

### 共通ジョブマネージャー方針（2026年3月更新）

- **新規共通クレート**: `model/job_runtime`
- **目的**: CAD/CAM/CAEに依存しない実行制御（submit/status/cancel/retry）を提供
- **責務**: `JobType`/`JobStatus`/`RetryPolicy`、状態遷移、イベント、Artifact参照契約
- **非責務**: CAM計算・切削シミュレーション等のドメイン計算ロジック本体
- **依存方針**:
    - `job_runtime` は `geo_*` / `cam_*` / View系へ依存しない
    - `cam_*` / 将来のCAE側は `job_runtime` をアダプタ経由で利用する

## 📏 運用ルール

### Artifact API 命名運用ルール（2026年3月更新, #415）

- 運用原則: `*_v1` は wire format v1.0 を意味せず、API 世代識別子を表す
- wire format 判定原則: `version_major` / `version_minor` の実値で判定する（現行 v0.1）
- 改名方針: 現時点では公開 API 互換を優先し、実コード改名は行わない

### 参照識別子・パス文字列運用ルール（2026年8月更新）

- 適用範囲: RedRing 全体（`foundation/` / `model/` / `viewmodel/` / `view/` / `scripts/`）
- 対象: 参照識別子として扱う文字列全般
    - `input_ref` / `result_ref` / `log_ref`
    - artifact 参照ID、ジョブ関連参照、追跡キー
    - 今後追加される `*_ref` / `*_path` 系の契約文字列

#### 基本原則

- 直書き禁止: 本番コードで参照文字列を都度 `format!` やリテラルで組み立てない
- 生成の一元化: 参照文字列は専用生成関数またはファクトリを経由して作成する
- 検証の一元化: 参照文字列の妥当性検証は専用バリデータへ集約する
- 責務分離: 参照形式の検証と payload 本体の解釈責務を分離する

#### 参照スキーム運用

- 既定スキーム:
    - `input://`
    - `result://`
    - `log://`
- 新規スキーム追加時の要件:
    - 設計書へ追加理由、構文、生成関数、検証関数、互換方針を明記する
    - 受理側の失敗時挙動（reject理由、エラー分類）を先に定義する

#### 命名・フォーマット規約

- 参照識別子は ASCII を基本とする
- 区切りは `/` を使用し、空セグメントを許可しない
- 意味分類（例: `invalid_input` / `no_solution` / `convergence_failure`）と追跡キー（`log_ref`）を同一視しない
- 可視化文言と契約キーを分離し、契約キーの意味定義は設計書を正本とする

#### 実装規約

- 参照文字列は value object（newtype）または生成API経由で扱う
- 外部境界（I/O、Job投入、永続化）では必ずバリデーションを通す
- 直接比較は最小化し、型またはヘルパー関数で比較する

#### 参照型とファクトリの共通設計

- 目的:
    - 文字列直書きの乱立を防ぎ、参照契約を型で固定する
    - CAM専用実装に閉じず、RedRing 全体で再利用できる共通資材にする

- 共通型（最小セット）:
    - InputRef
    - ResultRef
    - LogRef
    - RefValidationError

- 共通ファクトリ（最小セット）:
    - RefFactory::input(...)
    - RefFactory::result(...)
    - RefFactory::log(...)
    - RefParser::parse_input(...)
    - RefParser::parse_result(...)
    - RefParser::parse_log(...)

- 型の責務:
    - 生成時にスキームとセグメントを検証し、不正値を拒否する
    - 表示文字列化は as_str 相当の読み取り専用APIに限定する
    - 文字列結合による後加工を禁止し、必要な派生はファクトリ経由で再生成する

- ファクトリの責務:
    - スキーム差分を隠蔽し、呼び出し側が prefix を意識しないようにする
    - エラー分類を RefValidationError に正規化して返す
    - 新規スキーム追加時の変更点をファクトリ層へ閉じ込める

- 配置方針:
    - 実装は job_runtime 直下または専用共通クレートへ配置し、CAM以外からも参照可能にする
    - cam_sim / application / 将来のCAE系は同一APIを利用し、独自実装を持たない

- 境界インターフェース設計:
    - 生成境界: `RefFactory` が `InputRef` / `ResultRef` / `LogRef` を生成する
    - 受理境界: `RefParser` が外部入力を parse し、型へ昇格させる
    - 検証境界: `RefValidationError` を単一の失敗表現として返す
    - 利用境界: 業務ロジック層は参照型のみを受け取り、生文字列を受け取らない

- データモデル設計:
    - `InputRef` / `ResultRef` / `LogRef` は不変の value object とする
    - 内部表現は opaque とし、直接の文字列編集を禁止する
    - 出力は `as_str` 相当の読み取り専用インターフェースに限定する

- 不変条件（Invariant）:
    - 参照は必ず既知スキームで始まる
    - スキーム以降に空セグメントを持たない
    - ASCII 制約を満たす
    - 型生成時に不変条件を満たさない値は常に reject される

- エラーモデル設計:
    - 構文不正、未知スキーム、空セグメント、文字集合違反を `RefValidationError` で表現する
    - ドメイン失敗分類（`invalid_input` / `no_solution` / `convergence_failure`）とは責務を分離する
    - `log_ref` は追跡キーであり、失敗分類の意味モデルを兼務しない

- 拡張性設計:
    - 新規スキームは `RefFactory` / `RefParser` の拡張点として追加する
    - 既存利用側は型インターフェースを維持し、スキーム追加の影響を局所化する
    - 参照形式の版管理が必要な場合は、型の外側で互換判定ポリシーを持つ

- 互換性設計:
    - `String` ベース境界と参照型境界の併存を許容する
    - 変換は常に parse を経由し、暗黙変換を禁止する
    - 既存文字列契約との互換は、生成・解析APIの同値性で担保する

#### テスト規約

- テストでは可読性のため最小限の固定文字列を許可する
- 同一形式を3箇所以上で使う場合はテストヘルパーへ寄せる
- 本番コードが生成APIへ移行した箇所は、テスト側も同じ生成規約に追随する

#### 変更管理規約

- 既存直書きは一括置換しない
- 変更対象周辺から段階移行する
- 新規追加コードは初回から本規約を適用する
- 互換を壊す変更は、移行手順とロールバック方針を同一PR本文で提示する

#### 標準注記テンプレート

```text
注記: `*_v1` は API 世代識別子を表す。wire format の実バージョンは
`version_major` / `version_minor` の実値で判定する（現行は v0.1）。
```

#### 改名再評価トリガー

- `version_minor` 増加（例: v0.2）で API 名と wire format 名の混同が顕著化した場合
- `cam_sim` / 将来 `cam_algorithms` で命名誤解による障害が発生した場合
- 破壊的変更を許容できるリリース計画が確保された場合

### 依存チェック運用

- 実行コマンド: `pwsh -NoProfile -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError`
- CI / ローカルともに上記スクリプトで依存境界の違反を検出する
- ワークスペースのクレートと依存は `cargo metadata` から取得し、ルール（`scripts/_arch_rules_data.ps1`）は `Cargo.toml` のパッケージ名で書く
- ルールにないクレート・許可されていない依存はエラーになる。クレートを追加・変更するときは、ワークスペースの `members`、`[lints] workspace = true`、`scripts/_arch_rules_data.ps1`、この文書を同じ変更セットで更新する

### 共有テスト補助コード配置方針（2026年7月更新）

- 共有テスト補助コードは、実際に同時利用している層の最小範囲で配置する
- 層をまたいで継続利用する補助コードは、ワークスペース共通のテスト補助クレート（`model/test_support`、パッケージ名 `redring_test_support`）に置き、dev-dependencies で使う
- テストの置き場所は `TEST_PLACEMENT_RULES.md` に従う

### 実行時文言の言語ポリシー（2026年8月更新）

- 適用範囲: `model/` 配下の実行時エラー文言（`Display` 実装など）と実行時ログ文言
- 方針: 上記文言は英語で統一する
- 非対象禁止: `model/` がドメイン層であることを理由に対象外扱いしない
- 日本語表示要件: UI 向け文言は `i18n_foundation` のメッセージカタログ経由で提供する
- 例外手続き: 例外を認める場合は「ユーザー明示承認 + 対応 Issue 番号」を必須とし、PR 本文へ理由を明記する
- 例外手続きの記載先: PR 本文と `dev/architecture/` 配下の設計書に限定する
- コード記載ルール: ソースコード本文・コメントへ GitHub 固有情報（Issue 番号、PR 番号、URL）を記載しない
    - 検出: `scripts/check_github_refs_in_source.ps1`（`check_all_before_push.ps1` と CI のアーキテクチャチェックで実行）
    - 経緯の参照が必要な場合は、Issue 番号ではなく設計書名と節（例: `設計: CAM_ALGORITHMS_DESIGN.md §8`）で記載する

## geo_commons の位置づけ

### 現在方針

- `geo_commons` は廃止前提ではなく、`analysis` のみに依存する再利用可能な幾何の数値計算の層として維持する
- `geo_contracts` は trait定義の正本であり、数値アルゴリズム本体は持たない
- `geo_primitives` / `geo_nurbs` は自クレートの型の trait定義の実装の入口を担い、必要に応じて `geo_commons` へ委譲する
- `geo_algorithms` は形状をまたぐ演算や高レベルの戦略を置く層であり、`geo_commons` の置き換えではない
- 置くのは呼び出し元のある関数だけとし、使われなくなった関数は削除する

### 置くもの

- 形状の型に依存しない数値計算（楕円・楕円体の最近点と距離、線分と AABB の距離 等）
- trait定義や orchestration を含まない、純粋関数ベースの幾何の補助計算

### 置かないもの

- trait定義そのもの
- 形状をまたぐ capability の公開契約
- 形状の実装の入口
- 重い戦略 / ソルバーの orchestration

## レンダリング層

```text
redring（view/app） → stage → render
          ↓             ↓        ↓
              viewmodel（converter / graphics / cam_demo） → model
```

- **`render`**: GPU 描画基盤（wgpu + WGSL）。デバイス・パイプライン・シェーダ・頂点型・描画対象ごとの GPU リソース
- **`stage`**: レンダリングステージ（`RenderStage` の実装）
- **`viewmodel`**: モデルのデータを描画用のデータに変換する
- **`redring`**: メインアプリケーション（ウィンドウ・入力・ステージの組み立て）
- 実装パターンは `.github/skills/rendering/skill.md` を参照する

## 📐 座標系の規約

形状処理と表示の両方にまたがる規約として、次を正本とする。

1. **形状処理（`analysis` / `geo_*`）は常に右手系で扱う**
2. **回転の正の向きは反時計回り（右手の法則）とする**
   - 2D: 角度の増加方向が反時計回り
   - 3D: 法線・軸まわりに反時計回り（軸の先端から見て反時計回り）
   - 円・円弧の角度、`Arc3D` の掃引角もこの向きで測る
3. **メッシュの表面は反時計回りの巻き順（CCW）、法線は外向きとする**（[SHAPE_TESSELLATION_DESIGN.md](SHAPE_TESSELLATION_DESIGN.md)、描画パイプラインの `FrontFace::Ccw` と一致）
4. **表示はビュー・投影行列も右手系とする**
   - wgpu の NDC との違い（深度の向き・範囲）は投影行列（`perspective_rh_01` / `orthographic_rh_01`）の中で吸収する
   - 表示の見え方を理由に、カーネルの変換（`geo_core` の変換型・行列積の順序）や形状データを変更しない
5. **左手系の環境（ゲームエンジン等）へ渡す場合は、出力・表示の境界で 1 回だけ変換する**
   - 座標系の左右の切り替えはミラーリング（行列式が負）にあたるため、同じ境界で三角形の巻き順（表面の判定）と法線も反転させる
   - ミラーリングを含む変換の扱いは [#763](https://github.com/RedRing2020/RedRing/issues/763) で定める

## 🔗 関連ドキュメント

- **[📖 オンラインドキュメント](https://redring2020.github.io/RedRing/)** - GitHub Pages（自動更新）
- [`APPLICATION_ORCHESTRATION_LAYER_DESIGN.md`](APPLICATION_ORCHESTRATION_LAYER_DESIGN.md) - Application Layer / orchestration 層の責務定義
- [`GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md`](GEO_CONTRACTS_TRAIT_STRUCTURE_MINIMIZATION_DESIGN.md) - geo_contracts の trait定義の構成
- [`GEOMETRY_SHAPE_SEMANTICS_DESIGN.md`](GEOMETRY_SHAPE_SEMANTICS_DESIGN.md) - 形状の意味と API の意図
- [`GEO_API_ADDITION_RULES.md`](GEO_API_ADDITION_RULES.md) - 公開 API の追加の規則
- [`GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md`](GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md) - 形状のモジュール構成
- [`TEST_PLACEMENT_RULES.md`](TEST_PLACEMENT_RULES.md) - テストの配置
- [`GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md`](GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md) - geo_algorithms の分割ルール
- [`TRANSFORM_TRAIT_DESIGN.md`](TRANSFORM_TRAIT_DESIGN.md) - 変換の trait定義
- [`FIXTURE_SEPARATION_DESIGN.md`](FIXTURE_SEPARATION_DESIGN.md) - Model fixtures の分離方針
- [`BATCH_COMPUTE_PLATFORM_DESIGN.md`](BATCH_COMPUTE_PLATFORM_DESIGN.md) - 夜間バッチ計算基盤（Docker ヘッドレス + Kubernetes）
- [`manual/philosophy.md`](../../manual/philosophy.md) - 設計思想・エラー処理ガイドライン
- [`issue-412-artifact-api-naming-archive-note.md`](../archive/issues/issue-412-artifact-api-naming-archive-note.md) - artifact API 命名整理の判断記録
- [`issue-832-architecture-history-archive-note.md`](../archive/issues/architecture/issue-832-architecture-history-archive-note.md) - 完了した移行計画と、解消した課題の記録
