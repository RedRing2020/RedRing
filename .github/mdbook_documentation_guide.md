# RedRing mdbook ドキュメントシステムガイド

**作成日: 2025年11月11日**  
**目的: mdbook装飾システム・テーマ設定・ドキュメント作成支援**

## 📚 mdbook システム概要

### 基本構成

```text
RedRing/
├── book.toml              # mdbook設定
├── manual/                # ドキュメントソース
│   ├── SUMMARY.md         # 目次構造
│   ├── intro.md           # はじめに
│   ├── modules.md         # モジュール構成
│   ├── kinds.md           # 型分類
│   ├── philosophy.md      # 設計思想
│   ├── core_extension_pattern.md  # Foundation パターン
│   ├── nurbs.md           # NURBS システム
│   └── theme/             # カスタムテーマ
│       ├── custom.css     # カスタムスタイル
│       └── custom.js      # カスタムJavaScript
└── docs/                  # 生成されたドキュメント（GitHub Pages）
```

## ⚙️ 設定ファイル (book.toml)

```toml
[book]
authors = ["RedRing Development Team"]
description = "RedRing CAD/CAM Platform Documentation - 高性能CAD/CAMソフトウェア開発プラットフォーム"
language = "ja"
multilingual = false
src = "manual"
title = "RedRing Documentation"

[build]
build-dir = "docs"

[output.html]
default-theme = "navy"
preferred-dark-theme = "navy"
curly-quotes = true
mathjax-support = true
copy-fonts = true
google-analytics = ""
additional-css = ["theme/custom.css"]
additional-js = ["theme/custom.js"]
git-repository-url = "https://github.com/RedRing2020/RedRing"
edit-url-template = "https://github.com/RedRing2020/RedRing/edit/main/manual/{path}"

[output.html.fold]
enable = true
level = 2

[output.html.print]
enable = true
```

## 🎨 カスタムCSS機能

### 主要スタイル機能

- **グラデーションヘッダー**: h1要素に美しいグラデーション効果
- **ステータスバッジ**: `✅ 実装完了`, `🚧 実装中`, `📋 計画中` の自動カラーリング
- **装飾ボックス**: `.highlight-box`, `.warning-box`, `.success-box`
- **コードブロック**: 言語ラベル、シャドウ効果、コピー機能
- **テーブル**: ホバー効果、交互カラー、グラデーションヘッダー

### CSSクラス一覧

```css
/* 装飾ボックス */
.highlight-box    /* 重要な情報（青系） */
.warning-box      /* 注意事項（オレンジ系） */
.success-box      /* 成功メッセージ（緑系） */

/* ステータスバッジ（自動適用） */
.status-complete  /* ✅ 完了系 */
.status-progress  /* 🚧 進行中系 */
.status-planned   /* 📋 計画中系 */
```

## 🚀 カスタムJavaScript機能

### 自動機能

- **自動目次生成**: 長いページ（h2が3個以上）の場合
- **コードコピー機能**: ワンクリックでクリップボードにコピー
- **ステータスバッジ自動追加**: 特定キーワードを検出してバッジ追加
- **スムーススクロール**: アンカーリンクの改善
- **フェードインアニメーション**: ページロード時の美しいアニメーション

### JavaScript機能詳細

```javascript
// 自動検出キーワード
const statusKeywords = {
  complete: ["✅", "完了", "実装済み", "成功"],
  progress: ["🚧", "実装中", "開発中", "進行中"],
  planned: ["📋", "計画中", "予定", "検討中"],
};

// 目次自動生成（h2が3個以上で発動）
// コードコピー機能（全コードブロックに適用）
// スムーススクロール（全アンカーリンクに適用）
```

## 📝 利用可能な装飾記法

### HTMLボックス

```html
<div class="highlight-box">💡 重要な情報やポイントを強調表示</div>

<div class="warning-box">⚠️ 注意事項や警告メッセージ</div>

<div class="success-box">✅ 成功メッセージや完了通知</div>
```

### 推奨絵文字セット

```text
📦 モジュール    ⚡ パフォーマンス  🎯 重要機能    🏗️ アーキテクチャ
✅ 完了         🚧 進行中        📋 計画中      🎨 デザイン
🔒 セキュリティ  🌟 特徴         🎉 成果       📊 データ
🧪 テスト       🚀 改善         💡 アイデア    🏆 成功
📅 日付        📈 成長         🔧 ツール      💻 開発
```

### マークダウン拡張

````markdown
# グラデーションヘッダー（自動適用）

## セクションヘッダー

### サブセクション

- ✅ 完了項目（自動バッジ）
- 🚧 進行中項目（自動バッジ）
- 📋 計画項目（自動バッジ）

```rust
// コードブロック（自動コピー機能付き）
fn example() {
    println!("Hello, RedRing!");
}
```
````

| 項目  | 状態    | 説明         |
| ----- | ------- | ------------ |
| NURBS | ✅ 完了 | 基礎実装済み |

## 🔧 開発コマンド

### mdbook操作

```bash
# ドキュメント生成
mdbook build

# 開発サーバー起動
mdbook serve

# 開発サーバー（ポート指定）
mdbook serve --port 3001

# 自動リロード付き開発
mdbook serve --open
```

### ファイル更新時の確認

```bash
# 生成確認
ls docs/

# GitHub Pages確認
# https://redring2020.github.io/RedRing/
```

## 📋 ドキュメント作成ガイドライン

### ファイル構成原則

1. **SUMMARY.md**: 必ず目次構造を更新
2. **ファイル名**: 英語、小文字、アンダースコア区切り
3. **タイトル**: 日本語、分かりやすく
4. **更新日**: ファイル先頭に記載

### 推奨構造

```markdown
# タイトル

**作成日: YYYY年MM月DD日**  
**最終更新: YYYY年MM月DD日**

## 概要

<!-- 概要説明 -->

## 詳細

<!-- 詳細説明 -->

### サブセクション

<!-- サブセクション -->

## まとめ

<!-- まとめ -->
```

### スタイル統一原則

1. **絵文字**: 各セクションの先頭に適切な絵文字
2. **装飾ボックス**: 重要な情報は装飾ボックスで強調
3. **コードブロック**: 言語指定を必ず行う
4. **テーブル**: 比較や状況説明で積極活用

## 🎯 成功事例

### NURBS ドキュメント (`manual/nurbs.md`)

- **包括的な技術説明**: 数学的背景から実装詳細まで
- **美しい装飾**: カスタムCSS/JSの全機能を活用
- **実用的な例**: コード例とプログラム例の豊富な提供
- **ユーザビリティ**: 自動目次、コピー機能、スムーススクロール

## 🚀 カスタマイズ拡張

### CSS追加機能の実装方法

```css
/* manual/theme/custom.css に追加 */
.new-box {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-left: 4px solid #4f46e5;
  padding: 1rem;
  margin: 1rem 0;
  border-radius: 0.5rem;
  color: white;
}
```

### JavaScript追加機能の実装方法

```javascript
// manual/theme/custom.js に追加
document.addEventListener("DOMContentLoaded", function () {
  // 新機能の実装
});
```

---

**📚 このファイルの目的**: RedRing の mdbook ドキュメントシステムを効率的に活用し、美しく実用的なドキュメントを作成するための包括的ガイド
