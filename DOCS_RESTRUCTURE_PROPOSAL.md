# ドキュメント構造再編提案

## 現在の課題

- `docs/`, `book/`, `manual/` が並存
- GitHub Pages設定との整合性
- mdbook の出力先設定

## 推奨構造

### オプション1: 統合型（推奨）

```
RedRing/
├── documentation/          # ドキュメント統合ディレクトリ
│   ├── source/            # mdbook ソース（現在のmanual/）
│   │   ├── intro.md
│   │   ├── modules.md
│   │   ├── kinds.md
│   │   ├── philosophy.md
│   │   └── SUMMARY.md
│   ├── build/             # mdbook 出力先（GitHub Pages用）
│   └── archive/           # 旧book/フォルダの内容（必要に応じて）
└── docs/                  # GitHub Pages実際の配信先（CI/CDで生成）
```

### オプション2: 分離型

```
RedRing/
├── docs/                  # GitHub Pages配信用（CI/CD生成、現状維持）
├── manual/               # mdbook ソース（現状維持）
└── archive/              # 旧book/フォルダ（履歴用）
```

## 推奨移行手順

### 1. オプション1（統合型）の場合

```bash
# 1. 統合ディレクトリ作成
mkdir documentation
mkdir documentation/source
mkdir documentation/build

# 2. ソースファイル移動
Move-Item manual/* documentation/source/

# 3. book.toml 更新
# src = "documentation/source"
# build-dir = "docs"

# 4. 旧book/フォルダのアーカイブ
Move-Item book documentation/archive
```

### 2. オプション2（分離型）の場合

```bash
# 1. 旧book/フォルダをアーカイブ
Move-Item book archive

# 2. 他は現状維持
# manual/ → そのまま
# docs/ → GitHub Pages用（CI/CD管理）
```

## GitHub Pages への影響

### 変更なし（オプション2）
- `book.toml`: 現在の設定維持
- CI/CD: 変更不要
- GitHub Pages: 影響なし

### 変更あり（オプション1）
- `book.toml`: `src` パスを更新
- CI/CD: 変更不要（出力先は同じ`docs/`）
- GitHub Pages: 影響なし

## 推奨案

**オプション2（分離型）** を推奨します：

### 理由
1. GitHub Pages設定への影響最小
2. 移行リスクが低い
3. 現在の作業フローを維持
4. 必要に応じて将来統合可能

### 実装手順
1. 重複している`book/`フォルダを`archive/`に移動
2. 現在の`manual/` → `docs/`の流れを維持
3. GitHub Pages設定は変更なし

## 次のステップ

どちらのオプションをご希望か決定いただければ、実装いたします。