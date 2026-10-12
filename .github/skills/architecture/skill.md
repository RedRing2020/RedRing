# アーキテクチャ概要

## 最終更新日: 2026年10月12日

RedRing のワークスペース構成と、クレート間の依存関係のルールを定義します。

---

## Workspace 構成

```text
foundation/                 基盤（ドメイン非依存）
├── analysis                数値解析・線形代数・抽象型・許容誤差
├── i18n_foundation         ローカライズ
└── logging_foundation      ログ初期化・レート制御
model/                      モデル層
├── geo_contracts           形状の trait定義の正本
├── geo_commons             geo_* で共通する数値計算の関数
├── geo_core                点・ベクトル・方向・相似変換・AABB
├── geo_primitives          解析形状（直線・円・楕円・平面・二次曲面 等）
├── geo_nurbs               NURBS 曲線・曲面
├── geo_topology            位相構造（複合曲線・ワイヤ 等）
├── geo_algorithms          形状をまたぐ距離・交差・衝突・テセレーション
├── geo_entity              エンティティ基盤
├── geo_io                  ファイル I/O
├── cam_core                CAM 基盤（工具・工具経路）
├── cam_entity              CAM エンティティ
├── cam_algorithms          工具経路の生成
├── cam_sim                 切削シミュレーション
├── job_runtime             ジョブ管理の基盤
├── job_domain              ジョブ投入のポリシー
├── application             ユースケースの orchestration
└── test_support            共有のテスト補助（redring_test_support）
viewmodel/                  ビューモデル層
├── converter               モデル → 描画データの変換（パッケージ名 viewmodel）
├── cam_demo                CAM のデモ導線（viewmodel-cam-demo）
└── graphics                カメラ等のグラフィックス制御（viewmodel-graphics）
view/                       ビュー層
├── render                  GPU 描画基盤（wgpu + WGSL）
├── stage                   レンダリングステージ
└── app                     メインアプリケーション（パッケージ名 redring）
```

---

## 依存関係ルール

### 幾何のクレート

```text
analysis
  ├── geo_contracts ── geo_core ──┬── geo_primitives (+ geo_commons)
  │                               └── geo_nurbs
  └── geo_commons
geo_primitives, geo_nurbs → geo_topology → geo_algorithms
geo_primitives → geo_entity, geo_io
```

- `geo_contracts`: `analysis` のみに依存する。trait定義の正本で、具体型を持たない
- `geo_commons`: `analysis` のみに依存する。形状の型に依存しない共通の数値計算を置く
- `geo_core`: `analysis`、`geo_contracts` に依存する
- `geo_primitives`: `geo_contracts`、`geo_core`、`geo_commons`、`analysis` に依存する
- `geo_nurbs`: `geo_contracts`、`geo_core`、`analysis` に依存する。`geo_primitives` に直接依存しない
- `geo_topology`: `geo_primitives`、`geo_nurbs` まで依存できる
- `geo_algorithms`: 幾何のクレートすべてに依存できる

### CAM・ジョブ・アプリケーション

- `cam_core` → `geo_primitives`、`geo_algorithms`
- `cam_algorithms` → `cam_core`、`geo_algorithms`、`geo_contracts`
- `cam_sim` → `cam_algorithms`、`cam_core`、`geo_algorithms`、`job_runtime`、`job_domain`
- `application` → モデル層のクレート（ユースケースの orchestration）

### MVVM

```text
view (app / stage / render) → viewmodel (converter / graphics / cam_demo) → model
```

- `render` / `stage` はモデル層（`geo_*` / `cam_*`）に直接依存しない。描画に必要なデータは `viewmodel` で変換してから受け取る
- `app` もモデル層に直接依存しない
- 下位の層が上位の層に依存してはならない

---

## アーキテクチャ検証

```powershell
.\scripts\check_architecture_dependencies.ps1
```

- モデル層の命名規則、依存の向き、禁止依存（`geo_*` → `cam_*` 等）を検査する
- CI（`common_architecture_check.yml`）でも同じスクリプトを実行する
- 依存の一覧を手早く見る場合は `check_architecture_dependencies_simple.ps1` を使う

---

## 参照文書

- **詳細設計**: `dev/architecture/ARCHITECTURE.md`
- **MVVM 設計**: `dev/architecture/VIEWMODEL_ARCHITECTURE_DESIGN.md`
- **trait定義・形状の実装**: `.github/skills/foundation-pattern/skill.md`、`.github/skills/implementation/skill.md`

---

## 使用例コメントの配置方針（共通）

- 実装ファイル内の使用例は最小限に留め、詳細なサンプルは `manual/` に集約する
- import を含む長い例はコメントに埋め込まず、`manual/<topic>_examples.md` への参照に置き換える
- 使用例ページを追加した場合は `manual/SUMMARY.md` に必ず登録する
- 具体的な運用ルールは `.github/skills/implementation/skill.md` の「使用例コメントの配置ルール（CI整合）」に準拠する
