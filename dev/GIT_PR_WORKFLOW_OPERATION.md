# Git/PR運用ルール

最終更新: 2026-03-26

## 目的

- `develop` の履歴汚染と不要な競合を防ぐ
- PR前チェック漏れ（特に `cargo fmt --check`）を防ぐ

## 必須ルール

1. `develop` で直接実装しない
- 実装・修正・ドキュメント更新は必ず feature ブランチで実施

2. 作業開始前に同期確認
- `git checkout develop`
- `git pull --ff-only origin develop`
- `git checkout -b feature/<topic>`

3. `develop` が `ahead` のまま次作業へ進まない
- `git status -sb` で `ahead` を確認したら原因を解消してから作業開始

4. PR前の品質チェックを固定順序で実施
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

5. PR作成後の追いpush前にも同じ品質チェックを再実行
- 追加コミットを push する前に、必ず preflight を再実行する
- ルール対象: 「PR作成後の修正コミット」「レビュー指摘対応コミット」「fmt/clippy修正コミット」

6. PRマージ後の後処理を固定
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
