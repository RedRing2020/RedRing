# Copilot Instructions for RedRing

## 最終更新日: 2026年2月14日

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。

> **技術詳細**: 実装パターンや設計原則は [skills.md](skills.md) を参照してください。

## 🚨 AI開発者への厳格な制約（最終更新: 2025年12月27日）

### 回答言語

- AIの回答は必ず日本語で行う

### 用語統一ルール（表記ゆれ防止）

- `契約定義` という表現は使わず、`trait定義` を使用する
- `インターフェース` は文脈がRust traitを指す場合のみ補助的に使用し、基本は `trait` を優先する
- 詳細な正規用語は `dev/AI_TERMINOLOGY_GLOSSARY.md` を参照する

### 実装前の必須チェックリスト（絶対遵守）

**新規幾何プリミティブ実装時は以下を全て確認してからユーザーに報告**:

#### ステップ1: 既存実装の確認
```bash
# 同種の形状の完全な実装を確認
ls model/geo_foundation/src/core/*_solid_core_traits.rs
ls model/geo_primitives/src/*_solid_3d*.rs
```

#### ステップ2: Foundation Pattern の確認
以下の全てが揃っているか確認：
- [ ] `geo_foundation/src/core/{shape}_core_traits.rs` - Core Traits 定義
  - [ ] `{Shape}Constructor<T>` trait
  - [ ] `{Shape}Properties<T>` trait  
  - [ ] `{Shape}Measure<T>` trait
- [ ] `geo_primitives/src/{shape}_3d.rs` - Core Traits 実装
- [ ] `geo_primitives/src/{shape}_3d_foundation.rs` - Extension 実装
- [ ] `geo_primitives/src/{shape}_3d_transform.rs` - Transform 実装

#### ステップ3: ユーザーへの確認
実装を開始する前に、以下の情報をユーザーに提示：
- 既存実装との比較（何が足りないか）
- 実装が必要なファイルの完全なリスト
- 実装順序の提案

**ユーザーの明示的な承認を得るまで実装を開始しない**

### 必須確認プロセス

1. **設計検討フェーズ**: 実装前に複数選択肢を提示し、ユーザー承認を得る
2. **継続作業確認**: 既存の途中実装がないか `dev/` フォルダと `model/geo_core` を確認
3. **アーキテクチャ遵守**: `geo_core` ブリッジパターンを厳守（例: `geo_nurbs → geo_core → geo_foundation`）
4. **依存関係不変**: `scripts/check_architecture_dependencies_simple.ps1` の改変は絶対禁止

### 絶対禁止事項

- ❌ ユーザー承認なしでの実装開始
- ❌ Foundation パターンの破壊や迂回
- ❌ 既存設計方針の無断変更
- ❌ `geo_primitives` への直接依存の許可
- ❌ アーキテクチャチェックスクリプトの例外追加

### 実装許可が必要な作業

- 新クレートの作成
- 依存関係の変更
- Foundation パターンの修正
- アーキテクチャの変更

### ドキュメントファースト原則（2025年11月11日追加）

1. **実装前にドキュメント更新**: 変更前に必ず dev/ フォルダの関連ドキュメントを更新
2. **ドキュメントを情報源とする**: dev/architecture/ の情報を最優先の参考とする
3. **古い情報の上書き禁止**: 実装と乖離したドキュメントで上書きしない
4. **現状の正確な記録**: 問題状況も含めて正確にドキュメントに記録

**重要**: 「設計検討を行いましょう」という指示は**設計提案のみ**を意味し、実装開始の許可ではありません。

---

## 現在の状態

**✅ ビルド状況**: 正常（cargo build/test 成功）
**✅ 型システム**: ジェネリック<T: Scalar>対応完了
**✅ Foundation パターン**: 実装完了
**✅ Phase 3 完了**: 衝突判定・交差判定機能実装完了（2025年12月21日）
**✅ Issue #222 完了**: geo_commons Foundation Pattern準拠（2026年2月13日）
**✅ Issue #210 完了**: NURBS適応的テッセレーション実装（2026年2月14日）
  - ハイブリッド方式（CPU分割 + GPU評価）
  - NurbsCurve3D/NurbsSurface3D対応
  - Foundation Pattern準拠（Extension Traits実装）
**✅ geo_nurbs**: Foundation Pattern準拠完了（geo_foundation/geo_core のみに依存）
**✅ 情報管理**: GitHub Issues/Projects 移行済み

---

## クイックリファレンス

**よく使うコマンド**:

```bash
cargo build                 # 全体ビルド
cargo test --workspace      # 全体テスト実行
cargo fmt --all -- --check  # フォーマットチェック
.\scripts\check_architecture_dependencies_simple.ps1  # アーキテクチャ検証
```

**重要な制約**:

- 新規コード追加時は既存のトレイト設計と責務分離を尊重
- `render` と `stage` は独立してビルド可能（model に依存しない）
- PRは必ず `develop` ブランチへ（`main` への直接PRは禁止）

**参照ドキュメント**:
---

## 開発時の心構え

コードを変更する際は、以下を優先してください：

1. **既存のトレイト設計を尊重** - Foundation Pattern を遵守
2. **責務分離の維持** - ファイル構成ルールに従う
3. **型安全性の確保** - Option/Result による失敗の明示化
4. **ドキュメント更新** - 実装前に関連ドキュメントを更新
5. **テストの充実** - 独立したテストファイルで検証

詳細は [skills.md](skills.md) を参照
- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- ログレベルは `RUST_LOG` 環境変数で制御
- 進捗管理: GitHub Issues/Projects で追跡

コードを変更する際は、既存のトレイト設計と責務分離を尊重し、型安全性を保つことを優先してください。
