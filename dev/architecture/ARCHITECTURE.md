# RedRing アーキテクチャ構成

**最終更新日**: 2025年11月18日

RedRing の幾何計算層とレンダリング層の構成、および現在の課題と解決策について説明します。

## 🧱 ワークスペース構成

### 幾何計算層

## 🚨 現在の重大な課題（2025年11月18日更新）

### 系統的な*_core_traits実装問題

10以上の形状で**レガシーAPIとFoundation実装が共存**し、以下の問題が発生しています：

- **メソッド名競合**: `center()`, `arc_length()`, `point_at_parameter()` 等
- **型不一致**: `Point2D<T>` vs `(T, T)` のシグネチャ違い
- **Foundation Patternの破綱**: 統一アクセスが実現できない

**対象形状**: point, vector, circle, ellipse_arc, direction, bbox, ray, line_segment, infinite_line

### アーキテクチャ構成と責務

```text
analysis → geo_foundation
            ↓
        geo_commons
            ↓
        geo_core（ブリッジ役）
            ↓    ↓
   geo_primitives  geo_nurbs
```

#### クレート責務定義

- **`analysis`**: 数値解析・線形代数・微積分の基盤機能
- **`geo_foundation`**: 抽象トレイト定義（*_core_traits 等）
- **`geo_commons`**: 共通幾何計算機能、Foundation橋渡し  
- **`geo_primitives`**: プリミティブ幾何実装（Point, Vector, Circle等）
- **`geo_nurbs`**: NURBS 曲線・曲面実装
- **`geo_core`**: Foundation ブリッジ・交差判定基盤
- **`geo_algorithms`**: 高レベル幾何アルゴリズム
- **`geo_io`**: ファイル I/O（STL/OBJ/PLY 等）

## 🔧 修正方針

### レガシーAPIの段階的置き換え

1. **競合メソッドの内部化**: レガシー `pub fn` を `fn` に変更
2. **Foundation実装の真の移行**: 共存ではなく置き換えアプローチ
3. **最小限の実装から開始**: 1-2個のメソッドから段階的に実装
4. **geo_commons機能の活用**: 共通計算で内部実装を再利用

### Foundation Pattern の真の実現

**目標**: 全てのアクセスをFoundationトレイト経由に統一

```rust
// ✅ 目標: Foundation経由のみアクセス可能
use geo_foundation::EllipseArc2DMeasure;

let arc = EllipseArc2D::new(...);
let length = arc.arc_length(); // Foundation実装を呼び出し
let point = arc.point_at_parameter(0.5); // Foundation実装を呼び出し

// ❌ 禁止: レガシー直接アクセス
// arc.legacy_method() // コンパイルエラー
```

### レンダリング層

```text
redring ← stage ← render
       ↖ viewmodel
```

#### アプリケーション層の責務

- **`render`**: GPU 描画基盤（wgpu + WGSL）- 基本描画機能のみ
- **`stage`**: レンダリングステージ管理 - 最小限の構造のみ  
- **`viewmodel`**: ビュー操作・変換ロジック - 基礎機能のみ
- **`redring`**: メインアプリケーション - ウィンドウ表示のみ

**現在未実装の主要機能**:
- メニューシステム
- コマンドパレット
- ファイル操作（開く/保存）
- WebAssembly対応
- 実用的なCAD/CAM機能

## 🔄 f64正準化移行について

- **基本方針**: Vector/Point は f64 正準型、測定量は Scalar<T> 維持
- **Legacy型**: 全て削除済み、CI で deprecated symbols を deny
- **詳細履歴**: `MIGRATION_VECTOR_F64.md` を参照

## 📝 重要な教訓

1. **共存アプローチの失敗**: レガシーとFoundationの共存はメソッド名競合を引き起こす
2. **置き換えの必要性**: Foundation Patternの真の価値は統一アクセスにある
3. **段階的実装の重要性**: 一度に多数のメソッドを実装すると失敗する

## 🎆 期待される成果

- **統一アクセス**: 全てのAPIがFoundationトレイト経由
- **型安全性**: コンパイル時のインターフェース統一
- **保守性向上**: 明確な責務分離と依存関係
- **拡張性**: 新しい形状や機能の追加が容易

## 🔗 関連ドキュメント

- **[📖 オンラインドキュメント](https://redring2020.github.io/RedRing/)** - GitHub Pages（自動更新）
- [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md) - 幾何抽象化の詳細仕様
- [`manual/philosophy.md`](manual/philosophy.md) - 設計思想・エラー処理ガイドライン
- [`MIGRATION_VECTOR_F64.md`](MIGRATION_VECTOR_F64.md) - f64 正準化移行履歴
- [`GITHUB_PAGES_SETUP.md`](GITHUB_PAGES_SETUP.md) - GitHub Pages 設定ガイド
