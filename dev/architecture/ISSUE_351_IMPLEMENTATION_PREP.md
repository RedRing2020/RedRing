# Issue #351 実装準備メモ

## 目的

`geo_primitives` 側に残っている collision/intersection 実装を段階削減し、呼び出し経路を `geo_algorithms` 正本へ統一するための準備情報を整理する。

## 現在ブランチ

- `issue-351-main-cleanup`

## 棚卸し結果（要点）

- `model/geo_primitives/src` には collision/intersection 系ファイルが多数残存
- `model/geo_primitives/src/lib.rs` には collision/intersection `mod` 宣言が多数残存
- `model` 配下の検索では、実運用の参照は `geo_algorithms` 側が主経路

## 削減対象の一次分類

1. 2D collision/intersection 実装ファイル
- 例: `arc_2d_collision.rs`, `circle_2d_intersection.rs`, `ray_2d_collision.rs` など

2. 3D collision/intersection 実装ファイル
- 例: `arc_3d_collision.rs`, `plane_3d_intersection.rs`, `triangle_mesh_3d_collision.rs` など

3. `geo_primitives/src/lib.rs` の `mod` 宣言
- 上記ファイルに対応する `mod ...collision` / `mod ...intersection`

4. 付随テストモジュール
- `*_collision_tests.rs`, `*_intersection_tests.rs`（公開/非公開を個別確認）

## 推奨実装順

1. 参照実態の固定
- 削除候補ごとに「外部参照なし」を検索で確認

2. `lib.rs` から段階的に切り離し
- まず `mod` 宣言を削減し、コンパイルエラーで残存参照を顕在化

3. 実装ファイル削除
- `lib.rs` から切り離した単位で削除

4. 回帰テスト整備
- `geo_algorithms` 正本経路のテストを追加/更新

5. 検証
- `cargo clippy -- -D warnings`
- `cargo fmt`
- `cargo test --workspace`

## 実行時の注意

- 一括削除ではなく、小さな単位で削減してビルドを都度確認する
- `geo_algorithms` 実装内で `use geo_primitives::...` の直接 import を増やさない
- 既存ラベル/Issue運用に合わせ、作業ログは #351 に都度コメントする

## 参照用コマンド（PowerShell）

```powershell
# collision/intersection候補一覧
Get-ChildItem -Path model/geo_primitives/src -File -Recurse |
  Where-Object { $_.Name -match 'collision|intersection' }

# lib.rs の mod 宣言確認
Select-String -Path model/geo_primitives/src/lib.rs \
  -Pattern 'mod .*collision|mod .*intersection|pub mod .*collision|pub mod .*intersection'

# model 配下の collision/intersection 参照検索
Get-ChildItem -Path model -Filter *.rs -Recurse |
  Select-String -Pattern 'geo_primitives::.*collision|geo_primitives::.*intersection|::collision::|::intersection::'
```

## 実施済みスライス

### Slice 1: dead file 削減（2026-03-22）

- 方針: `lib.rs` で未公開（コメントアウト）かつ参照なしの 2D `EllipseArc` collision/intersection 実装を先に削減
- 削除ファイル:
  - `model/geo_primitives/src/ellipse_arc_2d_collision.rs`
  - `model/geo_primitives/src/ellipse_arc_2d_collision_tests.rs`
  - `model/geo_primitives/src/ellipse_arc_2d_intersection.rs`
  - `model/geo_primitives/src/ellipse_arc_2d_intersection_tests.rs`
- 根拠: ワークスペース検索で上記モジュールは `lib.rs` のコメント行以外から参照されないことを確認
- 検証:
  - `cargo clippy -p geo_primitives -- -D warnings`: pass
  - `cargo fmt --all`: pass
  - `cargo test -p geo_primitives`: pass（316 passed, 0 failed）

### 次スライス候補

1. `lib.rs` で「ファイル未実装」扱いの 3D collision/intersection モジュール群の実参照を再検証
2. dead file が確認できた単位から同様に削減
3. 削減不能なものは #351 で「残置理由」を明文化

### Slice 2: 3D dead file 一括削減（2026-03-22）

