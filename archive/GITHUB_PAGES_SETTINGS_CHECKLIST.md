# GitHub Pages 設定チェックリスト

## 現在の設定確認項目

### 1. GitHub リポジトリ設定

以下の設定を GitHub Web UI で確認してください：

**Settings > Pages で以下を確認:**

- ✅ Source: "GitHub Actions" が選択されているか
- ✅ Branch: 自動設定されているか（Actions 使用時は不要）
- ✅ Custom domain: 設定されている場合は維持

### 2. CI/CD 設定の整合性確認

**✅ 完了済み:**

- `book.toml`: `build-dir = "docs"` ✓
- CI/CD: `path: ./docs` ✓
- 除外パス: `book/` ディレクトリ削除済み ✓

### 3. ワークフロー権限確認

GitHub Actions の権限設定:

```yaml
permissions:
  contents: read
  pages: write # GitHub Pages デプロイ権限
  id-token: write # OIDC トークン権限
```

## 必要なアクション

### GitHub Web UI での設定確認

1. **リポジトリ設定にアクセス:**

   ```text
   https://github.com/RedRing2020/RedRing/settings/pages
   ```

2. **Source 設定確認:**
   - "GitHub Actions" が選択されていることを確認
   - 他の設定（Branch 等）は不要

3. **Environment 設定確認:**

   ```text
   https://github.com/RedRing2020/RedRing/settings/environments
   ```

   - `github-pages` environment が存在することを確認

### 次回 main ブランチへのマージ時の確認項目

1. **Actions ログ確認:**
   - `Deploy Documentation` ジョブが正常実行
   - `./docs` からのアップロードが成功
   - GitHub Pages へのデプロイが成功

2. **サイト確認:**
   - ドキュメントサイトが正常表示
   - リンクやナビゲーションが機能

## トラブルシューティング

### もし Pages デプロイが失敗する場合

1. **権限エラー:**

   ```text
   Settings > Actions > General > Workflow permissions
   → "Read and write permissions" を選択
   ```

2. **Pages 設定エラー:**

   ```text
   Settings > Pages > Source
   → "GitHub Actions" を再選択
   ```

3. **Environment エラー:**
   ```text
   Settings > Environments > github-pages
   → Protection rules を確認
   ```

---

**更新日:** 2025 年 10 月 26 日
**対応完了:** Foundation・Docs 整理に伴う CI/CD 設定更新
