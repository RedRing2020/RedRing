# Foundation Core Traits 再設計方法論

**作成日**: 2025年11月16日  
**最終更新**: 2025年11月16日

## 概要

RedRing Foundation システムにおける Core Traits 統合設計の標準方法論。
Point2D/Point3D および Vector2D/Vector3D の実装を通じて確立した、全ての幾何形状に適用可能な統一設計パターンと実装手順を文書化。

## 確定した設計方針

### 1. Core 3機能統合パターン

**原則**: 各形状は3つのCore機能を1つのファイルに統合する

- **Constructor**: オブジェクト生成機能
- **Properties**: 基本情報取得機能  
- **Measure**: 計量・関係演算機能
- **Transform**: 既存の`AnalysisTransform`トレイトを使用（共通化済み）

### 2. ファイル命名規則

```text
model/geo_foundation/src/core/
├── point_core_traits.rs     ✅ 実装済み
├── vector_core_traits.rs    ✅ 実装済み
├── circle_core_traits.rs    📋 次期対象
├── line_core_traits.rs      📋 次期対象
├── arc_core_traits.rs       📋 次期対象
└── {shape}_core_traits.rs   📋 将来追加
```

### 3. Analysis層統合の必須要件

- `analysis::linalg`層との相互変換サポート
- `Vector2<T>`/`Vector3<T>`との統一インターフェース
- 既存の`AnalysisTransform`トレイトとの連携

## 実装された標準パターン

### 1. Constructor Traits - オブジェクト生成

**基本方針**: 形状固有の生成方法を提供

```rust
pub trait {Shape}2DConstructor<T: Scalar> {
    /// 基本コンストラクタ（形状固有パラメータ）
    fn new(...) -> Self;
    
    /// Analysis層からの変換
    fn from_analysis_vector(vector: &Vector2<T>) -> Self;
    
    /// 特殊値生成（形状に応じて）
    fn zero() -> Self;  // Vector用
    fn origin() -> Self; // Point用
    fn unit_circle() -> Self; // Circle用
}
```

### 2. Properties Traits - 基本情報取得

**基本方針**: 座標・成分・基本プロパティの統一アクセス

```rust
pub trait {Shape}2DProperties<T: Scalar> {
    // 座標・成分アクセス
    fn x(&self) -> T;
    fn y(&self) -> T;
    
    // Analysis層への変換
    fn to_analysis_vector(&self) -> Vector2<T>;
    
    // 基本プロパティ
    fn length(&self) -> T; // Vector用
    fn distance_from_origin(&self) -> T; // Point用
    fn radius(&self) -> T; // Circle用
    
    // 共通プロパティ
    fn dimension(&self) -> u32;
}
```

### 3. Measure Traits - 計量・関係演算

**基本方針**: 形状間の関係・距離・計量を提供

```rust
pub trait {Shape}2DMeasure<T: Scalar> {
    // 基本計量
    fn distance_to(&self, other: &Self) -> T;
    fn distance_squared_to(&self, other: &Self) -> T;
    
    // 形状固有計量
    fn area(&self) -> Option<T>;
    fn length(&self) -> Option<T>;
    
    // 関係演算（形状固有）
    fn dot(&self, other: &Self) -> T; // Vector用
    fn intersects(&self, other: &Self) -> bool; // Circle用
}
```

## 標準実装手順

### Step 1: Core Traitsファイル作成

```bash
# 新しい形状のCore traitsファイルを作成
touch model/geo_foundation/src/core/{shape}_core_traits.rs

# geo_foundation/src/core/mod.rs に追加
pub mod {shape}_core_traits;
```

### Step 2: 3つのCore機能を定義

