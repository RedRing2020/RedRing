# RedRing Development Skills Reference

## 最終更新日: 2026年2月13日

本ドキュメントは、RedRing開発における技術的な実装パターンと設計原則をまとめたAI開発者向けリファレンスです。

> **重要**: 実装開始前に必ず [copilot-instructions.md](copilot-instructions.md) の制約事項を確認してください。

---

## 📚 トピック別ドキュメント

技術詳細は以下のトピック別ドキュメントを参照してください：

### 🏗️ アーキテクチャ
- **[architecture/skill.md](skills/architecture/skill.md)** - Workspace構成、依存関係ルール、アーキテクチャ検証

### 🔧 実装パターン
- **[foundation-pattern/skill.md](skills/foundation-pattern/skill.md)** - Foundation Pattern詳細、Core/Extension/Transform Traits
- **[implementation/skill.md](skills/implementation/skill.md)** - geo_primitives実装規約、ファイル構成、テストルール

### 🎨 描画システム
- **[rendering/skill.md](skills/rendering/skill.md)** - GPU描画、wgpu、シェーダ管理、RenderStage

### 🔀 Git/GitHub
- **[git-workflow/skill.md](skills/git-workflow/skill.md)** - ブランチ戦略、コミット規則、PR作成手順

### ⚙️ 開発ツール
- **[commands/skill.md](skills/commands/skill.md)** - Cargoコマンド、テスト、デバッグ、CI/CD

---

## クイックリファレンス

### よく使うコマンド

```bash
cargo build                 # 全体ビルド
cargo test --workspace      # 全テスト実行
cargo fmt --all -- --check  # フォーマットチェック
.\scripts\check_architecture_dependencies_simple.ps1  # アーキテクチャ検証
```

### 開発フロー

1. **作業開始**: [git-workflow/skill.md](skills/git-workflow/skill.md#新規作業の開始)
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/issue-xxx-description
   ```

2. **実装**: [implementation/skill.md](skills/implementation/skill.md#ファイル構成)
   - Foundation Pattern に準拠
   - テストを独立ファイルに分離

3. **検証**: [commands/skill.md](skills/commands/skill.md#コード品質チェック)
   ```bash
   cargo test --workspace
   cargo fmt --all -- --check
   cargo clippy -- -D warnings
   ```

4. **PR作成**: [git-workflow/skill.md](skills/git-workflow/skill.md#pull-request-pr-作成)
   ```bash
   gh pr create --base develop --title "..." --body "..."
   ```

---

## 重要な原則

### Foundation Pattern（3層構造）

```
Core Traits (Constructor/Properties/Measure)
    ↓
Extension Traits (Bounded, Transformable)
    ↓
Transform Traits (AnalysisTransform2D/3D)
```

詳細: [foundation-pattern/skill.md](skills/foundation-pattern/skill.md)

### 依存関係ルール

```
analysis → geo_foundation → geo_core
                              ↓    ↓
                     geo_primitives  geo_nurbs
```

詳細: [architecture/skill.md](skills/architecture/skill.md#依存関係ルール)

### 型安全パターン

- Option/Result による失敗の明示化
- Direction と Vector の明確な分離
- トレイト境界による抽象化

詳細: [implementation/skill.md](skills/implementation/skill.md#型安全パターン)

---

## 参照文書

- **プロジェクト制約**: [copilot-instructions.md](copilot-instructions.md)
- **プロジェクト構造方針**: `PROJECT_STRUCTURE_POLICY.md`
- **技術アーキテクチャ**: `dev/architecture/ARCHITECTURE.md`
- **設計文書集**: `dev/architecture/`, `dev/foundation/`
