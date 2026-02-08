# CAM機能の設計方針・クレート構成検討

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**ステータス**: 設計検討中 - ユーザー承認待ち  
**関連Issue**: #203

---

## 📋 検討事項サマリー

### 1. Issue #203の範囲精査と新規Issue分割

**質問**: 現在の設計内容を反映したIssue更新と、段階的実装のための新規Issue分割が必要か？

### 2. CAM機能のクレート配置

**質問**: CAM機能を `geo_algorithms` に配置するか、新規 `cam` クレートを作成するか？

### 3. CADとCAMの境界定義

**質問**: RedRingにおけるCADとCAMの責務をどう分離するか？

### 4. 2D CAM対象形状の扱い

**質問**: 閉じた線群セグメントとして判定する必要があるか？トレランスは必要か？

---

## 1. Issue #203の範囲精査

### 現在のIssue #203の問題点

**元のIssue内容**:
- 「CAM可視化システム設計・実装」として5日の工数
- Phase 1-3全てを含む広範な内容
- ツール管理、副座標、5軸加工などが混在

**今回の設計で明確化された内容**:
- **Phase 1**: 3軸加工の基本可視化のみ（2-3日）
- **Phase 2以降**: 別Issue化が必要（ツール管理、副座標、パス接続詳細）

### 提案: Issue分割案

#### ✅ Issue #203（更新後）: CAM工具経路可視化 Phase 1

**スコープ**:
- 3軸加工の工具経路表示（2D CAM/3D CAM）
- セグメント種別の色分け
- 等高線レベル管理
- 送り速度（F値）表示

**工数**: 2-3日（Week 3）  
**優先度**: Tier 1

#### 📝 Issue #211（新規作成）: ツール管理システム

**スコープ**:
- ツールセット登録・管理
- 工具ライブラリDB
- 工具選択UI

**工数**: 2週間（Week 10-12）  
**優先度**: Tier 2  
**依存**: #203完了後

#### 📝 Issue #212（新規作成）: 副座標（subaxis）機能

**スコープ**:
- 複数ワーク座標系の定義
- 座標系切り替え（G54-G59）
- 座標系ごとの工具経路表示

**工数**: 1週間（Week 10-12）  
**優先度**: Tier 2  
**依存**: #203完了後

#### 📝 Issue #213（新規作成）: パス接続詳細表示

**スコープ**:
- 直線角度接続の可視化
- 円弧接続の可視化
- 接続パラメータ表示

**工数**: 1週間（Week 10-12）  
**優先度**: Tier 2  
**依存**: #203完了後

---

## 2. CAM機能のクレート配置検討

### Option A: `geo_algorithms` に配置（現状維持）

**メリット**:
- ✅ 既存クレートを活用、新規クレート不要
- ✅ オフセット、ブール演算など幾何アルゴリズムと近い
- ✅ クレート数が増えない（シンプル）

**デメリット**:
- ❌ CADとCAMの境界が曖昧になる
- ❌ 将来的にCAM専用機能が増えた際に肥大化
- ❌ `geo_algorithms`の責務が不明瞭（「幾何アルゴリズム」なのか「CAM」なのか）

**配置イメージ**:
```
model/
├── geo_algorithms/
│   ├── src/
│   │   ├── offset.rs       # CAD/CAM共通（オフセット演算）
│   │   ├── boolean.rs      # CAD専用（ブール演算）
│   │   ├── toolpath.rs     # CAM専用（工具経路）
│   │   ├── toolpath_gen.rs # CAM専用（経路生成）
│   │   └── collision.rs    # CAD/CAM共通（衝突判定）
```

---

### Option B: 新規 `cam` クレート作成（推奨）

**メリット**:
- ✅ CADとCAMの責務が明確に分離
- ✅ CAM専用機能の拡張が容易
- ✅ `geo_algorithms` は純粋な幾何アルゴリズムに専念
- ✅ ドメイン駆動設計（DDD）に沿った構成

