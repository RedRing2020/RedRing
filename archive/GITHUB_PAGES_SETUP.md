# GitHub Pages デプロイ設定ガイド

このリポジトリでは、GitHub Actions を使用して自動的にドキュメントを GitHub Pages にデプロイします。

## 設定手順

### 1. GitHub Pages の有効化

1. リポジトリの **Settings** タブに移動
2. 左サイドバーの **Pages** をクリック
3. **Source** で **"GitHub Actions"** を選択

### 2. 権限設定の確認

以下の権限が有効になっていることを確認してください：

1. **Settings** → **Actions** → **General**
2. **Workflow permissions** で以下を選択：
   - ✅ **"Read and write permissions"**
   - ✅ **"Allow GitHub Actions to create and approve pull requests"** (オプション)

### 3. デプロイトリガー

ドキュメントは以下の条件で自動デプロイされます：

- `main` ブランチへの push
- ビルドとテストが成功した後
- `manual/` ディレクトリの変更が含まれる場合

## デプロイされる URL

デプロイ後、以下の URL でドキュメントにアクセスできます：

```
https://redring2020.github.io/RedRing/
```

## ローカルでのプレビュー

ローカルでドキュメントをプレビューする場合：

```bash
# mdbook のインストール（初回のみ）
cargo install mdbook

# ドキュメントのビルド
mdbook build

# ローカルサーバーでプレビュー
mdbook serve --open
```

## トラブルシューティング

### デプロイが失敗する場合

1. **権限エラー**: 上記の権限設定を確認
2. **ビルドエラー**: ローカルで `mdbook build` を実行してエラーを確認
3. **ブランチ設定**: `main` ブランチから実行されているか確認

### ドキュメントが更新されない場合

1. GitHub Actions のログを確認（Actions タブ）
2. `gh-pages` ブランチが作成されているか確認
3. Pages 設定で正しいブランチが選択されているか確認

## CI ワークフローの詳細

CI ワークフロー（`.github/workflows/ci.yml`）には以下のジョブが含まれています：

1. **build-test**: Rust プロジェクトのビルドとテスト
2. **deprecated-scan**: 非推奨 API の使用チェック
3. **deploy-docs**: ドキュメントのビルドと GitHub Pages へのデプロイ

`deploy-docs` ジョブは：

- `main` ブランチへの push でのみ実行
- 他のジョブが成功した場合のみ実行
- `mdbook build` でドキュメントをビルド
- `peaceiris/actions-gh-pages` でデプロイ
