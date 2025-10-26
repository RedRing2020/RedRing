# アーキテクチャ再編実行プラン

## 完了済み ✅
- `foundation/analysis/`: 移動済み、CI/CD独立性チェック済み
- `docs/`: 既に最適な状態（`manual/` → `docs/` の構造）

## 実行予定の移行

### Phase 1: Model層の統合
現在の散在クレートを `model/` 配下に整理：

```bash
# 現在の状態
geo_foundation/     → model/geo_foundation/
geo_core/          → model/geo_core/
geo_primitives/    → model/geo_primitives/
geo_algorithms/    → model/geo_algorithms/
model/             → model/integration/
```

### Phase 2: View層の統合  
```bash
# 現在の状態
render/            → view/render/
stage/             → view/stage/
redring/           → view/app/
```

### Phase 3: ViewModelの位置確認
```bash
viewmodel/         → viewmodel/viewmodel/ (または viewmodel/ のまま)
```

## VS Code ファイルネスト設定の更新

新しい構造に対応するネスト設定を追加：

```json
"explorer.fileNesting.patterns": {
  // 既存設定は維持...
  
  // アーキテクチャ層のグループ化
  "foundation": "foundation/**",
  "model": "model/**", 
  "viewmodel": "viewmodel/**",
  "view": "view/**",
  
  // 各層のCargo.toml
  "model/Cargo.toml": "model/*/Cargo.toml",
  "view/Cargo.toml": "view/*/Cargo.toml"
}
```

## Cargo.toml ワークスペース設定の更新

```toml
[workspace]
members = [
    "foundation/analysis",
    "model/geo_foundation", 
    "model/geo_core",
    "model/geo_primitives", 
    "model/geo_algorithms",
    "model/integration",
    "viewmodel",
    "view/render",
    "view/stage", 
    "view/app"
]
```

## 依存関係パス更新

各クレートの `Cargo.toml` で相対パスを更新：
- Foundation: `../foundation/analysis`
- Model内: `../geo_foundation`, `../geo_core` など  
- View層: `../../model/integration`, `../render` など

## メリット

### 1. VS Code エクスプローラでの視覚的整理
- Layer別にフォルダがグループ化
- ネスト設定で関連ファイルがまとまる
- アーキテクチャ構造が一目で理解可能

### 2. 開発効率の向上
- 責務別にファイル探索が効率化
- 新機能追加時の配置が明確
- 依存関係の把握が容易

### 3. 将来拡張の準備
- 新しいクレート追加時の方針が明確
- WebAssembly 対応時の構造準備済み
- チーム開発時の規約統一

## 実行順序

1. Model層統合（geo_* → model/）
2. View層統合（render, stage, redring → view/）  
3. VS Code設定更新
4. 依存関係パス更新
5. CI/CD設定更新
6. ビルド・テスト検証

## 想定作業時間

- Phase 1: 30分（Model層）
- Phase 2: 30分（View層）  
- Phase 3: 30分（設定更新・検証）
- 合計: 約1.5時間

この移行を実行しますか？