# RedRing 幾何計算アーキテクチャ設計原則

**作成日**: 2025年11月10日  
**最終更新日**: 2025年11月10日

## 設計原則

### 1. 責務分離の原則

#### geo_foundation (抽象化層)
- **責務**: 抽象トレイト定義（BasicTransform, ExtensionFoundation等）
- **特徴**: 具体実装を持たない、インターフェース定義のみ
- **依存関係**: analysis のみに依存

#### geo_primitives (プリミティブ幾何層)
- **責務**: 基本幾何形状の具体実装（Point3D, Vector3D, Circle3D等）
- **特徴**: 独自のBasicTransform実装を提供
- **依存関係**: geo_foundation, analysis に依存

#### geo_nurbs (NURBS幾何層) 
- **責務**: NURBS曲線・曲面の具体実装
- **特徴**: 独自のBasicTransform実装、geo_primitivesの基本型使用可能
- **依存関係**: geo_foundation, geo_primitives, analysis に依存

#### geo_core (共通計算層)
- **責務**: 交差判定等、全幾何の組み合わせが必要な実装
- **特徴**: 数値安定性・比較ロジック・低レベル補助機能
- **依存関係**: geo_foundation, analysis に依存

### 2. 循環参照回避の原則

正しい依存方向：
```
analysis → geo_foundation → geo_primitives
                ↓              ↓
           geo_core      geo_nurbs
```

**禁止される循環参照**:
- geo_foundation → geo_primitives → geo_foundation
- geo_core → geo_primitives → geo_core

### 3. Transform実装の分散原則

**各クレートで独自実装**:
- geo_primitives: プリミティブ用BasicTransform実装
- geo_nurbs: NURBS用BasicTransform実装
- geo_core: 共通のヘルパー関数・アルゴリズム提供

**geo_coreの役割変更**:
- ❌ 旧: Transform実装の中央集権化
- ✅ 新: 交差判定等の複合計算・共通アルゴリズム

### 4. 直接参照制限の緩和

**geo_nurbsのgeo_primitives使用を許可**:
- 理由: 基本形状（Point3D, Vector3D等）は共通基盤として必要
- 制限: 循環参照を引き起こす逆方向参照は禁止

## アーキテクチャ検証

### 依存関係チェック

```powershell
PowerShell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies_simple.ps1
```

### 許可されている依存関係

```powershell
"geo_foundation" = @("analysis")
"geo_primitives" = @("geo_foundation", "analysis") 
"geo_nurbs"      = @("geo_foundation", "geo_primitives", "analysis")
"geo_core"       = @("geo_foundation", "analysis")
```

## 今回の問題解決履歴

### 問題: NURBS実装時の循環参照
- **発生**: geo_core ← → geo_primitives の相互依存
- **原因**: Transform実装の配置に関する設計理解の混乱
- **解決**: Transform実装を各クレートに分散、geo_coreの役割を再定義

### 重要な学習事項
1. **Foundation vs Core**: Foundation=抽象層, Core=共通計算層
2. **Transform実装の分散**: 中央集権化ではなく、各専門クレートで実装
3. **geo_coreの真の役割**: 交差判定等の複合的な幾何計算

---

本文書は今後のアーキテクチャ決定の基準として使用される。