# geo_algorithms モジュール分割ルール

最終更新: 2026-10-17
適用範囲: `model/geo_algorithms/src/{operations,intersection}`

## 1. 目的

形状をまたぐ演算（衝突判定・交差・距離）の trait の実装と計算の本体の配置を統一し、組み合わせの追加時の迷いと配置のゆれを防ぐ。

## 2. ルールの原則

- 原則1: **公開面は `operations` の trait に限る**
  - `CrossCollision` / `CrossIntersection` / `CrossDistance` を `geo_algorithms::operations` に置き、`geo_algorithms` から再エクスポートする
  - 形状の組み合わせごとの公開関数は置かない
- 原則2: **代表の向きを実装し、逆の向きはそれを呼ぶ**
  - 対称な組み合わせの逆の向きは、代表の向きの trait の実装を呼ぶだけにする
- 原則3: **衝突判定は交差の有無で判定する**
  - 交差を実装した組み合わせの衝突判定は、交差の trait の実装を呼び、交点の有無で判定する（端点の距離などの簡易判定で代替しない）
- 原則4: **組み合わせの追加時は網羅表を同時に更新する**
  - `CROSS_OPERATION_COVERAGE_MATRIX_SPEC.md` の表と `tests/cross_operation_matrix.rs` を更新する
- 原則5: **点は値型として扱う**
  - 計算の本体の内部でも、座標をタプルに分解して組み直さず、`Point2D<T>` / `Point3D<T>` / `Vector*D<T>` の演算で書く

## 3. 標準ファイル構成

```text
operations.rs                  # trait定義
operations/
  impls_2d.rs                  # 2D の形状の組み合わせの trait の実装
  impls_3d.rs                  # 3D の形状の組み合わせの trait の実装
intersection.rs                # 交差の計算の本体（クレート内部）
intersection/
  common.rs                    # 3D の直線・半直線・線分の交点の基礎計算
  pair_base.rs                 # 2D の線分と円・円弧の交点
  primitive_2d.rs              # 2D の交差
  primitive_3d.rs              # 3D の交差（形状の系統ごとに分割）
  primitive_3d/
    linear_family.rs
    planar_and_mesh_family.rs
    cylindrical_and_conical_family.rs
    spherical_and_quadric_family.rs
    shared.rs                  # 系統をまたぐ補助の計算
```

補足:
- `intersection` はクレート内部のモジュール（`mod intersection;`）とし、外へ公開しない
- trait の実装が短い計算（距離・衝突判定の数行の判定）は、`operations/impls_*.rs` に直接書く
- NURBS を含む組み合わせを実装するときは、`operations/impls_nurbs_curve_3d.rs` / `impls_nurbs_surface_3d.rs` のように形状の系統でファイルを分ける

## 4. 配置基準

- `operations/impls_2d.rs`: 2D の形状同士
- `operations/impls_3d.rs`: 3D の形状同士（`Aabb3D` 等の `geo_core` の型を含む）
- `intersection/primitive_3d/*`: 形状の系統（線形・平面と三角形・円柱と円錐・球と 2 次曲面）でまとめる。1 形状 1 ファイルを既定にしない
- 系統をまたぐ補助の計算は `shared.rs` に隔離し、各系統のファイルに混在させない

## 5. PR チェック項目（必須）

- [ ] 追加した組み合わせの trait の実装が本ルールの配置と一致
- [ ] 対称な組み合わせの逆の向きが代表の向きを呼ぶ実装
- [ ] `CROSS_OPERATION_COVERAGE_MATRIX_SPEC.md` の表と `tests/cross_operation_matrix.rs` を更新
- [ ] `cargo clippy -p geo_algorithms -- -D warnings` 成功
- [ ] `cargo fmt --all` 実行
- [ ] `cargo test -p geo_algorithms` 成功
- [ ] `./scripts/check_architecture_dependencies.ps1` 成功

## 6. 改訂の履歴

- 2026-10-17（#855）: 演算ごとのディレクトリ（`collision/` / `distance/` / `intersection/`）の公開関数を、`operations` の trait の実装に置き換えた。`collision/` と `distance/` は削除し、`intersection/` はクレート内部の計算の本体とした。NURBS の演算のファイル（`*/nurbs_3d.rs`）は、結果が誤った簡易実装だったため削除した