**デメリット**:
- ❌ 新規クレート作成のオーバーヘッド
- ❌ クレート数が増加（管理コスト）
- ❌ CAD/CAM共通機能（オフセット等）の配置に悩む

**配置イメージ**:
```
model/
├── geo_algorithms/        # CAD幾何アルゴリズム
│   ├── src/
│   │   ├── offset.rs      # オフセット演算（CAM依存あり）
│   │   ├── boolean.rs     # ブール演算（CAD専用）
│   │   └── collision.rs   # 衝突判定（CAD/CAM共通）
│
├── cam/                   # CAM専用機能
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── toolpath.rs    # 工具経路データ構造
│   │   ├── path_gen/      # 経路生成アルゴリズム
│   │   │   ├── contour.rs # 等高線加工
│   │   │   ├── pocket.rs  # ポケット加工
│   │   │   └── drill.rs   # 穴あけ
│   │   ├── tool.rs        # 工具定義
│   │   ├── validation.rs  # 2D輪郭閉判定、トレランスチェック
│   │   └── optimization.rs # 経路最適化
```

---

### Option C: ハイブリッド案（段階的アプローチ）

**Phase 1（今回）**: `geo_algorithms` に配置
- まず `geo_algorithms/src/toolpath.rs` として実装
- CAM機能の規模を見極める

**Phase 2以降**: 必要に応じて `cam` クレート分離
- CAM機能が肥大化した時点で分離
- `geo_algorithms` からの移行として実施

**判断基準**:
- `toolpath.rs` 単体のみ → `geo_algorithms` で十分
- 経路生成、検証、最適化などが増えてきた → `cam` クレート分離

---

## 3. CADとCAMの境界定義

### RedRingにおける責務分離

#### CAD（Computer-Aided Design）の責務

**定義**: 形状の定義・編集・表現

**含まれる機能**:
- ✅ 幾何プリミティブ（点、線、円、NURBS等）
- ✅ 形状演算（オフセット、ブール演算、フィレット）
- ✅ 幾何計算（交差判定、距離計算）
- ✅ トポロジー管理（エッジ、フェース、ソリッド）
- ✅ 形状検証（閉曲線判定、自己交差チェック）

**クレート**:
- `geo_primitives`: 基本形状
- `geo_algorithms`: 形状演算・幾何計算
- `geo_nurbs`: NURBS
- Phase 4: `geo_topology`: トポロジー層（B-Rep）

---

#### CAM（Computer-Aided Manufacturing）の責務

**定義**: 加工戦略・工具経路生成・加工シミュレーション

**含まれる機能**:
- ✅ 工具経路生成（等高線、ポケット、穴あけ、輪郭加工）
- ✅ 工具定義（径、長さ、種別、切削条件）
- ✅ 加工条件（送り速度、回転数、切込み量）
- ✅ 経路最適化（順序、接続方式、干渉回避）
- ✅ 切削シミュレーション（材料除去、削り残し検出）
- ✅ NC データ生成（G コード出力）

**クレート候補**:
- `cam`: CAM専用機能（工具経路、加工戦略）
- `cam_sim`: 切削シミュレーション（将来）
- `cam_post`: ポストプロセッサ（G コード生成、将来）

---

#### 境界領域: CAD/CAM共通機能

**問題**: オフセット演算はCADかCAMか？

**結論**: **CAD側に配置、CAMから参照**

**理由**:
- オフセットは形状演算の一種（CAD機能）
- CAMはオフセット結果を**利用**して工具経路を生成
- 依存関係: `cam` → `geo_algorithms`（オフセット）

