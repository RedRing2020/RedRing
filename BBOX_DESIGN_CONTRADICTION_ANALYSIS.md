# 境界ボックス実装の設計矛盾分析

## 問題の構造

### 現在の状況 (修正前)

```
geo_foundation/             ← トレイト定義層
├── BBoxCore<T>            ← ✅ 正しい場所
├── BBox2DCore<T>          ← ✅ 正しい場所
└── basic_bbox.rs          ← ✅ 正しい場所

analysis/                   ← 数値計算基盤層
├── spatial/               ← ❌ 設計矛盾 (削除済み)
│   ├── bbox.rs           ← ❌ ジオメトリ処理なのに analysis にある
│   └── mod.rs            ← ❌ 「空間構造計算」という誤ったネーミング
└── lib.rs                 ← ❌ ジオメトリ機能を再エクスポート

geo_primitives/             ← 具体実装層
├── geometry2d/
│   └── bbox.rs            ← ✅ 正しい場所
└── geometry3d/
    └── bbox.rs            ← ✅ 正しい場所 (未実装)
```

### 理想的な設計

```
analysis/                   ← 純粋数値計算のみ
├── numerics/              ← 特殊数学定数
├── metrics/               ← 距離・長さ計算
└── approximations/        ← 数値近似

geo_foundation/             ← トレイト定義
├── BBoxCore<T>            ← 境界ボックストレイト
└── basic_bbox.rs          ← デフォルト実装

geo_primitives/             ← 具体実装
├── geometry2d/bbox.rs     ← 2D境界ボックス実装
└── geometry3d/bbox.rs     ← 3D境界ボックス実装
```

## 早まった実装の典型パターン

### 1. 設計時の思考プロセス (推測)

```
「BBoxCore トレイトを作った」
    ↓
「実装例が欲しい」
    ↓
「とりあえず analysis に計算関数を作っておこう」 ← ❌ ここが間違い
    ↓
「spatial モジュールという名前にしよう」 ← ❌ ジオメトリなのに analysis
```

### 2. 正しいプロセス

```
「BBoxCore トレイトを作った」
    ↓
「geo_primitives で具体実装を作る」 ← ✅ 正しい
    ↓
「必要に応じて analysis の数値計算機能を使用」 ← ✅ 適切な依存方向
```

## 設計原則違反の分析

### 1. 責務分離原則違反

```rust
// ❌ analysis に境界ボックス計算関数
pub fn bbox_2d_area<T: Scalar>(width: T, height: T) -> T {
    width * height  // これは単純な掛け算で「数値計算」ではない
}
```

### 2. 依存関係逆転

```
正しい依存: geo_primitives → geo_foundation → analysis
実際の依存: geo_foundation → analysis (spatial) → geo_primitives
```

### 3. モジュール命名の誤り

- `spatial` = 「空間構造計算」← ジオメトリ処理なのに数値計算っぽい名前

## 修正結果

### 削除したもの

- ✅ `analysis/src/spatial/` 全体
- ✅ `analysis/src/lib.rs` からの spatial 参照
- ✅ helpers.rs からの spatial 使用

### 保持するもの

- ✅ `geo_foundation` のトレイト定義
- ✅ `geo_primitives` の具体実装

## 学んだ教訓

### 1. トレイト設計と実装は分離する

- **トレイト定義**: 設計時に作成
- **具体実装**: 実装時に作成
- **ヘルパー関数**: 実装時に必要に応じて作成

### 2. 依存関係の方向を常に意識する

```rust
// ✅ 正しい: 上位層が下位層に依存
geo_primitives::BBox<T> → geo_foundation::BBoxCore<T> → analysis::Scalar

// ❌ 間違い: 下位層が上位層に依存
analysis → geo_foundation (spatial functions)
```

### 3. モジュール名は責務を正確に表現する

- `spatial` → ジオメトリ処理を連想させる ❌
- `numerics` → 数値計算を正確に表現 ✅

## 結論

**ユーザーの認識が 100%正確でした:**

1. ✅ **境界ボックスはジオメトリ処理** (数値計算ではない)
2. ✅ **geo_foundation にトレイト定義済み**
3. ✅ **geo_primitives に実装済み** (一部)
4. ✅ **analysis/spatial は早まった実装** (削除完了)

この事例は、設計段階でトレイトを作成した際に「実装例を示したい」という気持ちから、適切でない場所に部品実装を作ってしまった典型的な失敗パターンでした。
