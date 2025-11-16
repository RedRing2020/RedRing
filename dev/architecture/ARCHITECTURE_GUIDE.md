# RedRing アーキテクチャガイド

**最終更新**: 2025年11月11日

## 現在の構造（2025年11月11日時点）

✅ **層別アーキテクチャ構造完成済み**

- Foundation層: `foundation/analysis/` - 独立基盤ライブラリ
- Model層: `model/geo_*/` - 幾何・データ処理クレート群
- ViewModel層: `viewmodel/converter/`, `viewmodel/graphics/` - 変換・架け橋
- View層: `view/app/`, `view/render/`, `view/stage/` - 描画・UI
- Documentation: `manual/` → `docs/` の最適化構造

## 提案構造

```text
RedRing/
├── foundation/          # 独立基盤ライブラリ（ドメイン非依存）
│   └── analysis/       # 数値計算（現在のanalysis/）
│
├── model/              # Model層（幾何・データ処理）
│   ├── geo_foundation/ # 基盤トレイト（現在のgeo_foundation/）
│   ├── geo_core/       # 基本処理（現在のgeo_core/）
│   ├── geo_primitives/ # 基本要素（現在のgeo_primitives/）
│   ├── geo_algorithms/ # アルゴリズム（現在のgeo_algorithms/）
│   ├── geo_io/         # I/O境界（現在のgeo_io/）
│   └── integration/    # 統合（現在のmodel/）
│
├── viewmodel/          # ViewModel層（変換・架け橋）
│   └── viewmodel/      # 現在のviewmodel/
│
├── view/               # View層（描画・UI）
│   ├── render/         # GPU描画（移行済み）
│   ├── stage/          # レンダリングステージ（移行済み）
│   └── app/            # メインアプリ（移行済み）
│
├── manual/             # mdbook ソース（編集用）
└── docs/               # GitHub Pages 配信用（CI/CD生成）
```

## 利点

### 1. 明確な責務分離

- foundation/: ドメイン非依存の基盤
- model/: 幾何・データ処理専門
- viewmodel/: MVVM の架け橋
- view/: 描画・UI 専門

### 2. 依存関係の可視化

```text
view/ → viewmodel/ → model/
     ↗              ↗
foundation/ （全層で利用可能）
```

### 3. 新規開発者の理解促進

- ディレクトリ名でアーキテクチャが明確
- クレート探索の効率化
- 責務境界の視覚化

### 4. 将来拡張の容易性

- model/geo_nurbs/ (NURBS 追加)
- model/geo_step/ (STEP 対応)
- view/web/ (WebAssembly 版)
- foundation/physics/ (物理演算基盤)

## 移行作業量の見積もり

### 軽微な変更

- ディレクトリ移動
- Cargo.toml の path 更新
- CI/CD パスの更新

### 中程度の変更

- use 文のパス修正
- ドキュメントの更新

### 重大な変更

- 既存の import 文は互換性維持可能
- 段階的移行で手戻り最小化

## 現在の課題（2025年11月11日）

### 🚨 解決が必要な問題

1. **geo_nurbs Foundation パターン違反**: geo_primitives を直接インポート
2. **geo_core ブリッジ未完成**: Foundation トレイトと具体型の仲介機能が未実装
3. **アーキテクチャチェック失敗**: 依存関係チェックスクリプトでエラー検出

### 🎯 修正方針

- geo_core でのブリッジパターン実装完了
- geo_nurbs から geo_primitives への直接依存を削除
- geo_nurbs → geo_core → geo_foundation のルート確立

## 結論

この層別アーキテクチャにより、**責務分離**と**依存関係の明確化**が実現されています。
現在の Foundation パターン違反を修正することで、設計原則の完全な実現を目指します。

**維持コスト**: 低（明確な構造により保守が容易）
**拡張性**: 高（新機能の配置が明確）
