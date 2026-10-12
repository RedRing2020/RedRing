# アーキテクチャ概要

## 最終更新日: 2026年10月12日

RedRing のレイヤー構成と、クレート間の依存関係の汎用的な規則を定義します。個々のクレートの責務と依存の正本は `dev/architecture/ARCHITECTURE.md`（「クレート責務定義」「レイヤー設計の重要ポイント」）で、この手順書にはクレートの一覧を載せません。

---

## レイヤー構成

ワークスペースは、ディレクトリごとに次のレイヤーに分かれる。

| ディレクトリ | レイヤー | 役割 |
| --- | --- | --- |
| `foundation/` | 基盤 | ドメインに依存しない数値計算・ログ・ローカライズ |
| `model/` | モデル | 幾何（`geo_*`）・CAM（`cam_*`）・ジョブ・ユースケースの orchestration |
| `viewmodel/` | ビューモデル | モデルのデータを描画用のデータに変換する |
| `view/` | ビュー | GPU 描画・ステージ・アプリケーション |

各クレートがどのレイヤーに属し、何に依存してよいかは `ARCHITECTURE.md` と、依存関係のチェックスクリプトのルール（`scripts/_arch_rules_data.ps1`）で定義する。

---

## 依存関係の規則

### 向き

```text
view → viewmodel → model → foundation
```

- 依存は下位のレイヤーへの一方向だけとし、下位のレイヤーが上位のレイヤーに依存しない
- `foundation` のクレートはワークスペースの他のクレートに依存しない
- `view` はモデルのクレートに直接依存しない。描画に必要なデータは `viewmodel` で変換したものを受け取る
- クレートの間に循環する依存を作らない

### モデルの中

- 形状の trait定義は契約のクレート（`geo_contracts`）を正本とし、具体型のクレートはそれを実装する。trait定義のクレートは具体型を持たない
- 形状の型に依存しない共通の数値計算は、共通計算のクレート（`geo_commons`）に置き、形状のクレートで同じ計算を個別に実装しない
- 形状をまたぐ高レベルの計算は、アルゴリズムのクレート（`geo_algorithms`）に置く
- 幾何のクレート（`geo_*`）は CAM のクレート（`cam_*`）に依存しない（CAD / CAM の境界）
- ユースケースの orchestration（テセレーション・シミュレーション・ジョブ投入の組み立て）は application 層に置く

### クレートを追加・変更するとき

- ワークスペースの `members` に加える（`members` から漏れたクレートは `--workspace` のテスト・Clippy の対象にならない）
- `[lints] workspace = true` でワークスペースの lint を使う
- 依存関係のチェックのルール（`scripts/_arch_rules_data.ps1`）と `ARCHITECTURE.md` を、同じ変更の中で更新する

---

## アーキテクチャ検証

```powershell
.\scripts\check_architecture_dependencies.ps1
```

- レイヤーの依存の向き・禁止された依存・モデルのクレートの命名規則を検査する
- CI（`common_architecture_check.yml`）でも同じスクリプトを実行する

---

## 参照文書

- **クレートの責務と依存（正本）**: `dev/architecture/ARCHITECTURE.md`
- **MVVM 設計**: `dev/architecture/VIEWMODEL_ARCHITECTURE_DESIGN.md`
- **application 層**: `dev/architecture/APPLICATION_ORCHESTRATION_LAYER_DESIGN.md`
- **形状の意味**: `dev/architecture/GEOMETRY_SHAPE_SEMANTICS_DESIGN.md`
- **trait定義・形状の実装**: `.github/skills/foundation-pattern/skill.md`、`.github/skills/implementation/skill.md`

---

## 使用例コメントの配置方針（共通）

- 実装ファイル内の使用例は最小限に留め、詳細なサンプルは `manual/` に集約する
- import を含む長い例はコメントに埋め込まず、`manual/<topic>_examples.md` への参照に置き換える
- 使用例ページを追加した場合は `manual/SUMMARY.md` に必ず登録する
- 具体的な運用ルールは `.github/skills/implementation/skill.md` の「使用例コメントの配置ルール（CI整合）」に準拠する
