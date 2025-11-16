# RedRing 幾何システム設計計画

## 設計原則

### 1. アーキテクチャ層構造（2025年11月11日更新）

```text
┌─────────────────────────────────────┐
│ アプリケーション層                  │
│ (app, viewmodel, etc.)              │
├─────────────────────────────────────┤
│ geo_foundation                      │
│ - 抽象トレイト定義                  │
│ - インターフェース仕様              │
│ - 型安全性保証                      │
├─────────────────────────────────────┤
│ geo_core                            │
│ - Foundation ブリッジ役             │
│ - 抽象トレイトと具体型の仲介        │
│ - 交差判定・組み合わせ演算基盤      │
├─────────────────────────────────────┤
│ geo_primitives / geo_nurbs          │
│ - 具体的実装コード                  │
│ - 責務別ファイル分割                │
│ - Foundation パターン準拠           │
└─────────────────────────────────────┘
```

### 2. 呼び出しフロー（修正版）

1. **geo_foundation**: 抽象トレイト定義層
   - 全ての幾何プリミティブ共通のトレイト定義
   - 型安全性とインターフェース統一性を保証

2. **geo_core**: Foundation ブリッジ層
   - Foundation トレイトと具体型の仲介役
   - 上位クレート（geo_nurbs等）が具体型にアクセスする経路
   - 交差判定・組み合わせ演算の基盤

3. **geo_primitives / geo_nurbs**: 実装コード層
   - geo_foundation のトレイトを具体的に実装
   - geo_primitives: 基本幾何プリミティブ
   - geo_nurbs: NURBS 曲線・曲面（geo_core 経由でアクセス）

4. **正しい呼び出し**: geo_nurbs → geo_core → geo_foundation
   - geo_nurbs は geo_primitives を直接参照しない
   - geo_core 経由での安全なアクセス

## 3. 1 つの幾何形状あたり 9 ファイル構成（Foundation パターン対応）

### 現在の Arc2D の完全実装例

```text
{shape}_2d.rs                - Core実装（基本構造・必須API）
{shape}_2d_extensions.rs     - 汎用拡張機能
{shape}_2d_collision.rs      - 衝突検出専門
{shape}_2d_containment.rs    - 包含判定専門
{shape}_2d_intersection.rs   - 交差計算専門
{shape}_2d_metrics.rs        - 計量計算専門
{shape}_2d_sampling.rs       - サンプリング専門
{shape}_2d_transform.rs      - 変換操作専門
{shape}_2d_foundation.rs     - Foundation統一トレイト実装
{shape}_2d_tests.rs          - テスト（別カウント）
```

### 責務分離の詳細

| ファイル         | 責務         | 行数目標   | 実装内容                        |
| ---------------- | ------------ | ---------- | ------------------------------- |
| **Core**         | 基本構造     | 150-250 行 | 構築、アクセサ、基本操作        |
| **Extensions**   | 汎用拡張     | 100-200 行 | 便利メソッド、型変換            |
| **Collision**    | 衝突検出     | 150-200 行 | 交差判定、距離計算              |
| **Containment**  | 包含判定     | 80-120 行  | 点・図形包含判定                |
| **Intersection** | 交差計算     | 120-180 行 | 詳細交差ポイント計算            |
| **Metrics**      | 計量計算     | 60-100 行  | 距離、面積、周長等              |
| **Sampling**     | サンプリング | 100-150 行 | 点列生成、近似                  |
| **Transform**    | 変換操作     | 150-250 行 | 移動、回転、拡縮                |
| **Foundation**   | 統一トレイト | 50-100 行  | ExtensionFoundation, TolerantEq |

## 実装状況

### ✅ 完全実装済み（Foundation 対応）

- **3D 幾何プリミティブ Foundation**: 10 形状すべて完備
  - Arc3D, BBox3D, Circle3D, Cylinder3D, Point3D
  - Ray3D, Sphere3D, Triangle3D, TriangleMesh3D, Vector3D
  - ExtensionFoundation\<T\> + TolerantEq\<T\> 統一実装

### 🔄 部分実装済み（2D 系）

- **Arc2D**: 8 ファイル完備（合計 1,420 行）- Foundation 未対応
- **Circle2D**: 3 ファイル（Core + Extensions + Metrics、合計 861 行）
- **Ellipse2D**: 2 ファイル（Core + Extensions、合計 902 行）
- **Vector2D**: 2 ファイル（Core + Extensions）
- **Point2D**: 2 ファイル（Core + Extensions）
- **Direction2D**: 2 ファイル（Core + Extensions）
- **Ray2D**: 2 ファイル（Core + Extensions）

### 📋 今後の拡張計画

1. **2D 形状の Foundation 対応**（優先度：高）

   ```text
   各2D形状 + {shape}_2d_foundation.rs 追加
   ExtensionFoundation\<T\> + TolerantEq\<T\> 実装
   ```

2. **2D 形状の 9 ファイル構成完全化**
   ```text
   Circle2D: 861行 → 1,500-1,700行（+Foundation）
   Ellipse2D: 902行 → 1,300-1,600行（+Foundation）
   Vector2D: 推定400行 → 900-1,100行（+Foundation）
   Point2D: 推定300行 → 700-900行（+Foundation）
   ```

## 設計パターンの利点

### 1. 保守性

- 各責務が独立したファイルに分離
- 機能追加時の影響範囲限定
- コードレビューの効率化

### 2. 可読性

- ファイルサイズが適切（150-250 行程度）
- 責務が明確で理解しやすい
- 検索・編集の効率向上

### 3. 拡張性

- 新機能追加時の構造が統一
- トレイト実装の一貫性
- テストコードの分離

### 4. 型安全性

- geo_foundation による抽象化
- コンパイル時の型チェック
- インターフェース統一

## 実装ガイドライン

### ファイル命名規則

```text
{geometry_name}_2d.rs              - Core実装
{geometry_name}_2d_extensions.rs   - 拡張機能
{geometry_name}_2d_collision.rs    - 衝突検出
{geometry_name}_2d_containment.rs  - 包含判定
{geometry_name}_2d_intersection.rs - 交差計算
{geometry_name}_2d_metrics.rs      - 計量計算
{geometry_name}_2d_sampling.rs     - サンプリング
{geometry_name}_2d_transform.rs    - 変換操作
{geometry_name}_2d_foundation.rs   - Foundation統一トレイト実装
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

1. **2D 形状 Foundation 対応**: 全 2D 形状に foundation ファイル追加
2. **Circle2D 完全化**: 残り 6 ファイル追加（+Foundation）
3. **Ellipse2D 完全化**: 残り 7 ファイル追加（+Foundation）
4. **Vector2D 完全化**: 残り 7 ファイル追加（+Foundation）
5. **Point2D 完全化**: 残り 7 ファイル追加（+Foundation）
6. **1D・4D 形状対応**: 新次元での幾何プリミティブ拡張

## 品質指標

- **ファイルあたり行数**: 150-250 行（テスト除く）
- **責務純度**: 単一責務原則遵守
- **トレイト実装率**: geo_foundation 抽象化 100%対応
- **テストカバレッジ**: 各ファイルに対応するテスト完備

---

**最終更新**: 2025 年 10 月 26 日
**策定者**: RedRing 開発チーム
**承認状況**: Foundation パターン統合完了
