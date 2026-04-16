# Git/PR運用ルール

最終更新: 2026-04-03

## 目的

- `develop` の履歴汚染と不要な競合を防ぐ
- PR前チェック漏れ（特に `cargo fmt --check`）を防ぐ

## 必須ルール

1. `develop` で直接実装しない

- 実装・修正・ドキュメント更新は必ず feature ブランチで実施

1. 作業開始前に同期確認

- `git checkout develop`
- `git pull --ff-only origin develop`
- `git checkout -b feature/<topic>`

1. `develop` が `ahead` のまま次作業へ進まない

- `git status -sb` で `ahead` を確認したら原因を解消してから作業開始

1. PR前の品質チェックを固定順序で実施

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

1. PR作成後の追いpush前にも同じ品質チェックを再実行

- 追加コミットを push する前に、必ず preflight を再実行する
- ルール対象: 「PR作成後の修正コミット」「レビュー指摘対応コミット」「fmt/clippy修正コミット」

1. Phase と PR系列の命名を固定する

- Issue / PR タイトルでは、次の2軸を必ず分離して表記する
	- `Phase`: 実施段階（いつ進めるか）
	- `PR系列`: 変更単位（何をまとめるか）
- `Phase` は進行段階であり、PR分割数とは独立とする
- 同一 `Phase` に複数の `PR系列` が存在してよい
- 同一 `PR系列` が複数 `Phase` に分割される場合は枝番で表す

1. タイトル表記フォーマットを固定する

- PRタイトルは次の形式を必須とする
	- `issue #<番号> [Phase<番号>][<系列>-<枝番>] <対象>: <要約>`
- 適用開始:
	- 2026-04-17 以降に新規作成するPRから必須適用する
	- 進行中PRは改名を推奨し、改名しない場合は本文先頭のスコープ明示を必須とする
- 例:
	- `issue #671 [Phase1][A-1] geo_nurbs: 実装棚卸しと方針固定`
	- `issue #671 [Phase2][A-2] geo_nurbs: zero tolerance入口統一`
	- `issue #671 [Phase3][B-1] geo_nurbs: 型別定数入口へ切替`

1. PR本文先頭でスコープを明示する

- PR本文の先頭に次を明記する
	- このPRに含む単位（例: `A-1 + A-2`）
	- このPRに含めない単位（例: `B-1 は別PR`）
	- `Phase` と `PR系列` の関係（段階と単位を混同しない説明）

1. 命名の変更を許可する条件を固定する

- レビュー開始後でも、次の場合はPRタイトルの改名を許可する
	- `Phase` と `PR系列` が読み取れない
	- `Phase` と `PR系列` の意味が混在している
- 改名時は本文先頭のスコープ明示も同時更新する

1. PRマージ後の後処理を固定

- `git checkout develop`
- `git pull --ff-only origin develop`
- `git branch -d <feature-branch>`
- 必要なら `git push origin --delete <feature-branch>`

## 推奨コマンド

```powershell
# PR前チェック（全項目）
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_pr_preflight.ps1

# 時短チェック（テスト省略）
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_pr_preflight.ps1 -SkipTests

# PR作成後の追いpush前チェック（必須）
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_pr_preflight.ps1
```

## 運用メモ

- `git pull` は原則 `--ff-only` を使う
- 競合発生時は安易に続行せず、どちらを採用するか方針を先に決める
- CIで `fmt` が落ちた場合は `cargo fmt --all` 実行後に再チェックする
- PR作成後の追加コミットでも、push前に `check_pr_preflight.ps1` を再実行する
- コメントや命名規則の用語は `dev/AI_TERMINOLOGY_GLOSSARY.md` を参照し、業界用語を優先する

## ローカル文書運用

- 正本は GitHub Issue / PR 本文とし、ローカル md は補助用途に限定する
- 一時的なチェックリスト、進捗ログ、テスト結果、完了報告は恒久 md としてコミットしない
- クローズ済み Issue 文書の archive 配置を CI 合否条件にしない

### 文書種別の制御ルール

- 正本:
	- Issue の背景、目的、完了条件、実施タスク、判断経緯の正本は GitHub Issue / PR 本文とコメントに置く
	- 実装と独立に長期参照される設計判断だけを `dev/architecture/` 配下の設計文書に残す
	- 複数 Issue で再利用する文書整理基準、分類基準、命名・配置の判断基準は運用書正本として repo に残してよい
