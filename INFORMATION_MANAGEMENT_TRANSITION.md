# RedRing 情報管理方針転換

**作成日: 2025年10月28日**
**最終更新: 2025年10月28日**

## 背景
- ルート直下に約30個のMDファイルが散乱
- GitHub Pages (manual/docs/) との情報重複
- 進捗管理の破綻（毎日記憶喪失状態）

## 新しい管理方針

### 1. ドキュメント階層化
```
docs/           # GitHub Pages（公開用HTML生成先）
manual/         # mdbook ソース（ユーザー向けドキュメント）

dev/            # 開発者向けドキュメント（ルート直下）
├── architecture/   # アーキテクチャ設計ファイル
└── foundation/     # Foundation パターン関連

archive/        # 古い分析・提案ファイル（ルート直下）

.github/
├── copilot-instructions.md  # 簡素化版
└── ISSUE_TEMPLATE/          # Issue テンプレート
```

### 2. 進捗管理移行
- **GitHub Issues**: 具体的タスク
- **GitHub Projects**: フェーズ管理
- **GitHub Milestones**: 重要な節目
- **copilot-instructions.md**: 現在状況のみ（詳細進捗除外）

### 3. ルート直下整理
```bash
# 移動対象
ARCHITECTURE*.md         → dev/architecture/
FOUNDATION*.md          → dev/foundation/
*_ANALYSIS.md          → archive/
*_PROPOSAL.md          → archive/
MIGRATION*.md          → archive/
```

### 4. 残すもの
```
README.md              # プロジェクト概要
Cargo.toml            # ビルド設定
book.toml             # ドキュメント生成
LICENSE               # ライセンス
```

## 実装手順

1. **Phase 1**: アーカイブフォルダ作成・移動
2. **Phase 2**: copilot-instructions.md簡素化
3. **Phase 3**: GitHub Issues/Projects設定
4. **Phase 4**: CI/CDでドキュメント自動更新

## 期待効果

- **情報の一元化**: GitHub Issues中心
- **ドキュメント整理**: 用途別階層化
- **進捗追跡**: 自動化・可視化
- **記憶喪失対策**: Issueベース管理

この方針で進めることで、情報管理の破綻を防げると考えます。