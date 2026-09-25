# RedRing アーキテクチャ構成

**最終更新日**: 2026年4月3日

RedRing の幾何計算層とレンダリング層の構成、および現在の課題と解決策について説明します。

## 🧱 ワークスペース構成

### 幾何計算層

## 🚨 現在の重大な課題（2026年3月15日更新）

### 系統的な*_core_traits実装問題

10以上の形状で**レガシーAPIとFoundation実装が共存**し、以下の問題が発生しています：

- **メソッド名競合**: `center()`, `arc_length()`, `point_at_parameter()` 等
- **型不一致**: `Point2D<T>` vs `(T, T)` のシグネチャ違い
- **Foundation Patternの破綻**: 統一アクセスが実現できない

**対象形状**: point, vector, circle, ellipse_arc, direction, bbox, ray, line_segment, infinite_line

### アーキテクチャ再編提案（geo_foundation 廃止）

```text
foundation/analysis（将来改名候補）
            ↓
        geo_contracts（trait定義の正本）
          ↓        ↓
 geo_commons     geo_core（低レイヤー・基本型提供）
  （analysisのみ依存の   ↓             ↓
   数値カーネル）  geo_primitives      geo_nurbs
                      （impl入口）       （impl入口）
                            ↘           ↙
                    geo_algorithms（交差/衝突/幾何演算）
                    ↓
         application層クレート群
    （tessellation / simulation / job manager）

        CAM側の責務分離（設計更新）

            geo_*（汎用幾何演算）
                ↓
            cam_core（中立データ基盤）
               ↙          ↘
        cam_algorithms      cam_entity
        （CAM固有計算）     （表示/統合）
               ↓
              cam_sim
        （CAM特化ユースケース実行）
```

#### クレート責務定義

- **`foundation/analysis`**: 線形代数などの純粋な数値解析のみを提供（将来改名検討）
- **`geo_contracts`**: 幾何 trait定義の正本。shape definition core / extension / operations の公開境界を定義する
- **`geo_commons`**: `analysis` のみに依存する、再利用可能な幾何数値カーネル置き場。ellipse 近似や距離計算など、trait定義でも高レベル戦略でもない共通数値処理を置く
- **`geo_core`**: **低レイヤー基本型**（Aabb2D/Aabb3D等）- geo_primitives/geo_nurbsから直接アクセス可
- **`geo_primitives`**: プリミティブ形状の実装入口。`geo_contracts` の trait を実装し、必要に応じて `geo_commons` の数値カーネルへ薄く委譲する
- **`geo_nurbs`**: NURBS形状の実装入口。`geo_contracts` の trait を実装し、`geo_core` 経由で具体型へ接続する
- **`geo_algorithms`**: `geo_primitives` / `geo_nurbs` を利用した高レベル幾何アルゴリズム（intersect, collision 等）
- **`application::*`（新設方針）**: テセレーション、シミュレーション、ジョブ管理など業務ユースケース
    - Application Layer の主要責務は orchestration とし、入口API、委譲順序、境界DTO、port/adapter 切り替えを集約する
    - 詳細方針は `dev/architecture/APPLICATION_ORCHESTRATION_LAYER_DESIGN.md` を参照
- **`geo_io`**: ファイル I/O（STL/OBJ/PLY 等）
- **`cam_core`**: CAM の中立データ基盤（ToolPath / Tool / artifact I/O / 最小機械制約）
- **`cam_algorithms`**: CAM 固有アルゴリズム（経路生成、順序最適化、干渉回避、機械制約検証）。#684 で新設し、逆オフセット法による ToolPath 生成を実装
- **`cam_sim`**: CAM 特化の実行・ユースケース層（ToolPath 実行、除去量更新、結果キャッシュ）
- **`cam_entity`**: CAM 向け表示/属性統合層

#### レイヤー設計の重要ポイント

- **`geo_contracts` は境界定義層**: shape definition core / extension / operations の公開面は `geo_contracts` を正本とする
- **shape 意味論の正本は別文書**: 個別 shape の意味と API 意図は `dev/architecture/GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` を正本とし、`geo_contracts` の責務分離文書とは分けて管理する
- **`geo_commons` は存続対象**: `analysis` のみに依存する数値カーネル置き場として意図的に維持する。`geo_algorithms` の代替でも、廃止前提の移行対象でもない
- **`geo_core`は低レイヤー**: `geo_primitives`と`geo_nurbs`より下位に位置
- **直接アクセス許可**: `geo_core`からのインポート（特にAabb2D/Aabb3D）は許可
- **`geo_foundation`廃止（完了）**: 形状trait定義は`geo_contracts`へ統一済み
- **`geo_primitives` / `geo_nurbs` の役割**: 自 crate 型に対する impl entry point を担い、重い自由関数アルゴリズムは `geo_algorithms`、形状非依存の数値カーネルは `geo_commons` へ分離する
- **依存と import の区別**: `geo_algorithms -> geo_primitives/geo_nurbs` 依存は許可だが、`geo_algorithms` 実装ファイルでの `use geo_primitives::...` 直接 import は禁止（`use crate::...` 再エクスポート経由を使用）
- **`geo_topology` の曲線依存**: `geo_topology -> geo_primitives` は既存許可とし、curve edge の段階導入に限って `geo_topology -> geo_nurbs` 依存も許可する
- **NURBS topology 依存の制約**: 許可するのは `geo_topology -> geo_nurbs` の片方向のみとし、`geo_nurbs -> geo_topology` の逆依存は禁止する
- **上位責務分離**: tessellation/simulation/job managerは`geo_algorithms`より上位のapplication層へ集約
- **Application Layer の役割**: 非同期実装詳細そのものは持たず、同期/非同期/job投入の実行方式境界だけを管理する
- **CAM責務分離**: `cam_core` は中立データ、`cam_algorithms` は計算ロジック、`cam_sim` はCAM特化ユースケース実行として分離する
- **`cam_sim` の位置づけ**: 物理配置は `model/` 配下だが、責務としては純粋データ層ではなく CAM ドメイン専用の application 層に近い
- **循環依存回避**: `geo_primitives` ↔ `geo_nurbs` の直接依存は禁止（交差処理は`geo_algorithms`に集約）

