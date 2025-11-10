# RedRing 幾何計算アーキテクチャ設計原則

**作成日**: 2025 年 11 月 10 日
**最終更新日**: 2025 年 11 月 11 日

## 設計原則

### 1. 責務分離の原則

#### geo_foundation (抽象化層)

- **責務**: 抽象トレイト定義（BasicTransform, ExtensionFoundation 等）
- **特徴**: 具体実装を持たない、インターフェース定義のみ
- **依存関係**: analysis のみに依存

#### geo_primitives (プリミティブ幾何層)

- **責務**: 基本幾何形状の具体実装（Point3D, Vector3D, Circle3D 等）
- **特徴**: 独自の BasicTransform 実装を提供
- **依存関係**: geo_foundation, analysis に依存

#### geo_nurbs (NURBS 幾何層) ⚠️ 現在Foundation パターン違反状態

- **責務**: NURBS 曲線・曲面の具体実装
- **特徴**: 独自の BasicTransform 実装、geo_core 経由でのアクセス
- **現在の状態**: geo_primitives を直接インポート（違反状態）
- **正しい依存関係**: geo_foundation, geo_core, analysis のみ
- **修正予定**: geo_core ブリッジパターンによる間接アクセス

#### geo_core (共通計算層)

- **責務**: 交差判定等、全幾何の組み合わせが必要な実装
- **特徴**: 数値安定性・比較ロジック・低レベル補助機能
- **依存関係**: geo_foundation, analysis に依存

### 2. 循環参照回避の原則

正しい依存方向（修正版 2025年11月11日）：

```
analysis → geo_foundation
                ↓
           geo_core（ブリッジ役）
            ↓    ↓
   geo_primitives  geo_nurbs
```

**重要**: geo_nurbs は geo_primitives を直接参照してはいけない

**禁止される循環参照**:

- geo_foundation → geo_primitives → geo_foundation
- geo_core → geo_primitives → geo_core

### 3. Transform 実装の分散原則

**各クレートで独自実装**:

- geo_primitives: プリミティブ用 BasicTransform 実装
- geo_nurbs: NURBS 用 BasicTransform 実装
- geo_core: 共通のヘルパー関数・アルゴリズム提供

**geo_core の役割変更**:

- ❌ 旧: Transform 実装の中央集権化
- ✅ 新: 交差判定等の複合計算・共通アルゴリズム

### 4. Foundation パターン厳守の原則（2025年11月11日修正）

**geo_nurbs の geo_primitives 直接参照は禁止**:

- 理由: Foundation パターンの整合性維持のため
- 解決策: geo_core ブリッジパターンによる間接アクセス
- 現状: 違反状態のため修正が必要

## アーキテクチャ検証

### 依存関係チェック

```powershell
PowerShell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies_simple.ps1
```

### 正しい依存関係（2025年11月11日修正）

```powershell
"geo_foundation" = @("analysis")
"geo_core"       = @("geo_foundation", "analysis")
"geo_primitives" = @("geo_foundation", "geo_core", "analysis")
"geo_nurbs"      = @("geo_foundation", "geo_core", "analysis")  # geo_primitives 直接参照は禁止
```

**重要**: 現在の geo_nurbs は上記に違反している状態

### 現在の課題（2025年11月11日更新）

### 🚨 緊急対応が必要な問題

1. **geo_nurbs Foundation パターン違反**: 
   - 現状: geo_primitives を全ファイルで直接インポート
   - 影響: アーキテクチャチェック失敗、Foundation パターン破綻

2. **geo_core ブリッジ未実装**:
   - 現状: Foundation トレイトと具体型の仲介機能が未完成
   - 影響: 上位クレートが Foundation パターンを遵守できない

### 修正方針

1. **geo_core ブリッジパターン実装完了**
2. **geo_nurbs から geo_primitives への直接依存削除**
3. **geo_nurbs → geo_core → geo_foundation ルートの確立**

### 重要な教訓

1. **Foundation パターンは絶対遵守**: 例外は認めない
2. **ドキュメントファースト**: 実装前に必ずドキュメント更新
3. **アーキテクチャチェックの重要性**: 違反を早期発見

---

本文書は今後のアーキテクチャ決定の基準として使用される。