**共通機能の配置ルール**:
| 機能 | 配置先 | 理由 |
|------|--------|------|
| オフセット演算 | `geo_algorithms` | 形状演算（CAD） |
| ブール演算 | `geo_algorithms` | 形状演算（CAD） |
| 衝突判定 | `geo_algorithms` | 幾何計算（CAD） |
| トレランス検証 | `cam` | 加工精度要件（CAM） |
| 工具経路生成 | `cam` | 加工戦略（CAM） |
| 閉曲線判定 | `geo_algorithms` | 形状検証（CAD） |

---

## 4. 2D CAM対象形状の扱い

### 閉じた線群セグメントとしての判定

**2D CAM加工の典型例**:
- **輪郭加工**: 閉じた輪郭に沿って工具を移動
- **ポケット加工**: 閉じた領域内を材料除去

**必要な検証**:
1. ✅ **閉曲線判定**: 始点と終点が一致するか？
2. ✅ **自己交差判定**: 輪郭が自己交差していないか？
3. ✅ **方向性判定**: 時計回り/反時計回り（内側/外側の判定）

**実装配置**:
| 機能 | 配置先 | 理由 |
|------|--------|------|
| 閉曲線判定 | `geo_algorithms` | 形状検証（CAD機能） |
| 自己交差判定 | `geo_algorithms` | 幾何計算（CAD機能） |
| 輪郭方向判定 | `geo_algorithms` | 形状解析（CAD機能） |
| **CAM用検証** | `cam` | 加工可否判定（CAM機能） |

**CAM用検証の例**:
```rust
// cam/src/validation.rs

use geo_algorithms::curve_validation::{is_closed, has_self_intersection};

pub fn validate_2d_contour(
    segments: &[LineSegment2D<f64>],
    tolerance: f64,
) -> Result<(), ValidationError> {
    // CAD機能を利用
    if !is_closed(segments, tolerance) {
        return Err(ValidationError::NotClosed);
    }
    
    if has_self_intersection(segments) {
        return Err(ValidationError::SelfIntersection);
    }
    
    // CAM固有の検証
    if !is_manufacturable(segments, tolerance) {
        return Err(ValidationError::NotManufacturable);
    }
    
    Ok(())
}
```

---

### CAM用トレランスの必要性

**質問**: CAM用にトレランスを与える必要があるか？

**回答**: ✅ **必要**

**理由**:

#### 1. 加工精度の要件

CAMでは実際の機械加工を想定するため、数値誤差に対するトレランスが必須：

- **閉曲線判定**: 始点と終点の距離がトレランス以内か？
  - CAD: 厳密な一致（`start == end`）
  - CAM: トレランス許容（`distance(start, end) < tolerance`）

- **工具径との関係**: 工具径より小さい隙間は加工不可
  - トレランス = 工具径 × 0.1（例: φ10mm工具 → 1mm）

#### 2. NC機械の精度限界

実機の位置決め精度を考慮：
- 一般的な NC フライス: ±0.01mm
- 高精度機: ±0.001mm

**トレランス設定例**:
```rust
pub struct CamTolerance {
    /// 閉曲線判定トレランス（mm）
    pub closure_tolerance: f64,  // 例: 0.001
    
    /// 工具径に対する最小クリアランス
    pub tool_clearance_ratio: f64,  // 例: 0.1
    
    /// 機械精度
    pub machine_accuracy: f64,  // 例: 0.01
}
```

#### 3. トレランスの使用箇所

| 検証項目 | トレランス値 | 理由 |
|---------|-------------|------|
| 閉曲線判定 | 0.001mm | 数値誤差許容 |
| 自己交差判定 | 0.001mm | 微小な接触許容 |
| 工具干渉チェック | 工具径×0.1 | 最小クリアランス |
| 経路最適化 | 0.01mm | 機械精度範囲内の簡略化 |

---

## 推奨設計案

### ✅ Phase 1（Issue #203実装）の推奨構成

**クレート配置**: **Option B - 新規 `cam_core` クレート作成**（ユーザー承認済み）