### 段階移行計画（提案）

1. **Phase 1: trait移設**
- `geo_foundation` の形状traitを `geo_contracts` へ移設
- 既存利用側を新trait参照へ置換

2. **Phase 2: アルゴリズム統合**
- `geo_algorithms` に `geo_primitives` と `geo_nurbs` の両依存を明示
- Intersect/Collision等のクロス形状演算を `geo_algorithms` に統一

3. **Phase 3: application層分離**
- tessellation/simulation/job manager を application層クレートに移動
- `geo_algorithms` は純粋幾何アルゴリズム責務に限定
- 初期導入は PoC から段階適用とし、初手は単一受け皿 + module 分割を優先する
- `*_orchestration` の独立クレート化は、責務境界と依存分岐が安定した段階で判断する

4. **Phase 4: analysis再編**
- `foundation/analysis` の名称変更を検討
- 内部実装を純粋数値解析（線形代数等）に整理し、ドメイン責務を排除

### CAD/CAM 境界ルール（2026年3月更新）

- **原則**: `model/geo_*` から `model/cam_*` への依存は禁止
- **CAM内部の基準依存**: `cam_algorithms -> cam_core` は許可、`cam_core -> cam_algorithms` は禁止
- **CAM内部の実行依存**: `cam_sim -> cam_core` は許可、`cam_sim -> cam_algorithms` は限定的に許可
- **CAM内部の表示依存**: `cam_entity -> cam_core` は許可、`cam_entity -> cam_algorithms` は禁止
- **禁止**: `cam_core -> geo_entity`
- **禁止**: `geo_core -> cam_entity`
- **許可**: `geo_core -> geo_entity`

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

### Artifact API 命名運用ルール（2026年3月更新, #415）

- 運用原則: `*_v1` は wire format v1.0 を意味せず、API 世代識別子を表す
- wire format 判定原則: `version_major` / `version_minor` の実値で判定する（現行 v0.1）
- 改名方針: 現時点では公開 API 互換を優先し、実コード改名は行わない

### 参照識別子・パス文字列運用ルール（2026年8月更新）

- 適用範囲: RedRing 全体（`model/` / `viewmodel/` / `view/` / `render/` / `stage/` / `scripts/`）
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

- 実行コマンド: `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError`
- CI/ローカルともに上記スクリプトで依存境界違反を検出する
- `cam_algorithms` 新設時は、workspace 参加、`scripts/_arch_rules_data.ps1` の依存ルール更新、関連ドキュメント更新を同一変更セットで行う

### 共有テスト補助コード配置方針（2026年7月更新）

- 共有テスト補助コードは、実際に同時利用している層の最小範囲で配置する
- include による共有テスト補助コードを model層内で共通利用する場合は、`model/test_helpers` のように用途が明確なディレクトリへ配置する
- 層横断で同一補助コードを継続利用する必要が生じた場合は、ワークスペース共通テスト補助クレート化を再評価する
- 再評価を採用する場合は、workspace 構成、依存境界ルール、関連設計書を同一変更セットで更新する

### 実行時文言の言語ポリシー（2026年8月更新）

- 適用範囲: `model/` 配下の実行時エラー文言（`Display` 実装など）と実行時ログ文言
- 方針: 上記文言は英語で統一する
- 非対象禁止: `model/` がドメイン層であることを理由に対象外扱いしない
- 日本語表示要件: UI 向け文言は `i18n_foundation` のメッセージカタログ経由で提供する
- 例外手続き: 例外を認める場合は「ユーザー明示承認 + 対応 Issue 番号」を必須とし、PR 本文へ理由を明記する
- 例外手続きの記載先: PR 本文と `dev/architecture/` 配下の設計書に限定する
- コード記載ルール: ソースコード本文・コメントへ GitHub 固有情報（Issue 番号、PR 番号、URL）を記載しない

## 🔧 修正方針

### レガシーAPIの段階的置き換え

