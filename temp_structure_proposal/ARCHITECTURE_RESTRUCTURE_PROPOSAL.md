# RedRing アーキテクチャ再編提案

## 現在の課題

- Model 層クレートが散在（geo\_\*, model/）
- View 層クレートが散在（redring/, render/, stage/）
- 名前空間が不明確で依存関係が把握しにくい

## 提案構造

```
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
│   ├── render/         # GPU描画（現在のrender/）
│   ├── stage/          # レンダリングステージ（現在のstage/）
│   └── app/            # メインアプリ（現在のredring/）
│
└── docs/               # ドキュメント
    ├── manual/         # 現在のmanual/
    ├── book/           # 現在のbook/
    └── docs/           # 現在のdocs/
```

## 利点

### 1. 明確な責務分離

- foundation/: ドメイン非依存の基盤
- model/: 幾何・データ処理専門
- viewmodel/: MVVM の架け橋
- view/: 描画・UI 専門

### 2. 依存関係の可視化

```
view/ → viewmodel/ → model/
     ↗              ↗
foundation/ (全層で利用可能)
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

## 結論

**推奨**: この再編により、今後の実装手戻りが大幅に削減され、
新規機能追加時の配置決定が明確になります。

**移行コスト**: 中程度（1-2 日の作業）
**長期利益**: 大（開発効率向上、保守性向上）
