# RedRing Development Skills Reference

## 最終更新日: 2026年10月12日

RedRing 開発の実装パターンと設計原則をまとめた、AI 開発者向けの手順書の目次です。

> **重要**: 実装開始前に必ず [copilot-instructions.md](copilot-instructions.md) の制約事項を確認してください。

---

## 📚 トピック別の手順書

### 🏗️ アーキテクチャ
- **[architecture/skill.md](skills/architecture/skill.md)** - レイヤー構成、依存関係の規則、アーキテクチャ検証

### 🔧 実装パターン
- **[foundation-pattern/skill.md](skills/foundation-pattern/skill.md)** - `geo_contracts` の trait定義（capability）の構成、相似変換、API 追加の原則
- **[implementation/skill.md](skills/implementation/skill.md)** - 形状のファイル構成・責務分担、テストの配置、エラー処理

### 🎨 描画システム
- **[rendering/skill.md](skills/rendering/skill.md)** - wgpu による描画、シェーダ、頂点型、RenderStage

### 🔀 Git/GitHub
- **[git-workflow/skill.md](skills/git-workflow/skill.md)** - ブランチ、コミット、PR のタイトル・本文、事前チェック

### ⚙️ 開発ツール
- **[commands/skill.md](skills/commands/skill.md)** - Cargo コマンド、チェックスクリプト、デバッグ、CI

---

## 開発フロー

1. **作業開始**: `develop` を `git pull --ff-only` で同期し、作業ブランチを作る（[git-workflow](skills/git-workflow/skill.md)）
2. **実装**: trait定義は [foundation-pattern](skills/foundation-pattern/skill.md)、形状の実装は [implementation](skills/implementation/skill.md) に従う
3. **検証**: push・PR の前に必ず実行する
   ```powershell
   pwsh -NoProfile -File .\scripts\check_all_before_push.ps1
   ```
4. **PR 作成**: `develop` に向け、タイトル・本文の形式は [git-workflow](skills/git-workflow/skill.md) に従う

---

## 重要な原則

- **trait定義の正本**: 形状の trait定義は `geo_contracts` に置き、具体型のクレートが実装する
- **最低限の API から拡張**: 呼び出し元が必要になった時点で API を追加し、呼び出し元のない API・別名・簡易実装を置かない
- **モジュール構成**: 形状は親ファイルと同名のディレクトリの子モジュールで構成し、テストは `tests.rs`（`dev/architecture/GEO_PRIMITIVES_MODULE_STRUCTURE_RULES.md`、`dev/architecture/TEST_PLACEMENT_RULES.md`）
- **依存の向き**: `view` → `viewmodel` → `model` → `foundation` の一方向（[architecture](skills/architecture/skill.md)）
- **型安全**: 失敗は `Option` / `Result` で明示し、方向（`Direction3D`）とベクトル（`Vector3D`）を分ける

---

## 参照文書

- **プロジェクト制約**: [copilot-instructions.md](copilot-instructions.md)
- **アーキテクチャ（正本）**: `dev/architecture/ARCHITECTURE.md`
- **Git/PR の運用（正本）**: `dev/GIT_PR_WORKFLOW_OPERATION.md`
- **設計文書集**: `dev/architecture/`