1. **競合メソッドの内部化**: レガシー `pub fn` を `fn` に変更
2. **Foundation実装の真の移行**: 共存ではなく置き換えアプローチ
3. **最小限の実装から開始**: 1-2個のメソッドから段階的に実装
4. **geo_commons機能の活用**: 共通計算で内部実装を再利用

## geo_commons の位置づけ

### 現在方針

- `geo_commons` は廃止前提ではなく、`analysis` のみに依存する再利用可能な幾何数値カーネル層として維持する
- `geo_contracts` は trait定義の正本であり、数値アルゴリズム本体は持たない
- `geo_primitives` / `geo_nurbs` は自 crate 型の impl entry point を担い、必要に応じて `geo_commons` へ薄く委譲する
- `geo_algorithms` は cross-shape 演算や高レベル戦略を置く層であり、`geo_commons` の置き換えではない

### 置くもの

- ellipse 周長近似、焦点、離心率のような shape 非依存の数値式
- ellipse 距離計算のような、具体 shape 型を要求しない共通数値カーネル
- trait定義や orchestration を含まない、純粋関数ベースの幾何補助計算

### 置かないもの

- trait定義そのもの
- cross-shape capability の公開契約
- shape 実装の entry point
- heavy strategy / solver orchestration

この位置づけを正本とし、`geo_commons` 廃止前提で書かれた旧設計メモがある場合は、個別 Issue で現行方針へ更新する。

### 形状API統一の実現（新方針）

**目標**: 全てのアクセスを「各形状クレート内trait」経由に統一

```rust
// ✅ 目標: 形状クレート内trait経由のみアクセス可能
use geo_primitives::EllipseArc2DDerived;

let arc = EllipseArc2D::new(...);
let length = arc.length(); // trait実装を呼び出し
let point = arc.point_at_parameter(0.5); // trait実装を呼び出し

// ❌ 禁止: レガシー直接アクセス
// arc.legacy_method() // コンパイルエラー
```

### レンダリング層

```text
redring ← stage ← render
       ↖ viewmodel
```

#### アプリケーション層の責務

- **`render`**: GPU 描画基盤（wgpu + WGSL）- 基本描画機能のみ
- **`stage`**: レンダリングステージ管理 - 最小限の構造のみ
- **`viewmodel`**: ビュー操作・変換ロジック - 基礎機能のみ
- **`redring`**: メインアプリケーション - ウィンドウ表示のみ

**現在未実装の主要機能**:

- メニューシステム
- コマンドパレット
- ファイル操作（開く/保存）
- WebAssembly対応
- 実用的なCAD/CAM機能

## 🔄 f64正準化移行について

- **基本方針**: Vector/Point は f64 正準型、測定量は Scalar `<T>` 維持
- **Legacy型**: 全て削除済み、CI で deprecated symbols を deny
- **詳細履歴**: `MIGRATION_VECTOR_F64.md` を参照

## 📝 重要な教訓

1. **共存アプローチの失敗**: レガシーとFoundationの共存はメソッド名競合を引き起こす
2. **置き換えの必要性**: Foundation Patternの真の価値は統一アクセスにある
3. **段階的実装の重要性**: 一度に多数のメソッドを実装すると失敗する

## 🎆 期待される成果

- **統一アクセス**: 全てのAPIが各形状クレートのtrait経由
- **型安全性**: コンパイル時のインターフェース統一
- **保守性向上**: 明確な責務分離と依存関係
- **拡張性**: 新しい形状追加は各形状クレート、クロス演算は`geo_algorithms`に集約

## 🔗 関連ドキュメント

- **[📖 オンラインドキュメント](https://redring2020.github.io/RedRing/)** - GitHub Pages（自動更新）
- [`dev/architecture/APPLICATION_ORCHESTRATION_LAYER_DESIGN.md`](dev/architecture/APPLICATION_ORCHESTRATION_LAYER_DESIGN.md) - Application Layer / orchestration 層の責務定義
- [`dev/architecture/FIXTURE_SEPARATION_DESIGN.md`](dev/architecture/FIXTURE_SEPARATION_DESIGN.md) - Model fixtures 分離方針（Issue #516）
- [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md) - 幾何抽象化の詳細仕様
- [`manual/philosophy.md`](manual/philosophy.md) - 設計思想・エラー処理ガイドライン
- [`MIGRATION_VECTOR_F64.md`](MIGRATION_VECTOR_F64.md) - f64 正準化移行履歴
- [`GITHUB_PAGES_SETUP.md`](GITHUB_PAGES_SETUP.md) - GitHub Pages 設定ガイド
- [`dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md`](dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md) - 夜間バッチ計算基盤（Dockerヘッドレス + Kubernetes）
- [`dev/architecture/GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md`](dev/architecture/GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md) - geo_algorithms の分割ルール（primitive_2d/3d/NURBS/pair_base の統一規約）
- [`dev/archive/issues/issue-412-artifact-api-naming-archive-note.md`](dev/archive/issues/issue-412-artifact-api-naming-archive-note.md) - artifact API命名整理の判断記録

