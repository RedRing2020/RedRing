# モジュール構成 / Module Structure

RedRing は、責務分離と型安全性を重視したワークスペース設計に基づいて構成されています。以下は主要なクレート群の概要です。

## 幾何計算層

### `geo_foundation`

**抽象型・許容誤差管理・トレイト定義・橋渡し**

- 共通トレイト（`Normalizable`, `DistanceCalculation` など）
- `ToleranceContext` による許容誤差管理
- トレラント比較
- エラー処理ガイドライン
- 型安全な抽象化

### `geo_core`

** 基本図形処理・ロバスト幾何判定**

- 基本図形処理
- ロバスト幾何判定（orientation など）

### `geo_primitives`

**Foundation 統合済み幾何プリミティブ**

- 基本要素：`Point`, `Vector`, `Direction`（Core/Extensions 分離済み）
- 幾何形状：`LineSegment`, `Circle`, `Ellipse`, `Arc`（責務分離完了）
- 2D/3D 両対応のジェネリック実装
- Foundation 統合アーキテクチャによる保守性向上

### `geo_algorithms`

**幾何アルゴリズム**

- 交点計算
- 曲線・曲面操作
- 幾何変換

### `geo_nurbs`

**高次曲線・曲面**

- NURBS 曲線・曲面（今後実装予定）
- 複雑な幾何モデル
- 抽象化レイヤ

### `analysis`

**数値解析・汎用数値計算**

- 独立した汎用数値計算ライブラリ
- `Scalar`トレイト、許容誤差管理
- ドメイン非依存の数学的基盤
- 他のプロジェクトでも再利用可能

### `geo_io`

**ファイル I/O・境界層処理**

- STL、OBJ、PLY 等のファイル形式との変換
- **例外的設計**: `geo_foundation`トレイトを経由せず、直接`geo_primitives`にアクセス
- ゼロコピー最適化によるパフォーマンス重視
- 外部データ形式との効率的な境界処理

#### geo_io の例外設計根拠

```rust
// 他のgeo_*クレートは geo_foundation 経由
use geo_foundation::{Point3D, Vector3D};

// geo_ioのみ直接アクセス（例外パターン）
use geo_primitives::{Point3D, TriangleMesh3D, Vector3D};
```

この例外は以下の理由で採用：

- **パフォーマンス**: ファイル形式との直接変換でゼロコピー最適化
- **責務明確化**: I/O 境界層としての特化
- **実装複雑性回避**: 抽象化オーバーヘッドの排除

### `cam_solver`

**CAM 演算・パス作成 / 編集・ポスト処理・切削シミュレーション**

- CAM パス生成 / 編集（今後実装予定）
- 各 CNC コントローラー 対応ポスト処理（今後実装予定）
- 切削シミュレーション（今後実装予定）

### `data_exchange`

**データインポート/エクスポート（将来実装予定）**

- STL インポート/エクスポート
- OBJ インポート/エクスポート
- STEP インポート/エクスポート
- IGES インポート/エクスポート
- DXF インポート/エクスポート
- DWG インポート/エクスポート

_注記_: 現在は`geo_io`で STL 形式のみ実装済み

## レンダリング層

### `render`

**GPU 描画基盤**

- wgpu + WGSL による GPU レンダリング
- シェーダ管理
- 頂点データ処理

### `stage`

**レンダリングステージ管理**

- `RenderStage` トレイト
- レンダリングパイプライン管理
- シーン構成

### `viewmodel`

**ビュー操作・変換ロジック・MVVM アーキテクチャ**

- カメラ制御（将来実装予定）
- ビュー変換
- ユーザーインタラクション（将来実装予定）
- **MVVM 準拠**: Model 層（`geo_*`）と View 層（`render`）の架け橋
- STL 読み込み・GPU 変換機能（`stl_loader`）
- メッシュデータ変換（`mesh_converter`）

### `redring`

**メインアプリケーション**

- アプリケーションエントリポイント
- 全クレート統合
- ウィンドウ管理

## 構造設計の方針

- **型安全性**：コンパイル時エラー検出を最大化
- **責務分離**：各クレートが単一の責務を持つ
- **モジュール性**：`render` は `model` に依存しない設計
- **拡張性**：将来的な NURBS や WebAssembly 対応を考慮
- **国際化**：英語ベースの命名で国際的な貢献を促進
- **MVVM アーキテクチャ**：View → ViewModel → Model の層分離
- **例外設計許容**：パフォーマンス要件に応じた適切な例外設計（I/O 層など）

## アーキテクチャ依存関係

```
redring (View) → viewmodel → model (geo_*)
            ↘  stage → render

独立: analysis (汎用数値計算)
例外: geo_io (直接geo_primitives依存でゼロコピー最適化)
```