- リポジトリに残してよい補助文書:
	- 複数 Issue / 複数 PR をまたいで参照される設計判断
	- 正本アーキテクチャ文書から参照される設計補助文書
	- クローズ後も判断履歴として残す価値がある archive 文書
	- 複数の過去文書を同じ基準で分類するための分類表テンプレート、分類カテゴリ定義、判定ルール
	- 一時的な作業方針書の作成条件、命名、寿命、後処理を定義する運用ルール
- 原則として残してはいけない文書:
	- Issue 本文の下書き
	- 着手前チェックリストだけを目的とした実施準備メモ
	- 一時的な棚卸し結果、進捗ログ、完了報告、テスト結果の貼り付け
	- 「その Issue の本文やコメントに書けば足りる内容」を重複記録した md

### 分類表の扱い

- 個別 Issue の分類結果そのものは作業成果物であり、まず Issue 本文・コメントまたは専用設計文書に整理する
- ただし、分類表の列定義、判定カテゴリ、判断ルール、記入例は再利用可能な運用資産として repo に残してよい
- その場合、個別の対象一覧を正本にしない
- 正本として残すのは次のような「分類の枠組み」だけとする
	- 分類カテゴリ: `残す / 統合 / archive / 削除`
	- 判定観点: 正本との重複、長期参照価値、履歴価値、複数 Issue 再利用性
	- 実施順: 正本化 → 参照置換 → archive / 削除
- 個別 Issue の実データを残す必要がある場合は、Issue 本文か、用途が継続する設計文書へ要約転記する

### 一時的な作業方針書の扱い

- 一時的な作業方針書そのものを無制限に許可しない
- 作成を許容するのは次の条件を全て満たす場合だけとする
	- 1 つの Issue / PR の本文だけでは判断順序や除外範囲を保持しきれない
	- 複数回の実施で同じ判断基準を参照する必要がある
	- 単なる進捗メモやチェックリストではなく、作業境界の固定が主目的である
- 一時的な作業方針書を作る場合のルール:
	- 1 Issue あたり原則 1 ファイルまで
	- `ISSUE_` 接頭辞を使わない
	- 用途が分かる名前を使う: `_POLICY.md` `_PLAN.md` `_SUMMARY.md`
	- 役割を冒頭で明記する: 一時方針 / 正本ではない / 後処理先
	- 作業完了時に `正本へ要約転記 / archive / 削除` のいずれにするかを決める
- 一時的な作業方針書を repo に恒久保存するのは、複数 Issue で再利用される運用ルールへ昇格した場合に限る

### 分類表テンプレート正本

個別文書の分類結果そのものはコミットせず、一時管理または GitHub Issue 側へ要約する。
repo に残すのは、分類時に使う列定義と判定ルールだけとする。

推奨列定義:

| 列名 | 必須 | 内容 | 例 |
| --- | --- | --- | --- |
| `path` | 必須 | 対象文書のパス | `dev/architecture/GEOMETRY_REFACTOR_ISSUE_DRAFT.md` |
| `kind` | 必須 | 文書種別 | `design` `issue-draft` `implementation-prep` `archive-note` |
| `status` | 必須 | 現在の状態 | `active` `stale` `archived` `closed-context` |
| `source_of_truth` | 必須 | 正本の所在 | `github-issue` `github-pr` `architecture-doc` `archive-only` |
| `decision` | 必須 | 分類結果 | `keep` `merge` `archive` `delete` |
| `naming_action` | 任意 | 命名の扱い | `keep-name` `rename` `rename-on-merge` |
| `reason` | 必須 | 判定理由の要約 | `Issue本文と重複` `長期参照価値あり` |
| `follow_up` | 任意 | 後続作業 | `Issue本文へ要約転記` `参照リンク置換後に削除` |
| `owner_issue` | 任意 | 管理元 Issue | `#429` |

`decision` の定義:

- `keep`: 文書内容を維持対象とする。現名維持は意味せず、必要なら命名変更を併用する
- `merge`: 正本へ要約転記したうえで、元文書は archive または削除候補にする
- `archive`: 現行正本ではないが履歴価値があるため archive へ置く
- `delete`: 正本でも履歴資産でもなく、repo に残さない

`naming_action` の定義:

- `keep-name`: 現在のファイル名をそのまま使う
- `rename`: 内容は維持するが、命名規約に沿った名前へ変更する
- `rename-on-merge`: `merge` 実施時に転記先の正本名へ統一する

判定時の確認順序:

