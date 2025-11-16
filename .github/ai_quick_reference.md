# 🤖 AI開発者用クイックリファレンス

## 📅 最終更新: 2025年11月11日

## ⚡ クイックコマンド

```bash
# 現状確認
git status
git log --oneline -5

# ビルド・テスト
cargo build
cargo test --workspace

# 品質チェック
cargo clippy --workspace
cargo fmt

# アーキテクチャチェック
powershell -ExecutionPolicy Bypass -File "scripts\check_architecture_dependencies_simple.ps1"

# ドキュメント
mdbook build
mdbook serve

# 依存関係確認
cargo tree --depth 1
```

## 📚 重要ファイル参照

### AI開発支援

- `.github/copilot-instructions.md` - **メイン指示ファイル**（最重要）
- `.github/ai_session_context.md` - セッション復旧・状況把握
- `.github/mdbook_documentation_guide.md` - ドキュメント作成ガイド
- `.github/PROJECT_STRUCTURE_POLICY.md` - プロジェクト構造方針

### 設計文書

- `dev/architecture/ARCHITECTURE.md` - 技術アーキテクチャ
- `dev/foundation/` - Foundation パターン設計文書
- `scripts/check_architecture_dependencies_simple.ps1` - 依存関係チェック

### ドキュメント

- `manual/` - mdbook ソースファイル
- `docs/` - 生成された公開ドキュメント

## 🔧 装飾システム使用法

### HTMLボックス

```html
<div class="success-box">✅ 成功メッセージ</div>
<div class="warning-box">⚠️ 注意事項</div>
<div class="highlight-box">💡 重要な情報</div>
```

### 推奨絵文字

```text
📦 モジュール    ⚡ パフォーマンス  🎯 重要機能    🏗️ アーキテクチャ
✅ 完了         🚧 進行中        📋 計画中      🎨 デザイン
🔒 セキュリティ  🌟 特徴         🎉 成果       📊 データ
🧪 テスト       🚀 改善         💡 アイデア    🏆 成功
📅 日付        📈 成長         🔧 ツール      💻 開発
```

## 🎯 開発フロー

### 基本プロセス

1. **状況確認**: `ai_session_context.md` で現在の状況を把握
2. **制約確認**: `copilot-instructions.md` で開発制約を確認
3. **設計検討**: 実装前に必ず設計提案・ユーザー承認
4. **アーキテクチャ遵守**: Foundation パターン・依存関係の厳守
5. **実装・テスト**: 承認後の慎重な実装

### 禁止事項

- ❌ 設計検討なしでの実装開始
- ❌ アーキテクチャ制約の無視
- ❌ 依存関係チェックスクリプトの改変
- ❌ Foundation パターンの破壊

## 🔗 Git・GitHub

### プルリクエスト作成

```bash
# 現在のブランチから
git push origin <branch-name>

# GitHub URL
# https://github.com/RedRing2020/RedRing/pull/new/<branch-name>
```

### ブランチ戦略

- `main` - 安定版
- `feature/*` - 機能開発
- `fix/*` - バグ修正

## 💡 AI開発の成功原則

### 必須の心構え

1. **設計優先**: 実装前に必ず設計検討
2. **制約遵守**: 既存アーキテクチャの尊重
3. **継続性確認**: 途中実装の存在確認
4. **段階的進行**: 小さなステップでの確実な進行

### 品質保証

- **テスト**: 実装後は必ずテスト実行
- **Clippy**: 警告ゼロを維持
- **アーキテクチャ**: 依存関係チェック通過
- **ドキュメント**: 適切な文書更新

---

**💡 このファイルの目的**: AI開発者が RedRing プロジェクトで効率的かつ安全に作業するための基本的なリファレンス情報を提供
