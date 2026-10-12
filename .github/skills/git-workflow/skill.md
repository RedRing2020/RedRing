# Git/GitHub ワークフロー

## 最終更新日: 2026年10月12日

RedRing における Git 操作・コミット・PR 作成の手順です。運用ルールの正本は `dev/GIT_PR_WORKFLOW_OPERATION.md` で、この手順書と食い違う場合は正本を優先します。

---

## ブランチ戦略

```
main (リリース版) ← develop (開発版) ← feature/<topic>（作業ブランチ）
```

- **main**: リリース済みの安定版
- **develop**: 開発中の最新版。直接実装・コミットしない
- **feature/<topic>**: 作業ブランチ。Issue がある場合は `feature/issue-<番号>-<topic>` とする

---

## 作業の開始

```bash
git checkout develop
git pull --ff-only origin develop
git status -sb   # develop が ahead のまま次の作業へ進まない
git checkout -b feature/issue-123-sphere-primitive
```

---

## コミット

### 形式

```
<type>(<scope>): <要約> (#<Issue番号>)

- 変更点
- 変更点
```

- `type`: `feat` / `fix` / `refactor` / `docs` / `test` / `chore`
- `scope`: 対象のクレート・領域（`geo_primitives` / `scripts` / `vscode` 等）。複数にまたがる場合は省略してよい
- 要約・本文は日本語で書く

**例**:
```
refactor(geo_core): 点・ベクトル・方向・相似変換・AABB を親子のモジュール構成にする (#823)

- 10 個の型の拡張・テストを、型と同名のディレクトリの子モジュールに移した
- モジュールを private にし、型だけを lib.rs から再エクスポートする
```

### 注意

- pre-commit フックがフォーマットを確認する。フック・署名を省略するオプション（`--no-verify` 等）は使わない
- ソースコード（コメントを含む）には Issue / PR の番号などの GitHub 固有の情報を書かない（`scripts/check_github_refs_in_source.ps1` が検査する）。コミットメッセージ・PR 本文には書いてよい

---

## PR 作成

### 1. 事前チェック

PR 作成前と、PR 作成後に追加コミットを push する前に、必ず実行してすべて通ることを確認する。

```powershell
pwsh -NoProfile -File .\scripts\check_all_before_push.ps1
```

- 検査項目: fmt / clippy / テスト / 実行時文言の言語ポリシー / ソースコード中の GitHub 固有情報 / 呼び出し元のない公開メソッド・trait定義のメソッド（警告のみ）

### 2. タイトル

```
issue #<番号> [Phase<番号>][<系列>-<枝番>] <対象>: <要約>
```

- 例: `issue #815 [Design][B-2] geo_contracts: 直線系の trait定義の呼び出し元のないメソッドの削除`
- `Phase`（実施段階）と `PR系列`（変更単位）を混同しない（`dev/ISSUE_LABEL_OPERATION.md`）

### 3. 本文

本文の先頭に、この PR に含む単位と含めない単位を明記する。

```markdown
- 含む単位: B-2（直線系の trait定義のメソッドの削除）
- 含めない単位: B-3 以降（円・楕円系）は別 PR

## 概要
...

## 変更内容
...

## 確認
- `scripts/check_all_before_push.ps1` すべて通過

Closes #123
```

- 1 つの Issue を複数の PR に分ける場合、最後の PR 以外は `Refs #<番号>`、最後の PR は `Closes #<番号>` とする

### 4. 作成

```bash
git push -u origin feature/issue-123-sphere-primitive
gh pr create --base develop --title "..." --body-file pr_body.md
gh pr checks
```

- PR は必ず `develop` に向ける（`main` への PR は作らない）
- マージはユーザーが行う

---

## マージ後

```bash
git checkout develop
git pull --ff-only origin develop
git branch -d feature/issue-123-sphere-primitive
git push origin --delete feature/issue-123-sphere-primitive
```

---

## トラブルシューティング

### develop の変更の取り込み

```bash
git fetch origin develop
git merge origin/develop
# コンフリクトを解決したら
git add <解決したファイル>
git commit
```

- コンフリクトはどちらを採用するか方針を決めてから解決する

### 既に push したコミットの扱い

- 修正は新しいコミットで行う。`git commit --amend` や `git push --force` で履歴を書き換えない

---

## 参照文書

- **Git/PR の運用ルール（正本）**: `dev/GIT_PR_WORKFLOW_OPERATION.md`
- **Issue のラベル運用**: `dev/ISSUE_LABEL_OPERATION.md`
- **用語**: `dev/AI_TERMINOLOGY_GLOSSARY.md`
- **情報管理の方針**: `INFORMATION_MANAGEMENT_TRANSITION.md`