1. 正本が GitHub Issue / PR 本文か、既存設計文書として定まっているか
2. 内容が単なる下書き、進捗、チェックリスト、完了報告ではないか
3. closed 後も履歴価値があるか
4. 複数 Issue で再利用される判断基準を含むか
5. 現行文書から参照されているか

運用ルール:

- 分類表の実データは原則コミットしない
- 分類の進捗共有は GitHub Issue コメントへ要約する
- 分類結果を repo に残す必要がある場合は、一覧表ではなく設計判断だけを正本へ要約転記する
- 一時ファイルを使う場合は作業完了時に削除し、残さない
- `keep` 判定でも命名規約違反が残る場合は `naming_action=rename` を付けて扱う

### 配置ルール

- `dev/architecture/`:
	- 現行方針として参照される設計文書のみ置く
	- 特定 Issue 起点でも、内容が Issue の進捗管理ではなく設計判断そのものである場合に限る
- `dev/architecture/issues/`:
	- 現行進行中の設計補助文書を置く
	- ただし Issue 本文の代替や下書き置き場として使わない
- `dev/archive/issues/`:
	- closed Issue の履歴として保持する価値があるものだけ置く
	- archive は保管場所であり、現行作業の参照元を増やす場所として使わない

### 既存旧メモの扱い

- 既存の `ISSUE_*_IMPLEMENTATION_PREP.md` や `*_ISSUE_DRAFT.md` は、新規作成を禁止する
- 既存ファイルは次の 3 区分で扱う
	- 正本化する: 複数箇所から継続参照される設計判断だけを、用途が分かる設計文書へ要約転記する
	- archive へ退避する: 履歴価値はあるが現行正本ではないもの
	- 削除する: 下書き、進捗、チェックリスト、完了報告など、履歴としても repo に残す価値が低いもの
- 旧メモを参照する現行文書がある場合は、参照先を新しい正本へ置き換えてから archive / 削除する

### 新規追加前の判断基準

- その内容は GitHub Issue / PR 本文かコメントで足りないか
- 1 つの Issue が閉じた後も読む価値が残るか
- 実装作業の進行管理ではなく、設計判断の保存になっているか
- 正本アーキテクチャ文書から参照する必要があるか
- 分類表の場合、個別対象一覧ではなく判定枠組みを残そうとしているか
- 作業方針書の場合、終了時の後処理先が最初から決まっているか

上記のいずれにも当てはまらない場合、新しい md を増やさず GitHub 側へ記録する

### 設計として残す場合の命名規則

- 設計文書は `dev/architecture/` または `dev/architecture/issues/` に置く
- 新規ファイル名の先頭に `ISSUE_` を使わない
- closed 済み Issue 起点の正本文書も、正本として残すなら `ISSUE_` 接頭辞を外した名前へ寄せる
- ファイル名で用途が分かる語尾を付ける: `_DESIGN.md` `_PLAN.md` `_POLICY.md` `_SUMMARY.md`
- Issue 関連の設計は `2026-03-<topic>.md` または `issue-426-<topic>-design.md` のように、Issue 管理用文書と区別できる命名にする

### 旧命名の扱い

- `ISSUE_*.md` は既存資産としてのみ許容し、新規追加しない
- closed Issue に紐づく既存 `ISSUE_*.md` が現行正本として残る場合は、そのまま恒久維持せず、次回更新時に用途が分かる正本名へ rename することを原則とする
- 上記 rename では `ISSUE_` 接頭辞を外し、必要なら Issue 番号は小文字の `issue-<番号>-...` として保持してよい
- `*_IMPLEMENTATION_PREP.md` は archive 済み履歴を除き、新規追加しない
- `*_ISSUE_DRAFT.md` は正本へ昇格しない限り repo に残さない
- archive 配下で履歴保持する文書へ改名する場合は、`ISSUE_` 接頭辞を外し、英小文字の kebab-case で `issue-<番号>-<topic>-archive-note.md` に統一する
- `<topic>` は文書の主題を 2 から 5 語程度で表し、`implementation-prep` のような旧工程名をそのまま残さない
- topic を短く特定できない場合は、内容の役割語を使って `issue-<番号>-decision-history-archive-note.md` または `issue-<番号>-design-history-archive-note.md` を使う
- 配置先が `dev/archive/issues/architecture/` の場合でも、ファイル名規則は同一とし、ディレクトリで分野、ファイル名で Issue と主題を表す
