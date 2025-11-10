# RedRing アーキテクチャ構成

RedRing の幾何計算層とレンダリング層の構成について説明します。

## 🧱 ワークスペース構成

### 幾何計算層

```
analysis → geo_foundation → geo_primitives
                ↓              ↓
           geo_core      geo_nurbs
                ↓              ↓
         geo_algorithms  geo_io
```

| クレート         | 責務                                      | 状態        |
| ---------------- | ----------------------------------------- | ----------- |
| `analysis`       | 数値解析・線形代数・微積分                | ✅ 実装済み |
| `geo_foundation` | 抽象型・トレイト定義（BasicTransform等）  | ✅ 実装済み |
| `geo_primitives` | プリミティブ幾何専用（独自Transform実装） | ✅ 実装済み |
| `geo_nurbs`      | NURBS幾何専用（独自Transform実装）       | ✅ 実装済み |
| `geo_core`       | 交差判定等・全幾何組み合わせ実装用        | � 開発中   |
| `geo_algorithms` | 高レベル幾何アルゴリズム                  | 📋 計画中   |
| `geo_io`         | ファイル I/O（STL/OBJ/PLY 等）           | 📋 計画中   |

### レンダリング層

```
redring ← stage ← render
       ↖ viewmodel
```

| クレート    | 責務                        | 状態        |
| ----------- | --------------------------- | ----------- |
| `render`    | GPU 描画基盤（wgpu + WGSL） | ✅ 実装済み |
| `stage`     | レンダリングステージ管理    | ✅ 実装済み |
| `viewmodel` | ビュー操作・変換ロジック    | ✅ 基本実装 |
| `redring`   | メインアプリケーション      | ✅ 実装済み |

## 🔄 移行ステータス (f64 Canonical Geometry)

| 項目                    | 状態    | 説明                                      |
| ----------------------- | ------- | ----------------------------------------- |
| Vector/Point f64 化     | ✅ 完了 | `.value()` 呼び出し不要                   |
| 3D 基本プリミティブ抽出 | ✅ 完了 | Foundation 統合型に統一                   |
| Foundation 責務分離     | ✅ 完了 | Core/Extensions 分離による保守性向上      |
| Legacy 削除フェーズ     | ✅ 完了 | 旧 Legacy\* 型削除、CI で deprecated deny |

詳細な移行履歴と予定は `MIGRATION_VECTOR_F64.md` の末尾「Core Role Realignment」を参照してください。

## ✅ 互換性ポリシー

- すべての Legacy 型は削除されました。`geo_primitives` から f64 正準型をご利用ください。
- CI で deprecated symbols が deny されるため、古い Legacy 型の使用はビルドエラーとなります。
- f64 正準層では座標アクセサは全て `f64` を返却し、距離/面積など測定量のみ `Scalar` (単位意味付け) を維持。

## 🧪 テスト戦略

- f64 ベース幾何 (ベクトル / 点 / 方向 / 線分 / 平面 / 円) に最小ユニットテストを追加済み。
- 今後: レガシー排除前に alias 経由 API の smoke test を追加予定。

## 🔗 関連ドキュメント

- **[📖 オンラインドキュメント](https://redring2020.github.io/RedRing/)** - GitHub Pages（自動更新）
- [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md) - 幾何抽象化の詳細仕様
- [`manual/philosophy.md`](manual/philosophy.md) - 設計思想・エラー処理ガイドライン
- [`MIGRATION_VECTOR_F64.md`](MIGRATION_VECTOR_F64.md) - f64 正準化移行履歴
- [`GITHUB_PAGES_SETUP.md`](GITHUB_PAGES_SETUP.md) - GitHub Pages 設定ガイド
