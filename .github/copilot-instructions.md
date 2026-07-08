# Copilot Instructions for RedRing

## 最終更新日: 2026年4月17日

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。

> **技術詳細**: 実装パターンや設計原則は [skills.md](skills.md) を参照してください。

## 🚨 AI開発者への厳格な制約（最終更新: 2025年12月27日）

### 回答言語

- AIの回答は必ず日本語で行う

### 用語統一ルール（表記ゆれ防止）

- `契約定義` という表現は使わず、`trait定義` を使用する
- `インターフェース` は文脈がRust traitを指す場合のみ補助的に使用し、基本は `trait` を優先する
- 詳細な正規用語は `dev/AI_TERMINOLOGY_GLOSSARY.md` を参照する

### コメント運用ルール（可読性優先）

- `// ==================================` 形式の区切り線コメントは新規追加しない
- コメントで構造化せず、`mod` 分割・関数抽出・trait分離で構造化する
- 交差判定のように形状組み合わせが多い実装では、コメント量が過密にならないよう最小限にする
- コメントは「何をしているか」より「なぜそうするか」を優先する

### 実装前の必須チェックリスト（絶対遵守）

**新規幾何プリミティブ実装時は以下を全て確認してからユーザーに報告**:

#### ステップ1: 既存実装の確認
```bash
# 同種の形状の完全な実装を確認
ls model/geo_contracts/src/geometry/solids/*_core_traits.rs
ls model/geo_primitives/src/*_solid_3d*.rs
```

#### ステップ2: Foundation Pattern の確認
以下の全てが揃っているか確認：
- [ ] `geo_contracts/src/geometry/solids/{shape}_core_traits.rs` - Core Traits 定義
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
3. **アーキテクチャ遵守**: `geo_core` ブリッジパターンを厳守（例: `geo_nurbs → geo_core → geo_contracts`）
4. **依存関係不変**: `scripts/check_architecture_dependencies_simple.ps1` の改変は絶対禁止
5. **表示不具合とカーネル変換の分離確認**: wgpu表示の症状だけを根拠に `geo_core` / `geo_nurbs` の行列積順や transform 実装を変更しない。変更前に、対象の transform 経路が実際に呼ばれていることを確認する

### 依存関係と import の運用ルール（重要）

- `geo_algorithms` は設計上 `geo_primitives` / `geo_nurbs` の上位層であり、クレート依存は許可される
- ただし `geo_algorithms` の実装ファイルでは `use geo_primitives::...` の直接 import を禁止し、`geo_algorithms::lib.rs` の再エクスポート経由（`use crate::...`）を使用する
- `geo_algorithms` 以外の下位層で、レイヤールールに反する `geo_primitives` 直接依存は引き続き禁止

### 絶対禁止事項

- ❌ ユーザー承認なしでの実装開始
- ❌ Foundation パターンの破壊や迂回
- ❌ 既存設計方針の無断変更
- ❌ レイヤールールに反する `geo_primitives` への直接依存の許可
- ❌ `geo_algorithms` 実装ファイル内での `use geo_primitives::...` 直接 import
- ❌ アーキテクチャチェックスクリプトの例外追加
- ❌ 表示の見え方だけを理由に、呼び出し経路未確認のまま transform の積順・座標変換カーネルを変更すること

### CAMデモ導線の隔離ルール（Issue #717 継続）

経緯の明確化:

- `viewmodel/cam_demo` は Issue #717 の CAM 混在解消を目的に導入した
- CAD を含む demo 全体設計は別タスクで扱う（同時完了していない）

- `model/application` / `model/cam_sim` / `model/job_runtime` は本番導線のみを扱い、demo専用シナリオ・demo専用ロード関数を追加しない
- demo専用の生成・再生ロジックは `view` / `viewmodel` 側に限定し、`model` 層へ逆流させない
- CAMデモ機能の追加先は `viewmodel/cam_demo` を正本とし、demo専用ロジックを `viewmodel/converter` に再導入しない
- CADとCAMのdemoを単一クレートへ統合しない。責務境界に沿って個別に管理する
- `view/app` は当面別クレート化しない。`view/app` は起動トリガーと表示状態管理を担い、demoロジック本体は持たない
- `CamSimulationDemoScenario`、`build_demo_artifacts_for_cam_simulation`、`create_demo_snapshot_exports_for_scenario` を `model/application` で再導入しない
- demo導線の変更時は「本番導線に影響しないこと」をテストで固定する

`view/app` 分離の再評価は次の場合のみ行う:

- demo導線の状態管理が `view/app` 全体へ拡散し、変更影響の追跡が困難になった場合
- 本番導線とdemo導線の変更周期が分離し、同一クレート運用が開発ボトルネックになった場合

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
5. **設計書と運用情報を混在させない**: Issue / PR の進捗、完了判定、チェックリスト、Phase完了報告は設計書へ持ち込まず、運用情報として別管理する

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
**✅ geo_nurbs**: Foundation Pattern準拠完了（geo_contracts/geo_core のみに依存）
**✅ 情報管理**: GitHub Issues/Projects 移行済み
**✅ geo_foundation**: 廃止完了（2026年3月20日）- trait定義はgeo_contractsへ統一

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

### AI向け運用ルール参照の優先順位（見落とし防止）

AIは、命名や進行管理の判断時に次の順で必ず参照すること。

1. PR/ブランチ/命名運用の正本: `dev/GIT_PR_WORKFLOW_OPERATION.md`
  - 特に「Phase と PR系列の命名を固定する」「タイトル表記フォーマットを固定する」を最優先で参照
2. Issueラベル運用の正本: `dev/ISSUE_LABEL_OPERATION.md`
  - 特に `phase-*` の意味（段階管理）と PR命名との対応ルールを参照
3. 用語統一の正本: `dev/AI_TERMINOLOGY_GLOSSARY.md`

補足:

- `Phase` は実施段階、`PR系列` は変更単位であり、混同しない
- PRタイトルは `issue #<番号> [Phase<番号>][<系列>-<枝番>] <対象>: <要約>` 形式を原則とする
- PR本文先頭に「含む単位 / 含めない単位」を明記する

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
