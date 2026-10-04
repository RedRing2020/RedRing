# Transform 契約設計

対象 Issue: [#760 Transform 契約の再設計](https://github.com/RedRing2020/RedRing/issues/760)

## 1. 目的

- 他の層から呼び出せる変換の契約を `geo_contracts` に定義する
- 変換の種類（剛体・相似など）を型で表し、形状がどの変換で同種の形状に保たれるかを型で保証する
- 行列生成・合成などの共通処理を `geo_core` に一本化し、形状クレートは形状固有の変換のみを実装する

## 2. 現状と問題

### 2.1 契約がない

- `geo_contracts` に行列による汎用の変換契約がない。形状別の個別操作（`Direction2D/3DTransform`・`InfiniteLine2D/3DTransform`・`Plane3DTransform`・`Ray2D/3DTransform` の `reverse` / `rotate_90` 等）が 7 形状分あるのみ
- 共通 trait `AnalysisTransform3D/2D` が `geo_core` に定義されており、trait 定義の正本を `geo_contracts` に置く方針（[GEO_CORE_GEO_CONTRACTS_TRAIT_SOURCE_OF_TRUTH_DESIGN.md](GEO_CORE_GEO_CONTRACTS_TRAIT_SOURCE_OF_TRUTH_DESIGN.md)）と整合しない
- 形状の変換は `geo_*` 以外から使われていない（利用はテストのみ）

### 2.2 既存実装

| 区分 | ファイル | 状態 |
|---|---|---|
| コンパイル対象 | `geo_core`: `point_2d/3d_transform.rs`, `vector_2d/3d_transform.rs` | 稼働 |
| コンパイル対象 | `geo_primitives`: `ellipse_2d` / `ellipsoidal_solid_3d` / `infinite_line_2d` / `ray_2d` / `rectangle_2d` / `rectangle_3d` / `triangle_2d` / `triangle_mesh_3d` の `*_transform.rs`（8 ファイル） | 稼働 |
| コンパイル対象 | `geo_nurbs`: `curve_2d` / `curve_3d` / `surface_3d` の `*_transform.rs`（3 ファイル） | 稼働 |
| 未コンパイル | `geo_primitives` の `*_transform.rs` 24 ファイル（トーラス・球・円筒・円錐・円弧・円・線分・平面・三角形 等） | `lib.rs` 未宣言。宣言すると 21 ファイルで計 377 件のコンパイルエラー（主因は 2025-11 以降の API 変更: private 化されたフィールド・アクセサ名の変更・型の変更） |

経緯: 2025-11-15 の旧 Transform API（BasicTransform / SafeTransform）削除時に `*_analysis_transform.rs` の `mod` 宣言も `lib.rs` から外れ、同日 `*_transform.rs` へリネームして「メイン実装へ昇格」としたが宣言は再追加されなかった。

### 2.3 実装上の問題

- 行列生成（平行移動・回転・スケール・複合）が各形状ファイル・`geo_core::point_3d_transform`・`analysis` に重複（`translation_matrix` は 14 ファイルで同一、`CompositeTransform3D` は 11 ファイルで定義）
- 平行移動・回転・スケール・均等スケール・複合変換が形状ごとに同じ手順で繰り返し実装されている（形状固有の処理は 1 形状あたり数十行）
- 非一様スケールを黙って近似している（トーラスは x・z のスケール比を平均して半径に適用）
- `transform_point_matrix` が `Result` を返さず、内部の `.expect` で panic しうる
- 負スケール（鏡像）による向きの反転を扱っていない
- `geo_core::SafeTransform` は引数が不自然（`safe_translate(offset: T)` 等）で未使用

## 3. 設計方針

### 3.1 生の行列ではなく、変換の種類を型で表す

生の行列を受け取る API を契約の中心にしない。

- 行列には性質の保証がなく（特異・射影・非一様を含みうる）、形状側がその都度判定して実行時エラーにするしかない
- 「中心点まわりの回転」の平行移動の挟み込みや乗算順序の誤りを、呼び出し側がそれぞれ犯しうる
- 呼び出し側（アプリケーションの移動・回転コマンド、STEP の配置など）は行列ではなく操作や配置で考える

変換の種類を型とし、呼び出し側は操作を組み合わせて変換を構築する。型の内部で 1 つの行列に合成し、形状には 1 回で適用する。

```rust
let t = SimilarityTransform3D::rotation_about_axis(center, axis, angle)?
    .then(&SimilarityTransform3D::translation(v))
    .then(&SimilarityTransform3D::uniform_scale_about(center, s)?);

let moved = circle.transform_similarity(&t)?;
```

行列はクレート境界での変換にのみ使う（STEP の配置・GPU 転送: `from_matrix` で検証・判定して取り込み、`to_matrix` で出力）。

### 3.2 対応する変換の範囲

初版は **相似変換**（回転・平行移動・正の一様スケール）のみとする。

| 変換 | 初版 | 扱い |
|---|---|---|
| 剛体（回転・平行移動） | 対応 | 相似変換の特別な場合として扱う |
| 正の一様スケール | 対応 | |
| 非一様スケール（一般アフィン） | 未対応 | `from_matrix` で `TransformError::Unsupported`。[#763](https://github.com/RedRing2020/RedRing/issues/763) で設計 |
| ミラーリング（負スケール） | 未対応 | 同上 |
| 射影 | 未対応 | `from_matrix` で `TransformError::Unsupported` |

非一様スケール・ミラーリングは変換型で表現できないため、形状実装がこれらを受け取ることはない。

剛体変換専用の型（スケールを受け付けない形状のため）は、必要とする形状が現れた時点で追加する。初版の対象形状はすべて相似変換で同種の形状に保たれる。

## 4. クレート間の責務

| クレート | 責務 | 形状特性の参照 |
|---|---|---|
| `geo_contracts` | 変換型の trait（点・ベクトルへの適用、スケール係数、回転の有無）、形状の変換 trait（変換型 trait に対するジェネリック）、`TransformError` | なし |
| `geo_core` | 変換の具象型（構築・合成・行列からの構築と判定・再正規化）、Point / Vector / Direction / AABB への変換実装 | なし（自クレートの型のみ） |
| `geo_primitives` / `geo_nurbs` | 各形状の変換実装 | 自クレート内で実装 |

- `geo_core` は `geo_primitives` に依存しない。`geo_core` が扱うのは変換型と自クレートの Point / Vector / Direction / AABB のみで、形状特性を参照しない
- 形状の変換は形状クレート内で、契約 trait の「点への適用」「ベクトルへの適用」を使って原点・軸を変換し、形状固有の属性（半径等）を再計算して組み立てる
- 変換の呼び出しは `geo_contracts` の trait 経由とする。変換値の生成は `geo_core` の具象型を使う（`Point3D` と同じく `geo_primitives` / `geo_algorithms` から再エクスポートする）
- Rust の孤児ルール上、`geo_core` は `geo_contracts` の型に実装を追加できないため、変換型の具象型は `geo_core` に置き、`geo_contracts` には trait のみを置く（既存の Point / Vector と同じ分担）

`Direction2D` / `Direction3D` は Point / Vector と同じ基本型として `geo_core` に置く（#760 PR0 で `geo_primitives` から移動）。変換型の構築 API は回転軸を `Direction3D` で受け取り、軸が単位ベクトルであることを型で保証する。

## 5. 契約（`geo_contracts`）

点・ベクトルは既存の契約と同じくタプルで受け渡す。

```rust
/// 相似変換（回転・平行移動・正の一様スケール）
pub trait SimilarityTransform3DCore<T: Scalar> {
    /// 点に適用する
    fn apply_point(&self, point: (T, T, T)) -> (T, T, T);
    /// ベクトルに適用する（平行移動を含まない）
    fn apply_vector(&self, vector: (T, T, T)) -> (T, T, T);
    /// 一様スケール係数（> 0）
    fn scale_factor(&self) -> T;
    /// 回転成分を含むか
    fn has_rotation(&self) -> bool;
}

/// 相似変換を受け付ける形状
pub trait SimilarityTransformable3D<T: Scalar>: Sized {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError>;
}
```

2D は `SimilarityTransform2DCore` / `SimilarityTransformable2D` を同じ形で定義する（`apply_point` / `apply_vector` は 2 要素タプル）。

### 5.1 `TransformError`

`geo_core` から `geo_contracts` へ移す。

| バリアント | 用途 |
|---|---|
| `Unsupported(String)` | 初版で扱わない変換（非一様スケール・ミラーリング・射影）、形状が受け付けない変換（AABB の回転） |
| `InvalidGeometry(String)` | 変換結果が形状として成立しない（退化した軸 等） |
| `InvalidParameter(String)` | 構築時の不正な引数（0 以下のスケール、特異な行列 等） |

既存の `ZeroVector` / `InvalidScaleFactor` / `InvalidRotation` は旧変換 API（`AnalysisTransform*`）の実装が使用しているため、旧 API を削除する PR まで残す。新しい変換契約では使用せず、旧 API の削除時にあわせて削除する。`geo_core::TransformError` は `geo_contracts` からの再エクスポートとし、既存の参照パスは維持する。

## 6. 具象型（`geo_core`）

### 6.1 内部表現

`SimilarityTransform3D` は合成済みの 4×4 行列 1 つと、スケール係数・回転の有無を保持する。

- 合成（`then`）は行列の積で 1 つに畳み込み、スケール係数は積で更新する
- 回転の有無（`has_rotation`）はフラグで持たず、合成後の行列の線形部分をスケール係数で割った値と単位行列との差から判定する。打ち消し合う回転を合成した場合は回転なしとして扱える
- 相似 ∘ 相似 は常に相似であり、合成結果の再判定は不要
- 行列演算は `analysis::Matrix4x4`（`translation` / `rotation_axis` / `scale` / `transform_point_3d` / `transform_vector_3d`）を使う。行列生成のヘルパーは `geo_core` に一本化し、形状ファイルに持たない

### 6.2 構築 API

| API | 内容 |
|---|---|
| `identity()` | 恒等変換 |
| `translation(v)` | 平行移動 |
| `rotation_about_axis(center, axis, angle)` | 中心点・軸（`Direction3D`）まわりの回転 |
| `uniform_scale_about(center, s)` | 中心点まわりの一様スケール（`s` が 0 以下（`default_kernel_numerical_zero_tolerance` 以下）は `InvalidParameter`。負スケールはミラーリングのため初版では扱わない） |
| `then(&next)` | `self` の後に `next` を適用する合成 |
| `from_matrix(m)` | 行列から構築。射影・非一様スケール・ミラーリングは `Unsupported`、特異は `InvalidParameter` |
| `to_matrix()` | 行列として出力 |

2D（`SimilarityTransform2D`）は同じ構成で、回転は `rotation_about(center, angle)`（平面内の回転のため軸を取らない）、行列は `Matrix3x3` とする。

### 6.3 再正規化（オプション）

回転を多数回合成すると、浮動小数点誤差により線形部分が「スケール × 回転」から外れていく。合成時の自動再正規化は行わず、呼び出し側が明示的に選べるようにする。

| API | 内容 |
|---|---|
| `orthogonality_error()` | 線形部分をスケール係数で割った行列の、直交行列からのずれ（`‖RᵀR − I‖` の最大要素）を返す |
| `renormalized()` | 線形部分をスケール係数で割り、直交化（3D は Gram–Schmidt、2D は 1 列目の直交方向を 2 列目とする）してスケールを掛け直した変換を返す |

- 既定の合成（`then`）は再正規化しない。数回の合成では追加コストを発生させない
- 判定の許容誤差は `analysis` の許容誤差 API（`geo_contracts::default_orthogonality_dot_error_tolerance` 等）を使い、局所定数を定義しない
- `from_matrix` で取り込んだ外部行列も許容誤差内で判定済みであり、必要に応じて `renormalized()` を適用できる

## 7. 形状ごとの扱い（初版）

| 形状 | 相似変換 | 備考 |
|---|---|---|
| Point / Vector / Direction | 対応 | `geo_core`。Direction は変換後に正規化 |
| AABB（2D/3D） | 平行移動・一様スケールのみ | 回転を含む場合は `Unsupported`。形状の境界ボックスとして使う場合は形状側で再構築する |
| 線分・半直線・無限直線・平面・三角形・三角形メッシュ・矩形 | 対応 | `Ray3D` は方向を `Direction3D` で保持し、`to_line` の再正規化を不要にする |
| 円・円弧・楕円・楕円弧 | 対応 | 半径（長軸・短軸）にスケール係数を掛ける |
| 球・円筒・円錐・楕円体・トーラス（面・立体） | 対応 | 同上。円錐の半頂角は不変 |
| NURBS 曲線・曲面 | 対応 | 制御点を変換。重みは不変 |

### 7.1 法線

- 相似変換では法線はベクトルとして変換して正規化すれば正しい向きになる（一様スケールのため逆転置行列は不要）
- 解析曲面は変換後のフレーム（軸・参照方向）から法線を再計算する
- 非一様スケールでの法線（逆転置行列）と、エンティティ・トポロジー（面の向き・`same_sense`）との整合は [#763](https://github.com/RedRing2020/RedRing/issues/763) で扱う

## 8. 既存 API の扱い

| 対象 | 扱い |
|---|---|
| `geo_core::AnalysisTransform3D/2D` / `AnalysisTransformSupport` | 新契約への移行完了後に削除 |
| `geo_core::SafeTransform` | 削除（未使用） |
| `geo_core` の `point_*_transform.rs` / `vector_*_transform.rs` の行列生成ヘルパー | 変換型の構築 API に統合 |
| `geo_contracts` の形状別個別操作（`reverse` / `rotate_90` 等の `*Transform` trait 7 種） | 行列変換ではないため対象外（現状維持） |
| コンパイル対象の形状変換 11 ファイル（`geo_primitives` 8 / `geo_nurbs` 3） | 新契約の実装へ置き換え |
| 未コンパイルの形状変換 24 ファイル | そのまま復活させず、新契約で書き直す。形状固有の変換処理とテストケースのみ再利用する |
| 未コンパイルの変換以外の 11 ファイル（テスト・extensions） | 本設計と独立のため、別 PR で再利用 / 書き直し / 削除を判定する |

## 9. 実施計画（PR 系列）

| PR | 内容 |
|---|---|
| 0 | `Direction2D` / `Direction3D` を `geo_primitives` から `geo_core` へ移動 |
| 1 | 本設計書。`geo_contracts` の契約と `TransformError`、`geo_core` の具象型（構築・合成・行列からの構築・再正規化）と Point / Vector / Direction / AABB の実装 |
| 2 | 線形・平面系（線分・半直線・無限直線・平面・三角形・三角形メッシュ・矩形） |
| 3 | 円・円弧・楕円・楕円弧 |
| 4 | 球・円筒・円錐・楕円体・トーラス |
| 5 | NURBS |
| 6 | 旧 API（`AnalysisTransform*` / `SafeTransform`）と未コンパイルの形状変換 24 ファイルの削除 |

各 PR で、変換後に形状の種類・半径（スケール係数倍）・フレームの直交性が保たれること、非対応の変換が `Unsupported` になることをテストする。

## 10. スコープ外

- 非一様スケール・ミラーリング・射影（[#763](https://github.com/RedRing2020/RedRing/issues/763)）
- エンティティ・トポロジーへの変換の適用
- `viewmodel` の表示・カメラの座標変換

## 11. 関連

- [#760](https://github.com/RedRing2020/RedRing/issues/760) Transform 契約の再設計
- [#763](https://github.com/RedRing2020/RedRing/issues/763) 非一様スケール・ミラーリング
- [#319](https://github.com/RedRing2020/RedRing/issues/319) Transform 2 層化（[issue-319-transform-layering-plan.md](issue-319-transform-layering-plan.md)）
- [#533](https://github.com/RedRing2020/RedRing/issues/533) trait 定義の正本
- [#758](https://github.com/RedRing2020/RedRing/issues/758) 解析曲面のパラメータ規約（変換で規約が保たれる必要がある）
