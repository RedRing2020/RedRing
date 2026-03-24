# RedRing アーキテクチャ構成

**最終更新日**: 2026年3月15日

RedRing の幾何計算層とレンダリング層の構成、および現在の課題と解決策について説明します。

## 🧱 ワークスペース構成

### 幾何計算層

## 🚨 現在の重大な課題（2026年3月15日更新）

### 系統的な*_core_traits実装問題

10以上の形状で**レガシーAPIとFoundation実装が共存**し、以下の問題が発生しています：

- **メソッド名競合**: `center()`, `arc_length()`, `point_at_parameter()` 等
- **型不一致**: `Point2D<T>` vs `(T, T)` のシグネチャ違い
- **Foundation Patternの破綻**: 統一アクセスが実現できない

**対象形状**: point, vector, circle, ellipse_arc, direction, bbox, ray, line_segment, infinite_line

### アーキテクチャ再編提案（geo_foundation 廃止）

```text
foundation/analysis（将来改名候補）
            ↓
        geo_core（低レイヤー・基本型提供）
            ↓             ↓
   geo_primitives      geo_nurbs
      （trait+実装）     （trait+実装）
            ↘           ↙
            geo_algorithms（交差/衝突/幾何演算）
                    ↓
         application層クレート群
    （tessellation / simulation / job manager）
```

#### クレート責務定義

- **`foundation/analysis`**: 線形代数などの純粋な数値解析のみを提供（将来改名検討）
- **`geo_core`**: **低レイヤー基本型**（Aabb2D/Aabb3D等）- geo_primitives/geo_nurbsから直接アクセス可
- **`geo_primitives`**: プリミティブ形状のtrait定義と実装（Point, Vector, Circle等）
- **`geo_nurbs`**: NURBS形状のtrait定義と実装（Curve/Surface等）
- **`geo_algorithms`**: `geo_primitives` / `geo_nurbs` を利用した高レベル幾何アルゴリズム（intersect, collision 等）
- **`application::*`（新設方針）**: テセレーション、シミュレーション、ジョブ管理など業務ユースケース
- **`geo_io`**: ファイル I/O（STL/OBJ/PLY 等）

#### レイヤー設計の重要ポイント

- **`geo_core`は低レイヤー**: `geo_primitives`と`geo_nurbs`より下位に位置
- **直接アクセス許可**: `geo_core`からのインポート（特にAabb2D/Aabb3D）は許可
- **`geo_foundation`廃止（完了）**: 形状trait定義は`geo_contracts`へ統一済み
- **依存と import の区別**: `geo_algorithms -> geo_primitives/geo_nurbs` 依存は許可だが、`geo_algorithms` 実装ファイルでの `use geo_primitives::...` 直接 import は禁止（`use crate::...` 再エクスポート経由を使用）
- **上位責務分離**: tessellation/simulation/job managerは`geo_algorithms`より上位のapplication層へ集約
- **循環依存回避**: `geo_primitives` ↔ `geo_nurbs` の直接依存は禁止（交差処理は`geo_algorithms`に集約）

### 段階移行計画（提案）

1. **Phase 1: trait移設**
- `geo_foundation` の形状traitを `geo_contracts` へ移設
- 既存利用側を新trait参照へ置換

2. **Phase 2: アルゴリズム統合**
- `geo_algorithms` に `geo_primitives` と `geo_nurbs` の両依存を明示
- Intersect/Collision等のクロス形状演算を `geo_algorithms` に統一

3. **Phase 3: application層分離**
- tessellation/simulation/job manager を application層クレートに移動
- `geo_algorithms` は純粋幾何アルゴリズム責務に限定

4. **Phase 4: analysis再編**
- `foundation/analysis` の名称変更を検討
- 内部実装を純粋数値解析（線形代数等）に整理し、ドメイン責務を排除

### CAD/CAM 境界ルール（2026年2月更新）

- **原則**: `model/geo_*` から `model/cam_*` への依存は禁止
- **許可**: `cam_core -> cam_entity`
- **禁止**: `cam_core -> geo_entity`
- **禁止**: `geo_core -> cam_entity`
- **許可**: `geo_core -> geo_entity`

### 共通ジョブマネージャー方針（2026年3月更新）

