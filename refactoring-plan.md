# RedRing リファクタリング総合案

## 1. プロジェクト構造の再編成

### 新しいワークスペース構成

```
RedRing/
├── geo-core/           # 新規：幾何計算基盤
├── model/              # 既存：具体的形状実装（リファクタ後）
├── algorithms/         # 新規：高レベル幾何アルゴリズム
├── render/             # 既存：GPU描画（軽微な修正）
├── stage/              # 既存：レンダリング管理
├── viewmodel/          # 既存：UI/ビュー操作
├── redring/            # 既存：メインアプリケーション
└── redring-wasm/       # 新規：WebAssembly対応
```

## 2. geo-core への機能移行計画

### Phase 1: 基礎インフラ

- [ ] `model/src/analysis/consts.rs` → `geo-core/src/tolerance/constants.rs`
- [ ] `model/src/analysis/numeric.rs` → `geo-core/src/robust/solvers.rs`
- [ ] 新規：`geo-core/src/scalar/` モジュール
- [ ] 新規：`geo-core/src/vector/` モジュール（2D/3D 統合）

### Phase 2: 幾何要素の抽象化

- [ ] `model/src/geometry/geometry3d/point.rs` → `geo-core/src/primitives/point.rs`
- [ ] `model/src/geometry/geometry3d/vector.rs` → `geo-core/src/vector/vector3d.rs`
- [ ] `model/src/geometry/geometry3d/direction.rs` → `geo-core/src/vector/direction.rs`
- [ ] トレイト統合：`geo-core/src/primitives/traits.rs`

### Phase 3: 許容誤差ベース比較

- [ ] 全体的な EPSILON 定数の許容誤差コンテキストへの移行
- [ ] `TolerantEq`, `TolerantOrd` トレイトの実装
- [ ] 既存の `==`, `<` 比較の置き換え

## 3. algorithms クレートの提案

### 目的

- 高レベルな幾何アルゴリズム
- CAD/CAM 固有の操作
- クロスプラットフォーム対応

### 構成案

```rust
algorithms/
├── intersection/       # 交差計算
│   ├── curve_curve.rs
│   ├── curve_surface.rs
│   └── surface_surface.rs
├── tessellation/       # テセレーション
│   ├── adaptive.rs
│   └── uniform.rs
├── offset/            # オフセット計算
│   ├── curve_offset.rs
│   └── surface_offset.rs
├── boolean/           # ブール演算（将来）
└── meshing/           # メッシュ生成（将来）
```

## 4. WebAssembly 対応 (redring-wasm)

### 目的

- ブラウザでの CAD ビューア
- クロスプラットフォーム展開

### 技術スタック

```rust
redring-wasm/
├── Cargo.toml         # wasm-pack, wee_alloc
├── src/
│   ├── lib.rs         # wasm-bindgen エントリーポイント
│   ├── viewer.rs      # ブラウザ用ビューア
│   └── geometry_js.rs # JavaScript API
├── www/               # フロントエンド
│   ├── index.html
│   ├── index.js
│   └── package.json
└── README.md
```

## 5. 既存コードの段階的移行

### 移行戦略

1. **geo-core クレート作成**: 基本インフラから開始
2. **デュアル実装期間**: 既存コードと新コードを並行維持
3. **段階的置き換え**: モジュール単位で新実装に移行
4. **テスト駆動**: 各段階で回帰テスト実施
5. **最終統合**: 旧実装の削除と API クリーンアップ

### 互換性維持

```rust
// model/src/lib.rs - 移行期間中の互換性レイヤー
pub use geo_core::primitives::{Point, Vector};
pub use geo_core::tolerance::ToleranceContext;

// 既存APIの維持（deprecated警告付き）
#[deprecated(note = "Use geo_core::primitives::Point instead")]
pub type Point3D = geo_core::primitives::Point3D;
```

## 6. パフォーマンス最適化

### SIMD 対応

```rust
// geo-core の内部実装でSIMD活用
use std::simd::*;

impl Vector3D {
    #[cfg(target_feature = "avx2")]
    fn dot_simd(&self, other: &Self) -> f64 {
        // AVX2を使用した高速内積計算
    }
}
```

### メモリ効率化

- Copy 型の積極活用（Point, Vector 等）
- アロケーション最小化
- キャッシュ効率を考慮したデータ構造

## 7. テスト戦略

### テストピラミッド

```
algorithms/     # 結合テスト（CAD操作シナリオ）
    ↑
model/          # 統合テスト（形状操作）
    ↑
geo-core/       # 単体テスト（基本演算）
```

### 品質保証

- プロパティベーステスト（quickcheck）
- ベンチマークテスト（criterion）
- 回帰テスト（既存形状データ）

## 8. ドキュメント更新

### 技術文書

- [ ] アーキテクチャ図の更新
- [ ] API リファレンスの再構築
- [ ] マイグレーションガイドの作成

### 開発者向け

- [ ] コントリビューションガイドライン
- [ ] コーディング規約（許容誤差関連）
- [ ] パフォーマンスガイドライン

## 実装優先度

### High Priority (Phase 1-2)

1. geo-core 基本インフラ
2. 許容誤差システム
3. ベクトル演算統合
4. 基本幾何要素の移行

### Medium Priority (Phase 3)

1. algorithms クレート基盤
2. 交差計算アルゴリズム
3. パフォーマンス最適化

### Low Priority (Phase 4+)

1. WebAssembly 対応
2. SIMD 最適化
3. ブール演算（将来機能）