- 方針: `lib.rs` の `ファイル未実装` コメント対象で、実参照がないモジュールのみ削除
- 削除ファイル（22件）:
  - `conical_solid_3d_collision.rs`
  - `conical_solid_3d_collision_tests.rs`
  - `conical_solid_3d_intersection.rs`
  - `conical_solid_3d_intersection_tests.rs`
  - `conical_surface_3d_collision.rs`
  - `conical_surface_3d_collision_tests.rs`
  - `conical_surface_3d_intersection.rs`
  - `conical_surface_3d_intersection_tests.rs`
  - `cylindrical_solid_3d_collision_tests.rs`
  - `cylindrical_solid_3d_intersection.rs`
  - `cylindrical_solid_3d_intersection_tests.rs`
  - `cylindrical_surface_3d_intersection_tests.rs`
  - `ellipse_arc_3d_collision.rs`
  - `ellipse_arc_3d_collision_tests.rs`
  - `ellipse_arc_3d_intersection.rs`
  - `ellipse_arc_3d_intersection_tests.rs`
  - `spherical_solid_3d_intersection.rs`
  - `spherical_solid_3d_intersection_tests.rs`
  - `spherical_surface_3d_collision.rs`
  - `spherical_surface_3d_collision_tests.rs`
  - `spherical_surface_3d_intersection.rs`
  - `spherical_surface_3d_intersection_tests.rs`
- 検証:
  - `cargo clippy -p geo_primitives -- -D warnings`: pass
  - `cargo fmt --all`: pass
  - `cargo test -p geo_primitives`: pass（316 passed, 0 failed）

### 次アクション

1. `geo_primitives/src/lib.rs` の stale コメント（`ファイル未実装`）を実体に合わせて整理
2. #351 の主題に沿って、2D/3D の「有効モジュール内の旧実装」削減方針を段階化
3. 回帰テスト追加が必要な経路を #351 本文チェックリストへ反映

### Slice 3: `lib.rs` stale コメント整理（2026-03-22）

- `geo_primitives/src/lib.rs` から、削除済み dead file を指す `pub mod ...` コメント行を整理
- 対象: 2D/3D `EllipseArc`、Conical 系、一部 Cylindrical/Spherical の collision/intersection コメント宣言
- 検証:
  - `cargo clippy -p geo_primitives -- -D warnings`: pass
  - `cargo fmt --all`: pass
  - `cargo test -p geo_primitives`: pass（316 passed, 0 failed）

## 本丸着手準備（post PR #370 merge）

### 前提確認

- PR #370（dead code cleanup）は `develop` へマージ済み
- #351 は open のまま継続（本丸タスク未着手）
- ブランチは `issue-351-main-cleanup` で開始

### 現在有効な collision/intersection モジュール（`lib.rs` の `mod` 宣言ベース）

#### 2D

- `arc_2d_collision`, `arc_2d_intersection`
- `circle_2d_collision`, `circle_2d_intersection`
- `ellipse_2d_collision`, `ellipse_2d_intersection`
- `infinite_line_2d_collision`, `infinite_line_2d_intersection`
- `line_segment_2d_collision`, `line_segment_2d_intersection`
- `ray_2d_collision`, `ray_2d_intersection`
- `triangle_2d_collision`, `triangle_2d_intersection`

#### 3D

- `arc_3d_collision`, `arc_3d_intersection`
- `circle_3d_collision`, `circle_3d_intersection`
- `cylindrical_solid_3d_collision`
- `cylindrical_surface_3d_collision`, `cylindrical_surface_3d_intersection`
- `ellipse_3d_collision`, `ellipse_3d_intersection`
- `ellipsoidal_solid_3d_collision`, `ellipsoidal_solid_3d_intersection`
- `ellipsoidal_surface_3d_collision`, `ellipsoidal_surface_3d_intersection`
- `infinite_line_3d_collision`, `infinite_line_3d_intersection`
- `line_segment_3d_collision`, `line_segment_3d_intersection`
- `plane_3d_collision`, `plane_3d_intersection`
- `ray_3d_collision`, `ray_3d_intersection`
- `spherical_solid_3d_collision`
- `torus_solid_3d_collision`, `torus_solid_3d_intersection`
- `torus_surface_3d_collision`, `torus_surface_3d_intersection`
- `triangle_3d_collision`, `triangle_3d_intersection`
- `triangle_mesh_3d_collision`, `triangle_mesh_3d_intersection`

### 実行順（本丸）

1. 2D first: `arc/circle/line_segment/ray/triangle` から wrapper化・縮退
2. 2D second: `infinite_line/ellipse` を同手順で縮退
3. 3D first: `line_segment/plane/ray/infinite_line/circle/arc` の基礎ペアを縮退
4. 3D second: `ellipse/ellipsoidal/cylindrical/spherical/torus/triangle_mesh` を縮退
5. 段階ごとに `cargo clippy -p geo_primitives -- -D warnings` -> `cargo fmt --all` -> `cargo test -p geo_primitives`

### 直近スライス（次コミット候補）

- 対象: 2D `arc_2d_*`, `circle_2d_*`, `line_segment_2d_*`
- 目標: `geo_algorithms` 正本関数への委譲に寄せるか、削除不能な shape-local 処理だけ残置
- 完了判定:
  - old 実装の重複ロジックが減っている
  - `geo_algorithms` 正本経路で回帰しない
  - #351 に差分サマリを追記済み