- **新規共通クレート**: `model/job_runtime`
- **目的**: CAD/CAM/CAEに依存しない実行制御（submit/status/cancel/retry）を提供
- **責務**: `JobType`/`JobStatus`/`RetryPolicy`、状態遷移、イベント、Artifact参照契約
- **非責務**: CAM計算・切削シミュレーション等のドメイン計算ロジック本体
- **依存方針**:
    - `job_runtime` は `geo_*` / `cam_*` / View系へ依存しない
    - `cam_*` / 将来のCAE側は `job_runtime` をアダプタ経由で利用する

### Artifact API 命名運用ルール（2026年3月更新, #415）

- 運用原則: `*_v1` は wire format v1.0 を意味せず、API 世代識別子を表す
- wire format 判定原則: `version_major` / `version_minor` の実値で判定する（現行 v0.1）
- 改名方針: 現時点では公開 API 互換を優先し、実コード改名は行わない

#### 標準注記テンプレート

```text
注記: `*_v1` は API 世代識別子を表す。wire format の実バージョンは
`version_major` / `version_minor` の実値で判定する（現行は v0.1）。
```

#### 改名再評価トリガー

- `version_minor` 増加（例: v0.2）で API 名と wire format 名の混同が顕著化した場合
- `cam_sim` / 将来 `cam_algorithms` で命名誤解による障害が発生した場合
- 破壊的変更を許容できるリリース計画が確保された場合

### 依存チェック運用

- 実行コマンド: `powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1 -ExitOnError`
- CI/ローカルともに上記スクリプトで依存境界違反を検出する

## 🔧 修正方針

### レガシーAPIの段階的置き換え

1. **競合メソッドの内部化**: レガシー `pub fn` を `fn` に変更
2. **Foundation実装の真の移行**: 共存ではなく置き換えアプローチ
3. **最小限の実装から開始**: 1-2個のメソッドから段階的に実装
4. **geo_commons機能の活用**: 共通計算で内部実装を再利用

### 形状API統一の実現（新方針）

**目標**: 全てのアクセスを「各形状クレート内trait」経由に統一

```rust
// ✅ 目標: 形状クレート内trait経由のみアクセス可能
use geo_primitives::EllipseArc2DMeasure;

let arc = EllipseArc2D::new(...);
let length = arc.arc_length(); // trait実装を呼び出し
let point = arc.point_at_parameter(0.5); // trait実装を呼び出し

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

- **基本方針**: Vector/Point は f64 正準型、測定量は Scalar `<T>` 維持
- **Legacy型**: 全て削除済み、CI で deprecated symbols を deny
- **詳細履歴**: `MIGRATION_VECTOR_F64.md` を参照

## 📝 重要な教訓

1. **共存アプローチの失敗**: レガシーとFoundationの共存はメソッド名競合を引き起こす
2. **置き換えの必要性**: Foundation Patternの真の価値は統一アクセスにある
3. **段階的実装の重要性**: 一度に多数のメソッドを実装すると失敗する

## 🎆 期待される成果

- **統一アクセス**: 全てのAPIが各形状クレートのtrait経由
- **型安全性**: コンパイル時のインターフェース統一
- **保守性向上**: 明確な責務分離と依存関係
- **拡張性**: 新しい形状追加は各形状クレート、クロス演算は`geo_algorithms`に集約

## 🔗 関連ドキュメント

- **[📖 オンラインドキュメント](https://redring2020.github.io/RedRing/)** - GitHub Pages（自動更新）
- [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md) - 幾何抽象化の詳細仕様
- [`manual/philosophy.md`](manual/philosophy.md) - 設計思想・エラー処理ガイドライン
- [`MIGRATION_VECTOR_F64.md`](MIGRATION_VECTOR_F64.md) - f64 正準化移行履歴
- [`GITHUB_PAGES_SETUP.md`](GITHUB_PAGES_SETUP.md) - GitHub Pages 設定ガイド
- [`dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md`](dev/architecture/BATCH_COMPUTE_PLATFORM_DESIGN.md) - 夜間バッチ計算基盤（Dockerヘッドレス + Kubernetes）
- [`dev/architecture/GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md`](dev/architecture/GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md) - geo_algorithms の分割ルール（primitive_2d/3d/NURBS/pair_base の統一規約）
- [`dev/archive/issues/ISSUE_412_IMPLEMENTATION_PREP.md`](dev/archive/issues/ISSUE_412_IMPLEMENTATION_PREP.md) - artifact API命名整理の判断記録