```rust
//! {Shape} Core Traits - {Shape}形状の3つのCore機能統合
use crate::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

// 1. Constructor Traits
pub trait {Shape}2DConstructor<T: Scalar> {
    fn new(...) -> Self;
    fn from_analysis_vector(vector: &Vector2<T>) -> Self;
    // 形状固有の特殊値
}

// 2. Properties Traits  
pub trait {Shape}2DProperties<T: Scalar> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn to_analysis_vector(&self) -> Vector2<T>;
    fn dimension(&self) -> u32 { /* 形状固有の次元 */ }
    // 形状固有プロパティ
}

// 3. Measure Traits
pub trait {Shape}2DMeasure<T: Scalar> {
    fn distance_to(&self, other: &Self) -> T;
    fn area(&self) -> Option<T>;
    fn length(&self) -> Option<T>;
    // 形状固有の計量・関係演算
}

// 統合Trait
pub trait {Shape}2DCore<T: Scalar>:
    {Shape}2DConstructor<T> + {Shape}2DProperties<T> + {Shape}2DMeasure<T>
{
}
```

### Step 3: 型安全性の確保

**重要**: `Option<Self>`を返すメソッドには`where Self: Sized`制約を追加

```rust
fn try_normalize(&self) -> Option<Self>
where
    Self: Sized;

fn project_onto(&self, other: &Self) -> Option<Self>
where
    Self: Sized;
```

### Step 4: geo_primitives での実装

```rust
// geo_primitives/src/{shape}_2d.rs
impl<T: Scalar> {Shape}2DConstructor<T> for {Shape}2D<T> {
    fn new(...) -> Self {
        // 実装
    }
    
    fn from_analysis_vector(vector: &Vector2<T>) -> Self {
        // Analysis層からの変換
    }
}

impl<T: Scalar> {Shape}2DProperties<T> for {Shape}2D<T> {
    // プロパティ実装
}

impl<T: Scalar> {Shape}2DMeasure<T> for {Shape}2D<T> {
    // 計量・関係演算実装
}
```

## 技術的重要事項

### 型安全性の確保

**必須**: `Option<Self>`を返すメソッドには`where Self: Sized`制約

```rust
// ✅ 正しい実装
fn try_normalize(&self) -> Option<Self>
where
    Self: Sized;

// ❌ コンパイルエラー
fn try_normalize(&self) -> Option<Self>;
```

### Analysis層統合パターン

**必須**: 全ての形状でanalysis層との相互変換をサポート

```rust
// To Analysis
fn to_analysis_vector(&self) -> Vector2<T>;

// From Analysis  
fn from_analysis_vector(vector: &Vector2<T>) -> Self;
```

### Transform機能の統一化

**確定方針**: 既存の`AnalysisTransform2D<T>`/`AnalysisTransform3D<T>`トレイトを使用

- Core traitsにTransformは含めない
- 変換は既存の共通トレイトで提供済み
- `extensions/transform.rs` → `core/transform.rs` に移動済み

## 品質保証

### 必須チェック項目

1. **ビルド**: `cargo build` ✅
2. **Clippy**: `cargo clippy --workspace -- -D warnings` ✅  
3. **テスト**: `cargo test --workspace` ✅
4. **型制約**: `Option<Self>`に`Sized`制約 ✅

### 実装完了の確認方法

```bash
# 全体品質チェック
cargo build && cargo clippy --workspace -- -D warnings && cargo test --workspace

# トレイトの動作確認
cargo test -p geo_primitives {shape}_core_traits
```

## 次の実装対象

### 優先順位

1. **Circle Core Traits** - 最も使用頻度の高い基本図形
2. **Line Core Traits** - 直線・線分の統一インターフェース
3. **Arc Core Traits** - 円弧・楕円弧の扱い

### 各形状の特殊考慮事項

- **Circle**: 中心点・半径、面積計算、交差判定
- **Line**: 方向ベクトル、距離計算、平行・垂直判定  
- **Arc**: 開始・終了角度、弧長計算、角度範囲判定

## 利点

1. **統一性**: Point/Vector実装との完全な一貫性
2. **保守性**: 1ファイル = 1形状のシンプルな構造
3. **型安全性**: 適切な制約とエラーハンドリング
4. **拡張性**: 新形状追加の標準手順確立
5. **Analysis統合**: foundation層との完全連携
