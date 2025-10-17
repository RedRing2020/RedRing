# RedRing 幾何システム設計計画

## 設計原則

### 1. アーキテクチャ層構造

```
┌─────────────────────────────────────┐
│ アプリケーション層                  │
│ (redring, viewmodel, etc.)          │
├─────────────────────────────────────┤
│ geo_foundation                      │
│ - 抽象トレイト定義                  │
│ - インターフェース仕様              │
│ - 型安全性保証                      │
├─────────────────────────────────────┤
│ geo_primitives                      │
│ - 具体的実装コード                  │
│ - 責務別ファイル分割                │
│ - 8ファイル構成による専門化         │
└─────────────────────────────────────┘
```

### 2. 呼び出しフロー

1. **geo_foundation**: 抽象トレイト定義層
   - アプリケーションはこの層のトレイトを通じて幾何操作を呼び出す
   - 型安全性とインターフェース統一性を保証

2. **geo_primitives**: 実装コード層
   - geo_foundationのトレイトを具体的に実装
   - 8ファイル構成による責務分離

3. **幾何操作の呼び出し**: geo_foundation → geo_primitives
   - アプリケーションは直接geo_primitivesを使用しない
   - geo_foundationの抽象化層を経由

## 3. 1つの幾何形状あたり8ファイル構成

### 現在のArc2Dの完全実装例

```
{shape}_2d.rs                - Core実装（基本構造・必須API）
{shape}_2d_extensions.rs     - 汎用拡張機能
{shape}_2d_collision.rs      - 衝突検出専門
{shape}_2d_containment.rs    - 包含判定専門
{shape}_2d_intersection.rs   - 交差計算専門
{shape}_2d_metrics.rs        - 計量計算専門
{shape}_2d_sampling.rs       - サンプリング専門
{shape}_2d_transform.rs      - 変換操作専門
{shape}_2d_tests.rs          - テスト（別カウント）
```

### 責務分離の詳細

| ファイル | 責務 | 行数目標 | 実装内容 |
|---------|------|----------|----------|
| **Core** | 基本構造 | 150-250行 | 構築、アクセサ、基本操作 |
| **Extensions** | 汎用拡張 | 100-200行 | 便利メソッド、型変換 |
| **Collision** | 衝突検出 | 150-200行 | 交差判定、距離計算 |
| **Containment** | 包含判定 | 80-120行 | 点・図形包含判定 |
| **Intersection** | 交差計算 | 120-180行 | 詳細交差ポイント計算 |
| **Metrics** | 計量計算 | 60-100行 | 距離、面積、周長等 |
| **Sampling** | サンプリング | 100-150行 | 点列生成、近似 |
| **Transform** | 変換操作 | 150-250行 | 移動、回転、拡縮 |

## 実装状況

### ✅ 完全実装済み
- **Arc2D**: 8ファイル完備（合計1,420行）

### 🔄 部分実装済み
- **Circle2D**: 3ファイル（Core + Extensions + Metrics、合計861行）
- **Ellipse2D**: 2ファイル（Core + Extensions、合計902行）
- **Vector2D**: 2ファイル（Core + Extensions）
- **Point2D**: 2ファイル（Core + Extensions）
- **Direction2D**: 2ファイル（Core + Extensions）
- **Ray2D**: 2ファイル（Core + Extensions）

### 📋 今後の拡張計画

各形状を8ファイル構成に拡張：

```
Circle2D: 861行 → 1,400-1,600行（+500-700行）
Ellipse2D: 902行 → 1,200-1,500行（+300-600行）
Vector2D: 推定400行 → 800-1,000行（+400-600行）
Point2D: 推定300行 → 600-800行（+300-500行）
```

## 設計パターンの利点

### 1. 保守性
- 各責務が独立したファイルに分離
- 機能追加時の影響範囲限定
- コードレビューの効率化

### 2. 可読性
- ファイルサイズが適切（150-250行程度）
- 責務が明確で理解しやすい
- 検索・編集の効率向上

### 3. 拡張性
- 新機能追加時の構造が統一
- トレイト実装の一貫性
- テストコードの分離

### 4. 型安全性
- geo_foundationによる抽象化
- コンパイル時の型チェック
- インターフェース統一

## 実装ガイドライン

### ファイル命名規則
```
{geometry_name}_2d.rs              - Core実装
{geometry_name}_2d_extensions.rs   - 拡張機能
{geometry_name}_2d_collision.rs    - 衝突検出
{geometry_name}_2d_containment.rs  - 包含判定
{geometry_name}_2d_intersection.rs - 交差計算
{geometry_name}_2d_metrics.rs      - 計量計算
{geometry_name}_2d_sampling.rs     - サンプリング
{geometry_name}_2d_transform.rs    - 変換操作
{geometry_name}_2d_tests.rs        - テスト
```

### コード構造
```rust
//! {geometry_name} {responsibility} 実装
//!
//! Foundation統一システムに基づく {geometry_name} の {responsibility} 専門実装

use crate::{...};
use geo_foundation::Scalar;

impl<T: Scalar> {GeometryName}<T> {
    // 責務に特化したメソッド群
}
```

## 今後の作業優先順位

1. **Circle2D完全化**: 残り5ファイル追加
2. **Ellipse2D完全化**: 残り6ファイル追加  
3. **Vector2D完全化**: 残り6ファイル追加
4. **Point2D完全化**: 残り6ファイル追加
5. **3D形状対応**: 同様パターンで3D版実装

## 品質指標

- **ファイルあたり行数**: 150-250行（テスト除く）
- **責務純度**: 単一責務原則遵守
- **トレイト実装率**: geo_foundation抽象化100%対応
- **テストカバレッジ**: 各ファイルに対応するテスト完備

---

**最終更新**: 2025年10月16日  
**策定者**: RedRing開発チーム  
**承認状況**: 設計方針確定済み