**命名の意図**:
- `cam_core`: CAM機能の**基盤クレート**
  - 工具経路データ構造（ToolPath, PathSegment）
  - 工具定義（Tool, ToolType）
  - CAM用基本検証（トレランス、閉曲線判定）
  - 他のCAMクレート（将来）が依存する中核機能

**`geo_core` との違い**:
| 項目 | `geo_core` | `cam_core` |
|------|------------|-----------|
| 役割 | ブリッジパターン（Foundation トレイトと具体型の仲介） | CAM機能の基盤データ構造・アルゴリズム |
| 依存関係 | `geo_foundation` ↔ `geo_primitives` の橋渡し | CAM演算の中核、他CAMクレートの基盤 |
| 責務 | 抽象化・型変換 | 工具経路・工具定義・CAM検証 |
| 将来拡張 | 固定的（ブリッジ役） | 拡張的（`cam_algorithms`, `cam_sim` 等が依存） |

**命名の一貫性**:
- ✅ `geo_core` と対比して `cam_core` は自然
- ✅ 「基盤」を示す `_core` サフィックスの使用は適切
- ✅ 将来的なCAM関連クレート群の中核として明確

**将来のCAMクレート構成（Phase 2以降）**:
```
model/
├── cam_core/              # Phase 1: CAM基盤（工具経路、工具定義）
│   ├── src/
│   │   ├── toolpath.rs
│   │   ├── tool.rs
│   │   ├── validation.rs
│   │   └── tolerance.rs
│
├── cam_algorithms/        # Phase 2: 経路生成アルゴリズム
│   ├── src/
│   │   ├── contour.rs     # 等高線加工
│   │   ├── pocket.rs      # ポケット加工
│   │   ├── drill.rs       # 穴あけ
│   │   └── optimization.rs
│
├── cam_sim/               # Phase 3: 切削シミュレーション
│   ├── src/
│   │   ├── material.rs    # 材料除去
│   │   ├── collision.rs   # 干渉チェック
│   │   └── verification.rs
│
└── cam_post/              # Phase 4: ポストプロセッサ
    ├── src/
    │   ├── gcode.rs       # G コード生成
    │   └── machine.rs     # 機械固有設定
```

**依存関係ツリー**:
```
cam_post
  ↓
cam_sim
  ↓
cam_algorithms
  ↓
cam_core ← ViewModel/View層が直接参照
  ↓
geo_algorithms, geo_primitives, geo_foundation
```

**理由**:
1. **拡張性**: 将来的に複数のCAMクレートが `cam_core` に依存する構成が自然
2. **一貫性**: `geo_core` との命名パターンの統一
3. **明確性**: "core" は基盤・中核を意味し、Phase 1の位置づけが明確
4. **分離性**: CAD（geo系）とCAM（cam系）の境界が明瞭

**代替案との比較**:
| 命名 | メリット | デメリット | 評価 |
|------|---------|-----------|------|
| `cam` | シンプル | 将来拡張時に階層化しづらい | △ |
| `cam_core` | 基盤であることが明確、将来拡張に対応 | `geo_core` との役割の違いを理解する必要 | ⭐ **推奨** |
| `cam_foundation` | Foundation Pattern との一貫性 | CAMにはFoundation Patternを適用しない? | △ |
| `cam_base` | 基盤を示す | `_core` より一般的でない | △ |

**結論**: ✅ **`cam_core` を採用（違和感なし）**

---

**ディレクトリ構成（Phase 1）**:
**ディレクトリ構成（Phase 1）**:
```
model/
├── cam_core/                  # 新規クレート（Phase 1実装）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── toolpath.rs        # 工具経路データ構造
│   │   ├── tool.rs            # 工具定義
│   │   ├── validation.rs      # 2D輪郭検証
│   │   └── tolerance.rs       # トレランス管理
│   └── tests/
│       ├── toolpath_tests.rs
│       └── validation_tests.rs
```

