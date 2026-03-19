# Issue #318 実施準備チェックリスト

対象Issue: [#318 geo_foundation廃止完了: geo_contracts導入と参照置換](https://github.com/RedRing2020/RedRing/issues/318)

## 1. 目的

- `geo_foundation` を最終的に削除できる状態まで持っていく
- 形状契約の正規参照先を `geo_contracts` に一本化する
- 数値抽象は `analysis`、Transform共通責務は `geo_core` に分離する

## 2. 現在地（2026-03-19 更新）

- `geo_contracts` には以下の形状契約が既に移設済み（#320等で整備）:
  - `arc_traits`, `circle_traits`, `point_traits`, `vector_traits`, `triangle_traits`
  - `nurbs_curve_2d_traits`, `nurbs_curve_3d_traits`, `nurbs_surface_3d_traits`
  - `plane3d_traits`, `ray_traits`, `infinite_line_traits`
  - `geometry::operations::*` (collision, intersection, EllipseCalculation等)
  - `classification`, `entity` (EntityDisplayProperties 等)
- `geo_core -> geo_foundation` 依存解消は #317 で完了済み
- Transform の `geo_core` 基準化は #319 で実装側ほぼ完了。Extension Traits は #332 で整理完了
- `geo_foundation` Cargo依存は現時点で 5 クレートに残存（cam_core は #332 で解消済み）:
  - `geo_primitives`, `geo_nurbs`, `geo_algorithms`, `geo_io`, `geo_entity`
  - `viewmodel/converter` にも `geo_foundation` 参照が残存
- `geo_primitives` の `ellipse_*` は #320 で `geo_commons` 経由へ切替済み
- `geo_contracts` にまだ移設されていない形状契約:
  - `direction_traits`, `ellipse_traits`, `ellipse_arc_traits`, `linesegment_traits`
  - `rectangle_traits`, `spherical_solid_traits`, `spherical_surface_traits`
  - `conical_solid_traits`, `conical_surface_traits`
  - `cylindrical_solid_traits`, `cylindrical_surface_traits`
  - `ellipsoidal_solid_traits`, `ellipsoidal_surface_traits`
  - `torus_solid_traits`, `torus_surface_traits`

## 3. geo_foundation 公開 API の分類方針

## 3.0 固定境界表（本Issueでの判断基準）

| 領域 | 所有クレート | 置くもの | 置かないもの | 代表例 |
|---|---|---|---|---|
| 数値抽象 | analysis | 数値型抽象、定数、許容誤差定数 | 形状意味を持つ trait | Angle, Scalar, TolerantEq |
| 幾何契約 | geo_contracts | 形状定義、形状の最小契約 | 変換アルゴリズム、衝突判定、交差判定の処理本体 | Point2D/Point3D, Vector2D/Vector3D の契約、Circle/Arc 契約 |
| 共通変換核 | geo_core | 形状非依存の変換 trait とエラー、共通変換補助 | 形状固有の再構築ロジック | AnalysisTransform2D/3D, TransformError |
| 形状実装 | geo_primitives / geo_nurbs | 形状固有の実装、形状固有制約、再構築ロジック | 共通契約の再定義 | Circle2D の回転適用、NURBS のノット更新 |
| 横断演算 | geo_algorithms | 複数形状にまたがる計算アルゴリズム | 形状型の所有 | 交差探索、空間探索、サンプリング |

## 3.0.1 point / vector の配置ルール

- Point と Vector は geo_contracts の「幾何契約」に配置する
- Point と Vector の計算実装は geo_core（共通核）または各実装クレートに置く
- Point/Vector の contract に transform の処理本体を持たせない

## 3.0.2 transform の配置ルール（曖昧化防止）

| transform 要素 | 所有クレート | 判断基準 |
|---|---|---|
| 変換 trait 定義 | geo_core | 形状名が不要で、座標変換の抽象のみを表す |
| 変換エラー型 | geo_core | どの形状にも共通の失敗モデルである |
| 変換行列の共通適用補助 | geo_core | 形状固有パラメータに触れない |
| 形状別 transform 実装 | geo_primitives / geo_nurbs | 半径・法線・ノットなど形状固有パラメータ更新がある |
| transform 実行時の許容誤差定数 | analysis（定数） + geo_core（利用） | 値の定義は analysis、適用は geo_core |

## 3.0.3 判定ルール（同じ議論を繰り返さないための固定文）

- trait 名やメソッドに形状固有語が必要なら geo_core ではなく実装側に置く
- trait 本体が形状固有パラメータへアクセスするなら geo_core ではなく実装側に置く
- 3形状以上で同一意味・同一エラーモデルで使えるなら geo_core を優先する
- 判定が割れた場合は「共通化しない」をデフォルトにする

### 3.1 analysis 所有で確定しているもの

- `Angle`
- `Scalar`
- `TolerantEq`
- 数学定数・許容誤差定数
  - `PI`, `TAU`, `DEG_TO_RAD`, `RAD_TO_DEG`
  - `GEOMETRIC_DISTANCE_TOLERANCE`, `GEOMETRIC_ANGLE_TOLERANCE`

### 3.2 geo_core 所有で確定しているもの

- `AnalysisTransform2D`
- `AnalysisTransform3D`
- `AnalysisTransformSupport`
- `AnalysisTransformVector2D`
- `AnalysisTransformVector3D`
- `TransformError` の共通核
- `SafeTransform` の共通変換責務

### 3.3 geo_contracts へ移す対象

- `classification`
  - `DimensionClass`
  - `GeometryPrimitive`
  - `PrimitiveKind`
- `geometry::core::*_traits`
  - 形状の `Constructor`
  - 形状の `Properties`
  - 形状の `Measure`
  - 形状の `Core`
- NURBS 契約
  - `NurbsCurve2D*`
  - `NurbsCurve3D*`
  - `NurbsSurface3D*`

### 3.4 所有先を要判断とするもの

- `entity::core`
  - `EntityDisplayProperties`
  - `EntityIdentity`
  - `LineEntity3DProperties`
- `geometry::foundation::extension_foundation`
  - `Bounded`
  - `ExtensionFoundation`
  - `CollectionExtension`
  - `MeasurableExtension`
  - `SpatialExtension`
  - `TransformableExtension`
- `geometry::extensions`
  - `BasicCollision`
  - `BasicIntersection`
  - `MultipleIntersection`
  - `BooleanOperations`
  - `PointDistance`
  - `SelfIntersection`
- `tolerance`
  - `GeometryContext`
  - `ToleranceSettings`
- `tolerance_migration`
  - 将来削除前提のため、新設先へは移さず撤去方針を優先

## 4. 依存元クレート別の残存パターン

### 4.1 geo_primitives

- 形状契約参照が大量に残存（200箇所超）
- `extensions::*` / `core::*_traits` / `tolerance_migration::*` の参照が混在
- `src/lib.rs` の公開 re-export で `geo_foundation` を正規ルートとして残している
- `ellipse_*` は #320 で一部 `geo_commons` 経由へ切替済み

### 4.2 geo_nurbs

- NURBS 契約の `Constructor` / `Properties` / `Measure` 参照が残存
- README とテスト内 import に旧経路が残存

### 4.3 geo_algorithms

- 主に `Scalar` / `ToleranceSettings` / `ToleranceContext` を使用
- 一部で NURBS 契約参照も残存
- 形状契約よりも「数値抽象と許容誤差」の切替が先行課題

### 4.4 geo_io

- `Scalar`
- `Triangle3DProperties`

### 4.5 geo_entity

- `EntityDisplayProperties`
- `EntityIdentity`
- `LineEntity3DProperties`
- `Scalar`

### 4.6 cam_core（解消済み）

- #332 で `Cargo.toml` 依存と実コード参照を除去済み

### 4.7 viewmodel/converter

- `geo_foundation::contracts::NurbsCurve3D*` / `NurbsSurface3D*` 参照が残存（6箇所）
- `Triangle3DProperties`, `LineEntity3DProperties`, `Scalar` 等も残存

## 5. 推奨実施順

### Phase 0: 契約の棚卸し表を確定

- [x] `geo_foundation/src/lib.rs` の公開項目を、所有先ごとに表形式で確定
- [x] `要判断` 項目について、暫定所属を決める

**棚卸し結果（2026-03-19確定）**:

| 公開項目 | 現所在(geo_foundation) | 移設先 | ステータス |
|---|---|---|---|
| `Angle`, `Scalar`, `TolerantEq` | analysis 再エクスポート | analysis | ✅ 既に analysis が正 |
| `PI`, `TAU`, `DEG_TO_RAD`, 定数群 | analysis 再エクスポート | analysis | ✅ 既に analysis が正 |
| `PrimitiveKind`, `DimensionClass`, `GeometryPrimitive` | classification | geo_contracts | ✅ geo_contracts に移設済み |
| `Arc2D*`, `Arc3D*` | geometry::core::arc_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Circle2D*`, `Circle3D*` | geometry::core::circle_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Point2D*`, `Point3D*` | geometry::core::point_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Vector2D*`, `Vector3D*` | geometry::core::vector_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Triangle2D*`, `Triangle3D*` | geometry::core::triangle_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `NurbsCurve2D*`, `NurbsCurve3D*`, `NurbsSurface3D*` | geometry::core::nurbs_*_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Plane3D*` | geometry::core::plane_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Ray2D*`, `Ray3D*` | geometry::core::ray_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `InfiniteLine2D*`, `InfiniteLine3D*` | geometry::core::infinite_line_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Direction2D*`, `Direction3D*` | geometry::core::direction_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Ellipse2D*`, `Ellipse3D*` | geometry::core::ellipse_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `EllipseArc2D*`, `EllipseArc3D*` | geometry::core::ellipse_arc_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `LineSegment2D*`, `LineSegment3D*` | geometry::core::linesegment_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `Rect2D*`, `Rect3D*` | geometry::core::rectangle_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `SphericalSolid3D*`, `SphericalSurface3D*` | geometry::core::spherical_*_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `ConicalSolid3D*`, `ConicalSurface3D*` | geometry::core::conical_*_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `CylindricalSolid3D*`, `CylindricalSurface3D*` | geometry::core::cylindrical_*_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `EllipsoidalSolid3D*`, `EllipsoidalSurface3D*` | geometry::core::ellipsoidal_*_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `TorusSolid3D*`, `TorusSurface3D*` | geometry::core::torus_*_traits | geo_contracts | ✅ geo_contracts に移設済み |
| `AnalysisTransform2D`, `AnalysisTransform3D` 等 transform | geometry::core::transform | geo_core | ✅ geo_core に移設済み (#319) |
| `TransformError`, `SafeTransform` | geometry::core::transform_error / extensions | geo_core | ✅ geo_core に移設済み (#319) |
| `BasicCollision`, `BasicIntersection` 等 extension | geometry::extensions | geo_contracts::operations | ✅ geo_contracts::operations に移設済み (#332) |
| `Bounded`, `ExtensionFoundation` 等 | geometry::foundation::extension_foundation | → 暫定: geo_contracts | 🔲 Phase 1で判断 |
| `EntityDisplayProperties`, `EntityIdentity`, `LineEntity3DProperties` | entity::core | geo_contracts::entity | ✅ geo_contracts に移設済み |
| `ToleranceSettings`, `GeometryContext` | tolerance | → 暫定: geo_core | 🔲 Phase 4で整理 |
| `tolerance_migration::DefaultTolerances` | tolerance_migration | 撤去（移設先なし） | 🔲 Phase 4で除去 |

詳細:

- 入力:
  - `model/geo_foundation/src/lib.rs` の `pub mod` / `pub use`
  - `model/**` の `use geo_foundation::...` 参照
- 作業:
  - 公開項目を「analysis / geo_contracts / geo_core / 実装側 / 要判断」に分類
  - 参照クレートごとに利用用途（契約 / tolerance / extension / transform）をタグ付け
  - `要判断` 項目には暫定所属と理由を1行で記載
- 完了条件:
  - 分類表に未分類項目がない
  - `要判断` にオーナー候補が全て入っている

確認コマンド:

- `cargo check -p geo_foundation`
- `rg "^pub (mod|use) " model/geo_foundation/src/lib.rs`
- `rg "geo_foundation::" model/`

### Phase 1: geo_contracts の骨格追加

- [x] `classification` を `geo_contracts` に追加
- [x] `geometry::core` 相当のモジュール骨格を `geo_contracts` に追加
- [x] `geo_foundation` 側は一時的に `geo_contracts` から再エクスポートする薄い層へ寄せる
- [x] arc/circle/point/vector/triangle/nurbs/plane/ray/infinite_line 契約を `geo_contracts` に移設済み
- [x] direction/ellipse/ellipse_arc/linesegment/rectangle 契約を `geo_contracts` に追加
- [x] spherical/conical/cylindrical/ellipsoidal/torus の solid+surface 契約 (10種) を `geo_contracts` に追加
- [ ] `Bounded` / `ExtensionFoundation` 系の所属先を確定し `geo_contracts` に追加（または geo_core）
- [ ] `cargo check -p geo_contracts -p geo_foundation`

詳細:

- 入力:
  - `model/geo_foundation/src/classification.rs`
  - `model/geo_foundation/src/geometry/core/*_traits.rs`
- 作業:
  - 未移設の14種の _traits ファイルを `geo_contracts/src/geometry/core/` に追加
  - `geo_foundation` は直接定義を減らし、`pub use geo_contracts::...` の互換再エクスポートへ寄せる
  - `geo_contracts` には計算本体・拡張処理を入れない
- 完了条件:
  - `geo_contracts` 単体で全形状の公開契約が解決できる
  - `geo_foundation` 側で重複定義が増えていない

確認コマンド:

- `cargo check -p geo_contracts`
- `cargo check -p geo_foundation`

### Phase 2: geo_primitives 全面切替（縦切りをすべての形状に拡張）

- [x] arc/circle 系 contract: `geo_contracts` に移設済み、`geo_primitives` 側の一部切替済み
- [ ] direction/ellipse/ellipse_arc/linesegment/rectangle/solid・surface系 import を `geo_contracts` 基準へ全面切替
- [ ] `geo_primitives/src/lib.rs` の `pub use geo_foundation::*` 再エクスポートを `geo_contracts` 経路へ統一
- [ ] `geo_primitives/Cargo.toml` から `geo_foundation` 依存を除去
- [ ] `cargo check -p geo_primitives`
- [ ] `cargo test -p geo_primitives`

詳細:

- 入力:
  - `model/geo_primitives/src/**/*.rs`
  - `model/geo_primitives/src/lib.rs` の re-export
- 作業:
  - 全 `use geo_foundation::...` 参照を `geo_contracts::...` / `analysis::...` / `geo_core::...` 経由へ置換
  - `TransformError` / Transform trait は `geo_core` のまま維持
  - `tolerance_migration::DefaultTolerances` は `analysis` または `geo_core::ToleranceSettings` へ切替
- 完了条件:
  - `geo_primitives` 内に `geo_foundation` 参照ゼロ
  - `cargo check -p geo_primitives` 成功
  - `cargo test -p geo_primitives` 成功

確認コマンド:

- `cargo check -p geo_primitives`
- `cargo test -p geo_primitives`

### Phase 3: NURBS 契約切替

- [ ] `NurbsCurve2D*` / `NurbsCurve3D*` / `NurbsSurface3D*` は既に `geo_contracts` に存在; `geo_nurbs` の import を切替
- [ ] `geo_nurbs/Cargo.toml` から `geo_foundation` 依存を除去
- [ ] `cargo check -p geo_nurbs`
- [ ] `cargo test -p geo_nurbs`

詳細:

- 入力:
  - `model/geo_foundation/src/geometry/core/nurbs_*_traits.rs`
  - `model/geo_nurbs/src/*.rs`
- 作業:
  - NURBS 契約 trait を `geo_contracts` へ移設
  - `geo_nurbs` の constructor/properties/measure 参照を `geo_contracts` へ置換
  - README / doctest の import 経路を更新
- 完了条件:
  - `geo_nurbs` 内の NURBS 契約参照が `geo_contracts` へ統一
  - NURBS 関連テストが通る

確認コマンド:

- `cargo check -p geo_nurbs`
- `cargo test -p geo_nurbs`
- `rg "geo_foundation::(Nurbs|core::nurbs_)" model/geo_nurbs`

### Phase 4: 利用側クレート整理

- [ ] `geo_algorithms` の `Scalar` / `Tolerance*` / 形状契約参照を整理
- [ ] `geo_io` / `geo_entity` / `cam_core` の残存参照を除去

詳細:

- 入力:
  - `model/geo_algorithms/**`
  - `model/geo_io/**`
  - `model/geo_entity/**`
  - `model/cam_core/**`
- 作業:
  - `Scalar` / 定数は `analysis` または `geo_contracts` 由来へ切替
  - 形状契約参照は `geo_contracts` へ切替
  - transform 共通責務参照は `geo_core` へ切替
  - `Cargo.toml` から `geo_foundation` 依存を削除
- 完了条件:
  - 4クレートの `Cargo.toml` に `geo_foundation` 依存がない
  - 旧 import がコメント含めて管理可能な範囲に縮小

確認コマンド:

- `cargo check -p geo_algorithms -p geo_io -p geo_entity -p cam_core`
- `rg "geo_foundation::" model/geo_algorithms model/geo_io model/geo_entity model/cam_core`

### Phase 5: 互換層縮退と削除

- [ ] `geo_foundation` の再エクスポートを極小化
- [ ] workspace 内の旧 import をゼロにする
- [ ] `model/geo_foundation` を削除

詳細:

- 入力:
  - workspace 全クレートの import / dependency
  - `Cargo.toml` workspace members
- 作業:
  - `geo_foundation` を薄い互換層として最小化し、残利用をゼロ化
  - `Cargo.toml` から `model/geo_foundation` を除去
  - `model/geo_foundation` ディレクトリを削除
  - ドキュメントのアーキテクチャ図・依存説明を更新
- 完了条件:
  - workspace 内に `geo_foundation::` 参照が存在しない
  - workspace build/test と依存チェックが通る

確認コマンド:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies_simple.ps1`
- `rg "geo_foundation::|geo_foundation = \{ path = \"../geo_foundation\" \}" model/ Cargo.toml`

## 6. PR分割（実行計画）

`#318` は 3PR で進める。Cross Operation の本格移設は `#332` 側で扱い、
`#318` では依存置換と最終撤去に必要な最小差分に限定する。

### PR-A: contracts骨格 + primitives先行切替（arc/circle）

目的:

- `geo_contracts` を形状契約の参照先として成立させる
- `geo_primitives` の arc/circle 契約参照を先行置換する

完了条件:

- `geo_contracts` に `classification` / `geometry::core` 骨格がある
- `point/vector/arc/circle` 契約が `geo_contracts` に存在する
- `geo_primitives` の arc/circle import 切替が完了する
- `cargo check -p geo_contracts -p geo_foundation -p geo_primitives` が通る
- `cargo test -p geo_primitives arc_` / `circle_` が通る

注意:

- `conical_surface_3d_collision.rs` / `cylindrical_solid_3d_collision.rs` / `cylindrical_surface_3d_collision.rs` の cross collision 実装移設は `#332` で扱う

### PR-B: nurbs + 利用側クレート置換

目的:

- `geo_nurbs` の契約参照を `geo_contracts` へ移行する
- `geo_algorithms` / `geo_io` / `geo_entity` / `cam_core` の `geo_foundation` 参照を用途別に置換する

完了条件:

- `geo_nurbs` の NURBS 契約参照が `geo_contracts` 基準
- 4クレートの `Cargo.toml` から `geo_foundation` 依存を削除
- `cargo check -p geo_nurbs -p geo_algorithms -p geo_io -p geo_entity -p cam_core` が通る

注意:

- `collision/intersection` の本格移設は `#332` へ分離

### PR-C: 互換層縮退 + 最終撤去判定

目的:

- `geo_foundation` の互換層を最小化し、撤去可能判定を行う

完了条件:

- workspace 内の `geo_foundation::` 参照が説明可能な最小残件またはゼロ
- `cargo check --workspace` / `cargo test --workspace` が通る
- 依存チェックスクリプトが通る
- `model/geo_foundation` 削除準備が整う

## 6.1 PR-A 作業項目（更新）

- [x] `geo_contracts` に `classification` モジュール追加
- [x] `geo_contracts` に `geometry/core` 骨格追加
- [x] `point/vector` 契約を `geo_contracts` へ先行移設
- [x] `arc/circle` 契約を `geo_contracts` へ先行移設
- [x] `geo_foundation` の同名公開項目を `geo_contracts` 再エクスポートへ変更
- [x] `geo_primitives` の arc/circle 系 import を `geo_contracts` に先行切替
- [x] `cargo check -p geo_contracts -p geo_foundation -p geo_primitives`
- [x] `cargo test -p geo_primitives arc_`
- [x] `cargo test -p geo_primitives circle_`
- [ ] PR-A の差分をコミットしPR化

## 6.2 PR-B 作業項目（新設）

- [ ] `geo_nurbs` の constructor/properties/measure 参照を `geo_contracts` へ置換
- [ ] `geo_nurbs` README / doctest の旧 import を更新
- [ ] `geo_algorithms` の `Scalar` / `Tolerance*` / 形状契約参照を用途別に置換
- [ ] `geo_io` / `geo_entity` / `cam_core` の `geo_foundation` 参照を置換
- [ ] 4クレートの `Cargo.toml` から `geo_foundation` 依存を削除
- [ ] `cargo check -p geo_nurbs -p geo_algorithms -p geo_io -p geo_entity -p cam_core`

## 6.3 PR-C 作業項目（新設）

- [ ] `geo_foundation` の互換再エクスポートを最小化
- [ ] workspace の `geo_foundation::` 参照をゼロ化または `#332` 対象として明文化
- [ ] `cargo check --workspace`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies_simple.ps1`
- [ ] `Cargo.toml` workspace members から `model/geo_foundation` 削除準備

## 6.4 PRレビュー用 境界チェック

- [ ] `geo_contracts` に計算アルゴリズム本体が入っていない
- [ ] `geo_contracts` に transform 実装本体が入っていない
- [ ] `geo_core` に形状固有語依存の trait 実装が増えていない
- [ ] `geo_primitives` / `geo_nurbs` に共通契約の再定義を追加していない
- [ ] `collision/intersection` の本格移設差分が `#332` 側に分離されている
- [ ] 新規 public API が固定境界表（3.0節）に適合している

## 7. #318 クローズ条件（更新）

- [ ] `geo_contracts` が形状契約の正規参照先として成立している
- [ ] `geo_primitives` / `geo_nurbs` が実装責務中心に整理されている
- [ ] `geo_foundation` 依存 6 クレートの切替が完了している
- [ ] `geo_foundation` なしで workspace build/test が通る
- [ ] 依存チェックスクリプトが通る
- [ ] `#332` との境界（cross operation 移設対象）が明文化されている