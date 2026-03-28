# geo_algorithms モジュール分割ルール

最終更新: 2026-03-28
適用範囲: `model/geo_algorithms/src/{collision,intersection,distance}`

## 1. 目的

`primitive_2d` / `primitive_3d` / `primitive_nurbs*` の分割基準を統一し、
実装追加時の迷いと配置ゆれを防止する。

## 2. ルールの原則

- 原則1: **演算種別でディレクトリを分ける**
  - `collision/`, `intersection/`, `distance/`
- 原則2: **次元と形状系でファイルを分ける**
  - 2D primitive, 3D primitive, NURBS curve, NURBS surface
- 原則3: **対称性は正規方向 + 逆方向委譲**
  - 正規方向だけ計算本体を持ち、逆方向はラッパー
- 原則4: **公開面は mod.rs で一元再エクスポート**
- 原則5: **新規ペア追加時は coverage matrix を同時更新**

### 2.1 分割軸の定義（違和感対策）

本ルールは、次の2軸で分類する。

- 第1軸: 計算空間の次元（2D / 3D）
- 第2軸: 形状ファミリ（primitive / nurbs_curve / nurbs_surface）

整理すると以下になる。

- `primitive_2d.rs`: 2D × primitive
- `primitive_3d.rs`: 3D × primitive
- `nurbs_curve_3d.rs`: 3D × nurbs_curve
- `nurbs_surface_3d.rs`: 3D × nurbs_surface

ただし現行ファイル名 `primitive_nurbs*` は、
この2軸表現を名前に十分反映できておらず、命名上の一貫性は未達である。

理由:

- `geo_nurbs` 全体では `NurbsCurve2D` / `NurbsCurve3D` / `NurbsSurface3D` を提供する
- 一方 `geo_algorithms` の NURBS × Primitives 集約スコープ（本ルール対象）は現時点で 3D を主対象とする
- curve/surface でアルゴリズム責務（反復・収束特性・サンプリング戦略）が異なる
- `primitive_3d.rs` に混在させると責務過密になる

### 2.2 正規命名（目標）

分割軸に一致した正規命名を以下とする。

- `primitive_2d.rs`
- `primitive_3d.rs`
- `nurbs_curve_2d.rs`（必要時）
- `nurbs_curve_3d.rs`
- `nurbs_surface_3d.rs`

`primitive_nurbs.rs` / `primitive_nurbs_surface.rs` はレガシー互換名として扱う。

## 3. 標準ファイル構成

各演算ディレクトリで、以下を標準とする。

```text
{operation}/
  mod.rs
  pair_base.rs                 # 必要時のみ（共通計算）
  primitive_2d.rs
  primitive_3d.rs
  nurbs_curve_2d.rs            # NurbsCurve2D を扱う場合のみ
  nurbs_curve_3d.rs            # 既存 primitive_nurbs.rs の正規後継名
  nurbs_surface_3d.rs          # 既存 primitive_nurbs_surface.rs の正規後継名
```

補足:
- `distance` は現時点で `pair_base.rs` 不要。
- 既存互換のため、当面は `primitive_nurbs.rs` / `primitive_nurbs_surface.rs` を維持してよい。

## 4. 配置基準（どのファイルに置くか）

- `primitive_2d.rs`: 2D primitive 同士
- `primitive_3d.rs`: 3D primitive 同士
- `nurbs_curve_2d.rs`: NurbsCurve2D を含むペア（存在する場合）
- `nurbs_curve_3d.rs` (または `primitive_nurbs.rs`): NurbsCurve3D を含むペア
- `nurbs_surface_3d.rs` (または `primitive_nurbs_surface.rs`): NurbsSurface3D を含むペア
- `pair_base.rs`: collision/intersection の共通下位ロジック

補足（命名上の見え方について）:

- 既存ファイル名 `primitive_nurbs.rs` / `primitive_nurbs_surface.rs` は歴史的名称。
- 正規化後は `nurbs_curve_3d.rs` / `nurbs_surface_3d.rs` を推奨し、
  「primitive と NURBS で軸が違う」印象を無くす。

## 5. 関数命名規約

- 通常距離: `{shape_a}_{shape_b}_distance`
- 失敗可能距離: `{shape_a}_{shape_b}_try_distance`
- 逆方向: `{shape_b}_{shape_a}_distance` は `{shape_a}_{shape_b}_distance` へ委譲

例:
- `nurbscurve3d_ray3d_distance`
- `nurbscurve3d_ray3d_try_distance`
- `ray3d_nurbscurve3d_distance` (委譲)

## 6. mod.rs の並び順（固定）

`mod.rs` は次順序で宣言・`pub use` する。

1. `pair_base`（ある場合）
2. `primitive_2d`
3. `primitive_3d`
4. `nurbs_curve_2d`（ある場合）
5. `nurbs_curve_3d`（現行は `primitive_nurbs`）
6. `nurbs_surface_3d`（現行は `primitive_nurbs_surface`）

## 7. 段階移行ルール（既存コード保護）

- Phase A (即時): 新規実装は本ルールに従って配置。
- Phase B (中期): `primitive_nurbs*` を `nurbs_curve_3d` / `nurbs_surface_3d` へリネーム。
- Phase C (中期): NurbsCurve2D を演算層で扱う場合は `nurbs_curve_2d.rs` を新設。
- Phase D (中期): `distance` の NURBS 実装完了後、演算間の命名を完全統一。

重要:
- 一括リネームは避ける。Issue 単位で段階的に行う。
- API 互換を壊す変更は PR 単位で明示する。

## 8. PR チェック項目（必須）

- [ ] 追加関数の配置先が本ルールと一致
- [ ] 逆方向関数が委譲実装
- [ ] `mod.rs` の宣言・再エクスポート順が規約準拠
- [ ] `coverage_matrix_engine.rs` へ対称エントリ追加
- [ ] `cargo clippy -p geo_algorithms -- -D warnings` 成功
- [ ] `cargo fmt --all` 実行
- [ ] `cargo test -p geo_algorithms` 成功
- [ ] `./scripts/check_architecture_dependencies_simple.ps1` 成功

## 9. #403 への適用メモ

- #403 Phase2 では、当面 `distance/primitive_3d.rs` に NURBS×3D primitive を集約する。
- ただし将来整理で `distance/nurbs_curve_3d.rs` / `distance/nurbs_surface_3d.rs` へ分離可能。
- その際は本ルールを正として移行する。

## 10. #338 再開時の戻り値移行ルール

`#338` では、`Option<Point3D<T>>` を返す既存関数を一括置換せず、
`IntersectionResult<T>` への段階移行を行う。

### 10.1 基本方針

- 既存の `*_intersection` 関数は互換のため当面維持する。
- 新規に `*_intersection_result` 関数を追加し、呼び出し側を段階的に移行する。
- `IntersectionResult::from_option_point()` / `from_option_points()` を使用して変換規約を統一する。
- 一度に多数ペアを移行せず、1～2ペア単位の小PRで進める。

### 10.2 初期スライス（Phase C 再開の最小単位）

- `intersection/primitive_3d.rs` の point系ペアから開始する。
  - `arc3d_point3d_intersection_result`
  - `circle3d_point3d_intersection_result`

### 10.3 実装順序

1. `*_intersection_result` 追加（旧APIは変更しない）
2. 変換規約テスト追加（Disjoint / Crossing を最低限確認）
3. 呼び出し側を1箇所ずつ移行
4. 十分な移行完了後に旧API整理を検討
