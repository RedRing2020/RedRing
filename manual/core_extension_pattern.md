# Core/Extension Foundation パターン

RedRing の中核設計原則である Core/Extension Foundation パターンについて説明します。

## パターンの概要

幾何形状の機能を **Core（中核）** と **Extension（拡張）** に分離し、用途に応じて必要な機能のみを使用できる設計パターンです。

## 分離の方針

### Core Foundation（必須・高速）

- レンダリング・衝突判定・空間インデックスに必要な基本機能
- 軽量・高速・必須実装
- 構築、アクセサ、基本計量、基本包含、基本パラメータ、境界ボックス

### Extension Foundation（拡張・高機能）

- 高度な操作・分析・変換機能
- オプション実装・機能豊富
- 高度な構築、変形、空間関係、次元変換、コレクション操作

## ファイル構造

```text
circle_2d.rs              // Core実装（120行）
circle_2d_extensions.rs   // Extension実装（130行）
```

## 利用例

### Core のみ使用

```rust
use geo_foundation::Point2D;
use geo_foundation::Circle2D;
let circle = Circle2D::new(center, radius)?;
let area = circle.area();
```

### Extension を含む使用

```rust
let unit_circle = Circle2D::unit_circle();  // Extension
let scaled = circle.scale(2.0)?;            // Extension
```

### Analysis Transform（幾何変換拡張）

```rust
use analysis::linalg::vector::Vector3;
use geo_foundation::{AnalysisTransform3D, Angle};

// 平行移動
let translation = Vector3::new(1.0, 2.0, 3.0);
let translated = mesh.translate_analysis(&translation)?;

// 回転（軸回転）
let axis = Vector3::new(0.0, 0.0, 1.0);
let angle = Angle::from_degrees(90.0);
let rotated = mesh.rotate_analysis(&mesh, &axis, angle)?;

// 複合変換（平行移動 + 回転 + スケール）
let result = mesh.apply_composite_transform(
    Some(&translation),
    Some((&mesh, &axis, angle)),
    Some((2.0, 2.0, 2.0))
)?;
```

## メリット

1. **段階的実装**: 最小限から段階的に機能追加
2. **用途別最適化**: レンダリング用（軽量）vs 解析用（高機能）
3. **保守性向上**: 責務分離により理解・修正が容易
4. **拡張性**: 新しい Extension を後から追加可能

詳細は [CORE_EXTENSION_FOUNDATION_PATTERN.md](../CORE_EXTENSION_FOUNDATION_PATTERN.md) を参照してください。