**依存関係（Phase 1）**:
```toml
# model/cam_core/Cargo.toml
[package]
name = "cam_core"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
analysis = { path = "../../foundation/analysis" }
geo_foundation = { path = "../geo_foundation" }
geo_primitives = { path = "../geo_primitives" }
geo_algorithms = { path = "../geo_algorithms" }  # オフセット、閉曲線判定を利用
```（`cam_core` クレート）

#### 1. `model/cam_core/src/toolpath.rs`

工具経路データ構造（設計済み）:
- `PathSegment` enum
- `ToolPath` 構造体
- `ContourLevelPath` 構造体

#### 2. `model/cam_core/src/tool.rs`

工具定義（簡易版）:
- `Tool` 構造体
- `ToolType` enum

#### 3. `model/cam_core/src/validation.rs`

2D輪郭検証:
```rust
pub fn validate_2d_contour(
    segments: &[LineSegment2D<f64>],
    tolerance: &CamTolerance,
) -> Result<(), ValidationError>;
```

#### 4. `model/cam_core/src/tolerance.rs`

トレランス管理:
```rust
pub struct CamTolerance {
    pub closure_tolerance: f64,
    pub tool_clearance_ratio: f64,
    pub machine_accuracy: f64,
}
```

#### 5. `model/cam_core/src/lib.rs`

公開API:
```rust
pub mod toolpath;
pub mod tool;
pub mod validation;
pub mod tolerance;

// 主要型の再エクスポート
pub use toolpath::{ToolPath, PathSegment, ContourLevelPath};
pub use tool::{Tool, ToolType};
pub use validation::validate_2d_contour;
pub use tolerance::CamTolerance;
```

---

## 命名に関する補足説明

### `cam_core` の役割と `geo_core` との対比

#### `geo_core` の役割（既存）
- **目的**: Foundation Pattern のブリッジ
- **機能**: `geo_foundation` のトレイトと `geo_primitives` の具体型を仲介
- **パターン**: 抽象化層と実装層の橋渡し（アダプターパターン）

#### `cam_core` の役割（新規）
- **目的**: CAM機能の基盤データ構造とアルゴリズム
- **✅ 承認済み事項（2026年2月8日）

1. **設計方針の承認**:
   - ✅ クレート配置: Option B（新規 `cam_core` クレート作成）
   - ✅ 命名: `cam_core` を採用（違和感なし、将来拡張を見据えた適切な命名）
   - ⏳ CAD/CAM境界定義の最終確認
   - ⏳ トレランス要件の最終確認

2. **即座に実施（準備完了）**:
   - [ ] `model/cam_core` クレート作成
   - [ ] `Cargo.toml` にメンバー追加
   - [ ] Phase 1実装開始（toolpath.rs, tool.rs, validation.rs, tolerance.rs）

3. **Issue更新・新規作成（次ステップ）**:
   - [ ] Issue #203を更新（Phase 1のみに絞る、`cam_core` クレート使用を明記）
   - [ ] Issue #211, #212, #213を新規作成

---

## 実装開始準備チェックリスト

### Step 1: クレート作成

```bash
# ディレクトリ作成
mkdir -p model/cam_core/src
mkdir -p model/cam_core/tests

# Cargo.toml 作成
cat > model/cam_core/Cargo.toml << 'EOF'
[package]
name = "cam_core"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
analysis = { path = "../../foundation/analysis" }
geo_foundation = { path = "../geo_foundation" }
geo_primitives = { path = "../geo_primitives" }
geo_algorithms = { path = "../geo_algorithms" }
EOF

# lib.rs 作成
cat > model/cam_core/src/lib.rs << 'EOF'
//! CAM機能の基盤クレート
//! 
//! 工具経路データ構造、工具定義、CAM検証機能を提供します。

pub mod toolpath;
pub mod tool;
pub mod validation;
pub mod tolerance;

// 主要型の再エクスポート
pub use toolpath::{ToolPath, PathSegment, ContourLevelPath};
pub use tool::{Tool, ToolType};
pub use validation::validate_2d_contour;
pub use tolerance::CamTolerance;
EOF
```

