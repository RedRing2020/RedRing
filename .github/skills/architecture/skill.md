# アーキテクチャ概要

## 最終更新日: 2026年2月13日

RedRingプロジェクトのアーキテクチャ構成と依存関係ルールを定義します。

---

## Workspace 構成

```text
foundation/         # 基礎機能（analysis: 数値解析・線形代数）
model/             # 幾何データ層
├── geo_foundation # トレイト定義・型システム
├── geo_commons    # 共通計算関数（楕円近似、距離計算等）
├── geo_primitives # 基本幾何要素
├── geo_core      # 幾何計算基盤
├── geo_algorithms # 高レベル幾何アルゴリズム
├── geo_nurbs     # NURBS曲線・曲面
└── geo_io        # ファイルI/O
view/              # アプリケーション・描画層
├── app           # メインアプリケーション
├── render        # GPU描画基盤（wgpu + WGSL）
└── stage         # レンダリングステージ管理
viewmodel/         # ビュー変換ロジック
├── converter     # 幾何データ→描画データ変換
└── graphics      # グラフィックス設定
```

---

## 依存関係ルール

### 基本原則

```text
analysis → geo_foundation
                ↓
           geo_commons
                ↓
           geo_core
            ↓    ↓
   geo_primitives  geo_nurbs
            ↓         ↓
      geo_algorithms  geo_io
            ↓
        viewmodel
            ↓
          view
```

### レイヤー間依存

1. **Foundation レイヤー** (`analysis`)
   - 他レイヤーに依存しない
   - 数値計算、線形代数、抽象型を提供

2. **Model レイヤー** (`geo_*`)
   - `geo_foundation`: analysis のみに依存
   - `geo_commons`: analysis, geo_foundation に依存
   - `geo_core`: analysis, geo_foundation に依存
   - `geo_primitives`: geo_foundation, geo_core, analysis に依存
   - `geo_nurbs`: geo_foundation, geo_core, analysis に依存（geo_primitives 直接依存禁止）
   - `geo_algorithms`: 上記全て + geo_primitives, geo_nurbs に依存可能

3. **ViewModel レイヤー** (`viewmodel`)
   - Model レイヤーに依存可能
   - View レイヤーに依存しない

4. **View レイヤー** (`view`)
   - `render`: analysis のみに依存（幾何データ層に依存しない）
   - `stage`: render, analysis に依存
   - `app`: render, stage, analysis に依存

### 重要な制約

- ✅ **geo_core ブリッジパターン**: `geo_nurbs` は `geo_core` 経由で具体型にアクセス
- ✅ **Foundation Pattern**: 全ての幾何計算は Foundation トレイト経由
- ✅ **View 独立性**: `render` と `stage` は model に依存しない
- ❌ **逆向き依存禁止**: 下位レイヤーが上位レイヤーに依存してはならない
- ❌ **geo_primitives 直接依存**: `geo_nurbs` が `geo_primitives` を直接参照することは禁止

---

## アーキテクチャ検証

### 依存関係チェック

```powershell
# アーキテクチャルールに準拠しているか検証
.\scripts\check_architecture_dependencies_simple.ps1
```

**検証項目**:
1. Model レイヤーの命名規則（`geo_` プレフィックス）
2. 依存関係の方向性
3. 禁止依存の検出
4. レイヤー間境界の遵守

### 出力例

```
=== RedRing Architecture Dependency Check ===
✅ SUCCESS: All architecture dependency checks passed!
  - View -> ViewModel -> Model direction maintained
  - Model layer naming rules followed
  - No forbidden dependencies detected
```

---

## 参照文書

- **詳細設計**: `dev/architecture/ARCHITECTURE.md`
- **MVVM設計**: `dev/architecture/VIEWMODEL_ARCHITECTURE_DESIGN.md`
- **依存関係図**: `dev/architecture/` 内の各種設計文書
