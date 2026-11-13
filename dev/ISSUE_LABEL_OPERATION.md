# Issueラベル運用ガイド

**作成日**: 2026年2月27日  
**更新日**: 2026年11月13日（ラベルの軸を決め直し、古いラベルを整理した。#553・#667）  
**対象**: RedRing GitHub Issues の日常運用

---

## 1. 目的

Issue の種別・かかわる層と領域・優先度・リリースの対象を、ラベルで一貫して表す。何を付ければよいかがぶれないように、軸ごとに使うラベルと付け方を決める。

---

## 2. 基本方針

- ラベルは軸（種別・層・領域・優先度・リリース・保留）のどれか 1 つに属する。この文書にないラベルは作らない
- 新しいラベルが要るときは、まずこの文書の軸に当てはめ、同じ意味のラベルがないかを確かめる。足すときは、この文書を同じ日に更新する
- 名前は小文字とハイフン（`cutting-simulation`）、優先度・リリースは今の書き方（`priority: high`・`release:in`）にそろえる。同じ意味の別名（`refactoring` と `refactor`、`Phase 3` と `phase-3` など）は作らない
- 一括で付け替えたときは、件数を確かめて Issue に残す

---

## 3. 軸とラベル

| 軸 | ラベル | 付け方 |
|---|---|---|
| 種別 | `bug`・`enhancement`・`design`・`refactor`・`tech-debt`・`documentation`・`investigation` | 必ず 1 つ |
| 層 | `foundation`・`model`・`viewmodel`・`view` | 変える層をすべて。CI・開発の環境など、層にかからなければ付けない |
| 領域 | `architecture`・`cam`・`cutting-simulation`・`geometry`・`job`・`ui`・`performance`・`security`・`ci` | 当てはまるものを任意の数 |
| 優先度 | `priority: high`・`priority: medium`・`priority: low` | 開いている Issue に必ず 1 つ |
| リリース | `release:in`・`release:out`・`release:triage` | 開いている Issue に必ず 1 つ |
| 保留 | `on-hold` | 前提がそろうまで進めないとき |
| Dependabot | `dependencies`・`github_actions`・`rust` | Dependabot が PR に付ける。Issue には付けない |

### 3.1 種別

| ラベル | 使うとき |
|---|---|
| `bug` | 不具合を直す |
| `enhancement` | 機能を足す・改善する（速くする・使いやすくするを含む） |
| `design` | 設計を決める（実装は別の Issue にする） |
| `refactor` | 振る舞いを変えずにコードを整理する |
| `tech-debt` | 依存・設定・決まり・検査などの負債を返す |
| `documentation` | 文書だけを変える |
| `investigation` | 調べて結論を出す（実装しない） |

### 3.2 領域

| ラベル | 使うとき |
|---|---|
| `architecture` | 層の構造・層をまたぐ仕組み（非同期化・コマンドの層など） |
| `cam` | 工程・加工手順・ツールパス・工具 |
| `cutting-simulation` | 切削シミュレーション（トリデクセル・削り込みの表示など） |
| `geometry` | 幾何・形状・トポロジー・NURBS・許容誤差 |
| `job` | ジョブマネージャー・ジョブの実行・ランナー |
| `ui` | 画面の操作・表示・メニュー・文言 |
| `performance` | 速さ・メモリ・画面を止めないこと |
| `security` | セキュリティ（保存先の権限・ファジング・署名など） |
| `ci` | CI・ビルド・ツールチェーン・開発の環境 |

### 3.3 優先度

- `priority: high`: 次の版に欠かせない、または利用者の作業を止める
- `priority: medium`: 次の版か、その次に行う
- `priority: low`: 前提が要るもの・先のもの

### 3.4 リリース

- `release:in`: 次に出す版に含める。次の版は `dev/architecture/RELEASE_VERSION_PLAN.md` 2. の表の、まだ出していない最初の版とし、その目標に直接かかわるものを `in` にする
- `release:out`: 次の版に含めない
- `release:triage`: まだ振り分けていない（一時の状態。作ってから 2 週間以内に `in`・`out` に振り分ける）
- 版を出したら、`release:in` で残った Issue を、次の版の目標で振り分け直す

---

## 4. 自動化

`.github/workflows/issue_labels.yml` が `scripts/check_issue_labels.ps1` を動かす。

- **Issue を作ったとき**: リリースのラベルがなければ `release:triage`、優先度がなければ `priority: medium` を付ける（中身を読まずに決まる既定だけ）
- **週に 1 回（月曜 9:00 JST）・手動**: 開いている Issue に同じ既定を付けたうえで、次の Issue を一覧にしてジョブの要約に出す。1 件でもあればワークフローを失敗にする（GitHub の通知で気づく）
  - 種別がない・2 つ以上ある
  - 優先度・リリースが 2 つ以上ある
  - この文書にないラベルが付いている
  - 作ってから 2 週間より長く `release:triage` のまま
- 種別・層・領域・優先度・リリースの中身は、Issue を読んで人（または Issue を作る AI）が決める。自動では付け替えない
- 手元で確かめるときは `pwsh -NoProfile -File scripts/check_issue_labels.ps1` を動かす（`-Fix` で既定を付ける）

---

## 5. 古いラベルの整理（2026年11月13日）

使わなくなった軸と、同じ意味のラベルを消した。付け替える先があるものは、閉じた Issue・PR も付け替えてから消した。

| 消したラベル | 付け替えた先 |
|---|---|
| `refactoring` | `refactor` |
| `type: bug` | `bug` |
| `area: design` | `design` |
| `status: on-hold` | `on-hold` |
| `CAM-visualization` | `cam` |
| `mvvm` | `architecture` |
| `foundation-pattern` | `foundation` |
| `entity` | `model` |
| `nurbs`・`geo_algorithms`・`geo_contracts`・`geo_primitives`・`collision-detection`・`area: tolerance`・`area: distance`・`area: intersection` | `geometry` |
| `camera`・`area: camera`・`area: viewing`・`graphics`・`visualization` | `view` |
| `tier-1`〜`tier-3`・`status:*`（準備中・設計完了・実装中・レビュー中）・`phase-1`〜`phase-4`・`phase-c`・`area: phase-c`・`Phase 3` | なし（使わなくなった軸） |
| `area: coverage`・`debug`・`future`・`outdated-architecture`・`needs-foundation-update`・名前の壊れた `release:triage` の重複 | なし |
| `duplicate`・`help wanted`・`invalid`・`question`・`wontfix`・`good first issue`（GitHub の既定） | なし（使っていない） |

- 段階を表すときは、ラベルではなく Issue を分ける（エピックと子の Issue）。PR の題名の決まりは `dev/GIT_PR_WORKFLOW_OPERATION.md` による
- 閉じた Issue の `release:*` はそのままにした（決まりは開いている Issue だけにかかる）

---

## 6. 更新ルール

- ラベルの体系を変えたら、この文書と `scripts/check_issue_labels.ps1` のラベルの一覧を同じ変更で更新する
- 変更の理由と影響（対象の Issue の数）を、PR か Issue のコメントに残す

### 関連ドキュメント

- `dev/architecture/RELEASE_VERSION_PLAN.md`（版の目標）
- `dev/foundation/GITHUB_PROJECT_SETUP_GUIDE.md`（Project の運用）