### Step 2: Workspace 更新

```toml
# Cargo.toml (ルート) のメンバーに追加
members = [
  # ... 既存メンバー ...
  "model/cam_core",  # 追加
]✅ 承認済み

1. ✅ **クレート配置**: Option B（新規 `cam_core` クレート）を採用
2. ✅ **クレート命名**: `cam_core` を採用（違和感なし）

### ⏳ 最終確認待ち

3. **CAD/CAM境界**: 提案した責務分離で問題ありませんか？
   - オフセット演算は `geo_algorithms`（CAD）
   - 工具経路生成は `cam_core`（CAM）
   - 閉曲線判定は `geo_algorithms`（CAD形状検証）
   - CAM用検証は `cam_core`（CAM加工可否判定）

4. **2D輪郭検証**: 上記の配置でよいですか？

5. **トレランス**: CAM用トレランスは必要ですか？デフォルト値は適切ですか？
   - `closure_tolerance = 0.001mm`
   - `tool_clearance_ratio = 0.1`
   - `machine_accuracy = 0.01mm`

6. **Issue分割**: Issue #211-#213を新規作成してよいですか？

---

**状態**: クレート名承認済み、実装開始準備完了  
**次回更新**: 実装開始時（トレランス・CAD/CAM境界の最終確認後）
- ✅ `cam_core` は「CAM機能群の基盤・中核」として適切
- ✅ `geo_core` とは役割が異なるが、「中核」を示す `_core` の使用は一貫性がある
- ✅ 将来的なCAMクレート群の拡張を見据えた命名として違和感なし# 2. `model/cam/src/tool.rs`

工具定義（簡易版）:
- `Tool` 構造体
- `ToolType` enum

#### 3. `model/cam/src/validation.rs`

2D輪郭検証:
```rust
pub fn validate_2d_contour(
    segments: &[LineSegment2D<f64>],
    tolerance: &CamTolerance,
) -> Result<(), ValidationError>;
```

#### 4. `model/cam/src/tolerance.rs`

トレランス管理:
```rust
pub struct CamTolerance {
    pub closure_tolerance: f64,
    pub tool_clearance_ratio: f64,
    pub machine_accuracy: f64,
}
```

---

## 次のアクション

### 即座に実施（ユーザー承認待ち）

1. **設計方針の承認**:
   - [ ] クレート配置: Option B（新規`cam`クレート）の承認
   - [ ] CAD/CAM境界定義の承認
   - [ ] トレランス要件の承認

2. **Issue更新・新規作成**:
   - [ ] Issue #203を更新（Phase 1のみに絞る）
   - [ ] Issue #211, #212, #213を新規作成

3. **実装開始準備**:
   - [ ] `model/cam` クレート作成
   - [ ] `Cargo.toml` にメンバー追加
   - [ ] Phase 1実装開始

---

## 質問・確認事項

### ユーザーへの確認

1. **クレート配置**: Option A（geo_algorithms）、Option B（新規cam）、Option C（ハイブリッド）のどれを採用しますか？
   - **推奨**: Option B

2. **CAD/CAM境界**: 提案した責務分離で問題ありませんか？
   - オフセット演算は `geo_algorithms`（CAD）
   - 工具経路生成は `cam`（CAM）

3. **2D輪郭検証**: 閉曲線判定は `geo_algorithms`、CAM用検証は `cam` でよいですか？

4. **トレランス**: CAM用トレランスは必要ですか？デフォルト値は適切ですか？
   - `closure_tolerance = 0.001mm`
   - `tool_clearance_ratio = 0.1`

5. **Issue分割**: Issue #211-#213を新規作成してよいですか？

---

**承認待ち**: 上記の設計方針について、ユーザーからの承認をお待ちしています。

**次回更新**: 設計承認後、実装開始時
