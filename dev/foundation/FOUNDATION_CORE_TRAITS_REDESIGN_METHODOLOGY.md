# Foundation Core Traits 再設計方法論

**作成日**: 2025年11月16日
**最終更新**: 2025年11月18日



## 概要

RedRing Foundation システムにおける Core Traits 統合設計の標準方法論。
Point2D/Point3D および Vector2D/Vector3D の実装を通じて確立した、全ての幾何形状に適用可能な統一設計パターンと実装手順を文書化。

## 🚨 **Foundation Pattern の強制実装ルール**

### **【絶対遵守】3-Function Pattern**

**ルール**: **全ての幾何図形は正確に3つのCore Traitsを実装する**

- **Constructor**: オブジェクト生成機能（`new()`, `try_new()`）
- **Properties**: 基本情報取得機能（座標アクセス、Analysis層変換）
- **Measure**: 計量・関係演算機能（距離、長さ、面積）
- **Transform**: 既存の`AnalysisTransform`トレイトを使用（共通化済み）

**❌ 禁止事項**:
- Legacy traits（`direction_traits`など）の継続使用
- Core Traitsの部分実装（3つ全て必須）
- 「動作するから変更不要」という判断
- アーキテクチャ移行の先送り

**🎯 強制理由**:
1. **アーキテクチャ統一**: 全図形で同一パターン
2. **予測可能性**: インターフェース統一
3. **型安全性**: 統一された戻り値型
4. **将来拡張性**: 統一基盤による機能追加容易性

## 確定した設計方針

### 2. ファイル命名規則

**理想的な構成** (レガシートレイト整理後):
```text
model/geo_foundation/src/core/
├── point_traits.rs
├── vector_traits.rs  
├── circle_traits.rs
└── {shape}_traits.rs
```

**現在の問題**: レガシートレイトが同名で存在し、理想的な手順を阻害

```text
model/geo_foundation/src/core/
├── {shape}_traits.rs        # 各形状の機能定義
├── transform.rs             # 共通Transform実装
└── transform_error.rs       # 変換エラー定義
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

### Step 1: トレイトファイル作成

```bash
# 新しい形状のトレイトファイルを作成
touch model/geo_foundation/src/core/{shape}_traits.rs

# geo_foundation/src/core/mod.rs に追加
pub mod {shape}_traits;
```

**注意**: レガシー`{shape}_traits.rs`と名前が重複する場合は、
一時的に`{shape}_new_traits.rs`とし、レガシー削除後にリネーム

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

**✅ 新パターン**: Core Traits実装はメインファイルに統合

```rust
// ✅ 正しい実装: geo_primitives/src/{shape}_2d.rs に直接実装
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

**❌ 禁止事項**:
- `{shape}_2d_core_traits.rs` などの分離ファイル作成
- レガシートレイトの継続使用
- 「動作するから変更不要」という判断

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

## 🚨 **CRITICAL RULES - AI開発者への強制ルール（実装違反防止）**

### **【絶対遵守】derive マクロ統一ルール**

**ルール**: **全ての幾何図形構造体は `#[derive(Debug, Clone, Copy, PartialEq)]` を使用する**

```rust
// ✅ 必須パターン - 例外なし
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct {Shape}2D<T: Scalar> { /* fields */ }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct {Shape}3D<T: Scalar> { /* fields */ }
```

**❌ 禁止事項**:
- 手動実装による `impl Debug/Clone/Copy/PartialEq`
- extensionsファイルでの重複実装
- 「動作するから良い」という判断
- 「既存コードを尊重」による統一性無視

**🎯 強制理由**:
1. **コード量削減**: 手動実装20行 → derive 1行
2. **保守性向上**: 統一性により予測可能
3. **標準準拠**: Rustコミュニティ慣例
4. **調査コスト削減**: 統一性により確認作業不要

### **【検証必須】統一性チェック**

**実装前チェック**:
```bash
# derive使用状況確認
grep -r "#\[derive.*Debug.*Clone.*Copy.*PartialEq" model/geo_primitives/src/

# 手動実装検出（禁止）
grep -r "impl.*Debug.*for\|impl.*Clone.*for\|impl.*Copy.*for\|impl.*PartialEq.*for" model/geo_primitives/src/
```

**違反発見時の対応**:
1. **即座に修正**: 手動実装 → derive マクロ
2. **統一性確保**: 全図形で同一パターン
3. **テスト実行**: 動作確認

### **【絶対遵守】Foundation Pattern ファイル配置ルール**

**ルール**: **Core Traits実装は必ず主ファイルに配置する**

```rust
// ✅ 正しいファイル配置
// model/geo_primitives/src/direction_3d.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction3D<T: Scalar> { /* fields */ }

impl<T: Scalar> DirectionConstructor<T> for Direction3D<T> { /* impl */ }
impl<T: Scalar> DirectionProperties<T> for Direction3D<T> { /* impl */ }
impl<T: Scalar> DirectionMeasure<T> for Direction3D<T> { /* impl */ }
```

**❌ 禁止事項**:
- `direction_3d_core_traits.rs` などの分離ファイル作成
- extensionsファイルでのCore Traits実装
- 「既存構造を維持」による分離ファイル継続使用

**🎯 強制理由**:
1. **Foundation Pattern遵守**: トレイト実装は主ファイル集約
2. **可読性向上**: 1ファイルで完結した理解
3. **保守性向上**: 分散実装による混乱回避
4. **アーキテクチャ一貫性**: 全図形で統一パターン

## 品質保証

### 必須チェック項目

1. **ビルド**: `cargo build` ✅
2. **Clippy**: `cargo clippy --workspace -- -D warnings` ✅
3. **テスト**: `cargo test --workspace` ✅
4. **型制約**: `Option<Self>`に`Sized`制約 ✅
5. **🚨 derive統一**: 全図形で `#[derive(Debug, Clone, Copy, PartialEq)]` 使用 ✅

### 実装完了の確認方法

```bash
# 全体品質チェック
cargo build && cargo clippy --workspace -- -D warnings && cargo test --workspace

# 形状別テスト
cargo test -p geo_primitives {shape_name}

# Core Traits機能テスト
cargo test {shape}_core_traits
```

## 実装指針

### 基本方針

1. **方法論遵守**: 3つのCore機能パターンを全形状で適用
2. **レガシー問題解決**: Foundation実装への置き換えアプローチ
3. **段階的実装**: 1形状ずつ確実に完成させる

### 形状実装時の考慮事項

各形状実装時は以下の特性を考慮:

- **Constructor**: 形状固有の生成パラメータ
- **Properties**: 座標・寸法・方向などの基本属性
- **Measure**: 形状間関係・距離・計量の演算
- **Transform**: 共通のAnalysisTransformトレイト使用

**レガシートレイトとの競合回避**:
- メソッド名の競合を事前確認
- レガシーメソッドの段階的内部化 (`pub fn` → `fn`)
- Foundationトレイト経由の統一アクセス実現

## 利点

1. **統一性**: Point/Vector実装との完全な一貫性
2. **保守性**: 1ファイル = 1形状のシンプルな構造
3. **型安全性**: 適切な制約とエラーハンドリング
4. **拡張性**: 新形状追加の標準手順確立
5. **Analysis統合**: foundation層との完全連携
