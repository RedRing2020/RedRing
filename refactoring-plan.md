# RedRing リファクタリング総合案

## 1. プロジェクト構造の再編成

### 現在のワークスペース構成（更新版）

```
RedRing/
├── geo_foundation/     # 抽象化レイヤー：トレイト定義・型システム
├── geo_core/           # 幾何計算基盤：数値演算・許容誤差・ロバスト性
├── geo_primitives/     # 具体実装：基本幾何要素の実装
├── geo_algorithms/     # 高レベル幾何アルゴリズム
├── model/              # 既存：高次曲線・曲面（NURBS等）
├── analysis/           # 数値解析・線形代数・CAM処理
├── render/             # 既存：GPU描画基盤
├── stage/              # 既存：レンダリングステージ管理
├── viewmodel/          # 既存：UI/ビュー操作
├── redring/            # 既存：メインアプリケーション
└── redring-wasm/       # 将来：WebAssembly対応
```

### アーキテクチャ階層の明確化

**依存関係の方向:**
```
model → geo_algorithms → geo_primitives → geo_foundation ← geo_core
                                                      ↘     ↙
                                                        analysis
```

**各層の責務:**

1. **geo_foundation**: 抽象化・型システム
   - Core/Extension Foundation パターン
   - 共通トレイト定義（Normalizable, DistanceCalculation等）
   - 型安全な抽象化API

2. **geo_core**: 幾何計算基盤
   - 数値演算の実装
   - 許容誤差管理（ToleranceContext）
   - ロバスト幾何判定（orientation等）
   - スカラー型・ベクトル演算

3. **geo_primitives**: 具体実装
   - geo_foundationのトレイトの実装
   - geo_coreの計算基盤を活用
   - f64正準幾何プリミティブ

## 2. geo_foundation経由でのgeo_core活用計画

### Phase 1: geo_coreの数値基盤強化

- [ ] 許容誤差システムの完全実装（ToleranceContext拡張）
- [ ] ロバスト幾何判定の追加（orientation, incircle等）
- [ ] スカラー演算の最適化（SIMD対応検討）
- [ ] ベクトル演算ライブラリの統合

### Phase 2: geo_foundation抽象化層の拡張

- [ ] Core/Extension Foundation パターンの標準化
- [ ] 新しいトレイトの追加（曲線・曲面用）
- [ ] エラー型システムの統一
- [ ] 型安全なAPI設計の完成

### Phase 3: geo_primitives実装層の完成

- [ ] geo_foundationトレイトの完全実装
- [ ] geo_core計算基盤の活用
- [ ] パフォーマンス最適化
- [ ] 包括的テストスイートの構築

### Phase 4: 統合と最適化

- [ ] 全レイヤー間の依存関係最適化
- [ ] パフォーマンステストとボトルネック解析
- [ ] ドキュメント整備
- [ ] マイグレーションガイド作成

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

### 移行戦略（現在の設計に基づく）

1. **geo_foundation抽象化**: トレイト定義と型システムの確立
2. **geo_core基盤強化**: 数値演算とロバスト性の実装
3. **geo_primitives完成**: 具体実装の統合
4. **段階的移行**: 上位レイヤーの順次更新
5. **最終統合**: レガシーコードの削除とAPI統一

### 設計原則の実装

```rust
// geo_foundationでの抽象化例
pub trait GeometricPrimitive<T> {
    type Point;
    type Vector;
    fn bounding_box(&self) -> BoundingBox<T>;
}

// geo_coreでの計算基盤例
pub mod tolerance {
    pub fn tolerant_eq<T: Scalar>(a: T, b: T, context: &ToleranceContext) -> bool {
        (a - b).abs() <= context.linear_tolerance()
    }
}

// geo_primitivesでの具体実装例
impl GeometricPrimitive<f64> for Circle2D {
    type Point = Point2D;
    type Vector = Vector2D;
    
    fn bounding_box(&self) -> BoundingBox<f64> {
        // geo_coreの計算を活用
        tolerance::robust_bounding_box(self.center, self.radius)
    }
}

```rust
// model/src/lib.rs - 移行期間中の互換性レイヤー
pub use geo_foundation::primitives::{Point, Vector};
pub use geo_foundation::tolerance::ToleranceContext;

// 既存APIの維持（deprecated警告付き）
#[deprecated(note = "Use geo_foundation::primitives::Point instead")]
pub type Point3D = geo_foundation::primitives::Point3D;
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

### テストピラミッド（更新版）

```
geo_algorithms/     # 結合テスト（CAD操作シナリオ）
    ↑
geo_primitives/     # 統合テスト（具体実装）
    ↑           ↗
geo_foundation/   geo_core/    # 単体テスト（抽象化・計算基盤）
```

### 品質保証

- プロパティベーステスト（quickcheck）
- ベンチマークテスト（criterion）  
- Core/Extension Foundation パターン準拠チェック
- 回帰テスト（既存形状データ）

## 8. ドキュメント更新

### 技術文書

- [ ] アーキテクチャ図の更新（3層構造の明示）
- [ ] Core/Extension Foundation パターンガイド
- [ ] geo_foundation ↔ geo_core 連携パターン
- [ ] API リファレンスの再構築

### 開発者向け

- [ ] 新しい依存関係ガイドライン
- [ ] 許容誤差ベース設計パターン
- [ ] パフォーマンス最適化ガイド

## 実装優先度（現在の進捗に基づく）

### 🎯 現在完了済み

1. ✅ geo_foundation 抽象化層（Core/Extension Foundation パターン）
2. ✅ analysis クレート（Matrix4x4 3D変換ライブラリ）
3. ✅ 基本的なCI/CD準拠（禁止参照チェック）
4. ✅ 統一された許容誤差管理システム

### 🚀 High Priority (次の段階)

1. geo_core 計算基盤の強化
2. geo_primitives での geo_core 活用
3. 包括的テストスイートの構築
4. パフォーマンス最適化

### 📈 Medium Priority

1. geo_algorithms 高レベルアルゴリズム
2. model の NURBS 本格実装
3. 交差計算・テセレーション

### 🔮 Future Priority

1. WebAssembly 対応
2. SIMD 最適化
3. GPU 演算統合
4. ブール演算（将来機能）

## 追加考慮事項

### CI/CD 準拠

- ✅ geo_primitives 直接インポート禁止対応済み
- ✅ Core/Extension Foundation パターン統一済み
- geo_foundation 経由での依存関係パターン確立

### パフォーマンス戦略

- geo_core での高精度計算（許容誤差管理）
- SIMD 最適化パス（将来）
- メモリ効率的なバッファ管理

### 設計原則の進化

1. **依存性の方向性**: model → geo_algorithms → geo_primitives → geo_foundation ← geo_core
2. **抽象化レベル**: geo_foundation（抽象）→ geo_core（計算）→ geo_primitives（実装）
3. **品質保証**: 3層それぞれでの独立テスト + 結合テスト

この計画により、RedRing は CAD/CAM 用途に適した堅牢で高性能な幾何ライブラリとして発展していきます。
