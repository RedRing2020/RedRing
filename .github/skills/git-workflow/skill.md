# Git/GitHub ワークフロー

## 最終更新日: 2026年2月13日

RedRingプロジェクトにおけるGit操作、コミット、PR作成の標準手順を定義します。

---

## ブランチ戦略

```
main (リリース版) ← develop (開発版) ← feature/issue-xxx-description (機能ブランチ)
```

### ブランチの役割

- **main**: リリース済みの安定版（本番環境デプロイ用）
- **develop**: 開発中の最新版（次期リリース候補）
- **feature/issue-xxx-\***: 機能追加・修正用の作業ブランチ

---

## 新規作業の開始

### 1. 最新の develop を取得

```bash
# developブランチに切り替え
git checkout develop

# 最新の変更を取得
git pull origin develop
```

### 2. 作業ブランチの作成

```bash
# Issue番号と説明を含む命名規則
git checkout -b feature/issue-123-add-sphere-primitive

# 別の例
git checkout -b fix/issue-456-vector-normalization
git checkout -b refactor/issue-789-foundation-pattern
```

**命名規則**:
- `feature/issue-xxx-description`: 新機能追加
- `fix/issue-xxx-description`: バグ修正
- `refactor/issue-xxx-description`: リファクタリング
- `docs/issue-xxx-description`: ドキュメント更新

---

## コミット作業

### 1. 変更内容の確認

```bash
# 変更ファイルの確認
git status

# 変更内容の詳細確認
git diff
```

### 2. ステージングとコミット

```bash
# 全ての変更をステージング
git add -A

# または特定ファイルのみ
git add model/geo_primitives/src/sphere_3d.rs

# コミットメッセージの作成
git commit -m "feat: Add Sphere3D primitive implementation

- Implement Core Traits (Constructor/Properties/Measure)
- Add Foundation Pattern integration
- Add comprehensive test suite

Ref: Issue #123"
```

### コミットメッセージ規則

**フォーマット**:
```
<type>: <subject>

<body>

<footer>
```

**Type 一覧**:
- `feat`: 新機能追加
- `fix`: バグ修正
- `refactor`: リファクタリング
- `docs`: ドキュメント更新
- `test`: テスト追加・修正
- `style`: コードフォーマット（機能変更なし）
- `chore`: ビルド設定、依存関係更新

**例**:
```
feat: Implement LineSegment3DCollisionDetection trait

- Add LineSegment3DCollisionDetection trait in geo_foundation
- Implement blanket implementation for LineSegment3D
- Refactor voxel.rs to use Foundation Pattern
- Remove geo_commons dependency from geo_algorithms

Ref: Issue #222, Option A
```

### 3. リモートへプッシュ

```bash
# ブランチをリモートにプッシュ
git push origin feature/issue-123-add-sphere-primitive

# 初回プッシュ時（上流ブランチ設定）
git push -u origin feature/issue-123-add-sphere-primitive
```

---

## Pull Request (PR) 作成

### 0. PR 作成前の手順

- PR 作成前に必ず `cargo fmt --all` を実行（変更を整形してからPR作成）

```bash
cargo fmt --all
```

### 1. GitHub CLI を使用した PR 作成

```bash
# developブランチへのPR作成（推奨）
gh pr create --base develop --title "feat: Add Sphere3D primitive" --body "$(cat PR_TEMPLATE.md)"

# または対話形式
gh pr create --base develop
```

### 2. PR テンプレート

```markdown
## 概要

Issue #123 の実装として、Sphere3D プリミティブを追加しました。

## 主な変更

### 1. Core Traits 実装
- **場所**: `geo_foundation/src/core/sphere_traits.rs`
- **機能**: Constructor/Properties/Measure トレイト

### 2. Sphere3D 実装
- **場所**: `geo_primitives/src/sphere_3d.rs`
- **実装方式**: Foundation Pattern準拠

### 3. テスト追加
- **テストケース**: 15件
- **カバレッジ**: Constructor, Properties, Measure

## テスト結果

\`\`\`
✅ cargo test -p geo_primitives: 全テスト成功
✅ cargo test --workspace: 全テスト成功
✅ cargo build: ビルド成功
✅ アーキテクチャチェック: 成功
\`\`\`

## 関連Issue

- Closes #123

## チェックリスト

- [x] develop から新規ブランチ作成
- [x] Core Traits 定義
- [x] Foundation Pattern実装
- [x] テスト実装
- [x] 全テスト成功確認
- [x] アーキテクチャチェック成功
- [x] ドキュメント更新
```

### 3. PR 作成後の CI 確認

```bash
# PR のステータス確認
gh pr status

# CIログの確認
gh pr checks
```

---

## 重要な注意事項（AI開発者向け）

### ❌ 絶対禁止事項

```bash
# main への直接PR作成は禁止
gh pr create --base main  # ← これは絶対ダメ
```

### ✅ 正しい手順

1. **必ず develop から分岐**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/issue-xxx
   ```

2. **PR は必ず develop へ**
   ```bash
   gh pr create --base develop
   ```

3. **重要な操作は必ずユーザー確認**
   - PR 作成前
   - ブランチ削除前
   - マージ実行前

### 確認コマンド

```bash
# 現在のブランチ確認
git branch --show-current

# ブランチの派生元確認
git log --oneline --graph --all -10

# PRのマージ先確認
gh pr view
```

---

## マージ後の作業

### 1. ブランチのクリーンアップ

```bash
# developに戻る
git checkout develop

# 最新の変更を取得
git pull origin develop

# マージ済みブランチの削除（ローカル）
git branch -d feature/issue-123-add-sphere-primitive

# リモートブランチの削除
git push origin --delete feature/issue-123-add-sphere-primitive
```

### 2. 次の作業の開始

```bash
# 最新のdevelopから新しいブランチを作成
git checkout -b feature/issue-124-next-task
```

---

## トラブルシューティング

### コンフリクトの解決

```bash
# developの最新変更を取得
git fetch origin develop

# 現在のブランチにマージ
git merge origin/develop

# コンフリクトを手動で解決後
git add .
git commit -m "Merge develop and resolve conflicts"
git push
```

### コミットの修正

```bash
# 直前のコミットメッセージを修正
git commit --amend -m "新しいメッセージ"

# プッシュ済みの場合（force pushは慎重に）
git push --force-with-lease
```

### 誤ったコミットの取り消し

```bash
# 直前のコミットを取り消し（変更は保持）
git reset --soft HEAD~1

# 直前のコミットを完全に取り消し
git reset --hard HEAD~1
```

---

## 参照文書

- **GitHub Projects**: プロジェクト管理とIssue追跡
- **INFORMATION_MANAGEMENT_TRANSITION.md**: 情報管理の移行方針
- **.github/PROJECT_STRUCTURE_POLICY.md**: プロジェクト構造方針
