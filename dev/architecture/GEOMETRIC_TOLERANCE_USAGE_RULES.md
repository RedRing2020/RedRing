# Geometric Tolerance Usage Rules

最終更新: 2026-03-29

## 目的

幾何演算におけるトレランスの責務を明確化し、正本と互換層の混在を段階的に解消するための運用ルールを定義する。

## 設計原則

- 正本は `geo_contracts::ToleranceSettings` とする。
- API入力トレランスは呼び出し元が `ToleranceSettings` から取得し、明示的に渡す。
- スケーリング責務は呼び出し元に置く。
- 互換層は暫定運用とし、新規実装での依存追加を禁止する。

## 現状の責務構造

### 正本（維持）

- `model/geo_contracts/src/tolerance.rs`
- 提供: `ToleranceSettings<T>`
- 役割: 幾何演算で使用する標準トレランスの単一の情報源

### 派生利用（維持）

- `model/geo_algorithms/src/octree/tolerance.rs`
- 提供: `OctreeTolerance<T>`
- 役割: `ToleranceSettings::relaxed().distance_tolerance` から Octree 用閾値を派生
- 方針: 幾何判定の別正本を作らず、用途特化の派生のみ許可

### 廃止済み互換層（履歴）

- `model/geo_contracts/src/tolerance_migration.rs`
- 提供: `DefaultTolerances`, `ScalarToleranceExt`
- 状態: 廃止済み

- `model/geo_algorithms/src/tolerance.rs`
- 提供: `ToleranceContext`
- 状態: 廃止済み（Issue #377）

## 判定種別ごとの選択ルール

| 判定種別 | 既定参照元 | 運用ルール |
| --- | --- | --- |
| 距離しきい値（包含、近接、一致） | `ToleranceSettings::distance_tolerance` | 呼び出し境界で受け渡した値を優先する |
| 角度しきい値（平行、垂直、角度比較） | `ToleranceSettings::angle_tolerance` | API呼び出し側でプロファイルを選択して渡す |
| 外積誤差（平行判定補助） | `default_parallel_cross_error_tolerance<T>()` | 型依存閾値を使用し、関数内マジックナンバーを追加しない |
| 内積誤差（直交判定補助） | `default_orthogonality_dot_error_tolerance<T>()` | 型依存閾値を使用し、用途を直交判定に限定する |
| 数値解法の収束補助 | `foundation/analysis/src/consts.rs` | 幾何意味判定の正本としては使わない |

## `default_*` と `Scalar` 関連定数の境界

Issue #548 では、`default_parallel_cross_error_tolerance<T>()` /
`default_orthogonality_dot_error_tolerance<T>()` と、`Scalar` の関連定数
`T::PARALLEL_CROSS_ERROR_TOLERANCE` /
`T::ORTHOGONALITY_DOT_ERROR_TOLERANCE` の責務境界を次で固定する。

- 数値定数の保持層は `foundation/analysis` とする
- geo 系の公開参照面は `geo_contracts` の `default_*` を正本とする
- `Scalar` 関連定数は公開方針の正本ではなく、`default_*` を支える橋渡しと低レベル kernel 向けの内部表現とみなす
- したがって、geo 実装層で「どの値を既定とするか」を決めるときは `default_*` を使い、`T::*` 直接参照を公開面の判断点にしない

この整理により、`analysis` は純粋な数値定数の保持に閉じ、`geo_contracts` は geo 系 API の公開入口として振る舞う。

### レイヤー別の直接参照ルール

| レイヤー | `default_*` | `T::*` 直接参照 | 位置づけ |
| --- | --- | --- | --- |
| `foundation/analysis` | 不要 | 定義元として保持 | 数値定数の正本 |
| `geo_contracts` | 公開する | ラッパー実装でのみ許可 | 公開境界 |
| `geo_core` / `geo_commons` | 任意 | 低レベル kernel に限定して許可 | 数値演算の内部実装 |
| `geo_primitives` / `geo_nurbs` | 原則こちらを使う | 公開 operations では禁止 | shape 実装入口 |
| `geo_algorithms` | 明示入力か `default_*` | 原則禁止 | 高レベル API |
| tests | テスト対象の公開面に合わせる | 低レベル kernel テストに限定 | 振る舞い検証 |

補足:

- `geo_core::Vector2D` / `Vector3D` のような基礎ベクトル演算は、低レベル kernel として `T::*` を内部で読んでよい
- `geo_primitives` の relation / intersection / validation 系 API は、公開面の既定値選択として `default_*` を使う
- 単に値が同じであることを理由に `geo_primitives` で `T::*` を読み続ける運用は採らない

### #548 時点の移行単位

`#548` 自体は設計固定 Issue とし、実装置換は後続 Phase に分ける。分割単位は次を基本とする。

1. 文書正規化: `default_*` と `T::*` の責務境界を固定する
2. `geo_primitives` / `geo_nurbs` の公開 operations から `T::*` 直接参照を除去する
3. `geo_algorithms` の relation / intersection / validation 経路を同ルールへ揃える
4. `geo_core` / `geo_commons` に残す `T::*` 直接参照が低レベル kernel に閉じているかを再点検する

この時点では `Scalar` の関連定数を即座に非公開化しない。まず利用境界を固定し、`default_*` への移行が収束した後に、非公開化の可否を再評価する。

## `geo_algorithms` の tolerance 入口ルール

Issue #611 では、`geo_algorithms` の tolerance 入口を「明示入力中心」で固定する。

前提認識は次の通り。

- `geo_algorithms` の collision / intersection / distance 系 API は、すでに free function + 明示 `tolerance` 引数を中心に構成されている
- `geo_algorithms` は `geo_primitives` と違い、`T::ORTHOGONALITY_DOT_ERROR_TOLERANCE` / `T::PARALLEL_CROSS_ERROR_TOLERANCE` の直接参照を主問題として持っていない
- `ToleranceSettings` の利用は主に tests や `OctreeTolerance` のような用途特化派生に留まっている

したがって、`geo_algorithms` で先に固定すべきなのは「どの値を使うか」より「どこで値を選ぶか」である。

### 基本方針

- `geo_algorithms` の正本 API は明示 `tolerance` 引数を受け取る形を維持する
- `ToleranceSettings` からの標準値選択は呼び出し境界で行う
- `geo_algorithms` 本体に暗黙既定値をばらまかない
- 用途特化の派生設定は許可するが、別正本を作らない

### モジュール別の運用ルール

| モジュール | 入口ルール | 備考 |
| --- | --- | --- |
| `collision` | 明示 `tolerance` 入力を正本とする | 形状ペア free function は既存方針を維持 |
| `intersection` | 明示 `tolerance` 入力を正本とする | `IntersectionResult` へ渡す `tolerance_used` も入力値を使う |
| `distance` | 既定値を持ち込まない | 距離計算自体は純関数として扱い、必要なら呼び出し側で比較閾値を選ぶ |
| `octree` | `ToleranceSettings` からの用途特化派生を許可する | `OctreeTolerance` は用途特化 wrapper として維持 |
| tests/examples | `ToleranceSettings` を利用してよい | 公開 API の正本ではなく呼び出し例として扱う |

### convenience 入口の扱い

現段階では、`geo_algorithms` 本体へ `ToleranceSettings` ベースの convenience 入口を追加しない。

理由は次の通り。

- 既存 API は明示入力で一貫しており、責務境界が崩れていない
- convenience 入口を早期導入すると、呼び出し境界と `geo_algorithms` 本体の責務が再び混ざりやすい
- 必要性が実利用で確認された後でも、本体 API を変えず薄い wrapper として追加できる

したがって、将来 convenience 入口が必要になった場合でも、次を条件とする。

- 本体 API は明示入力のまま維持する
- wrapper は `ToleranceSettings` から必要値を取り出して委譲するだけに留める
- collision / intersection / distance のすべてへ一律導入せず、利用頻度の高い入口から限定導入を検討する

### 実装への含意

- `geo_algorithms` で新規 API を追加する場合、まず明示 `tolerance` 引数版を定義する
- `ToleranceSettings` を直接受ける API を追加する場合は、正本 API ではなく wrapper か用途特化設定であることを明示する
- tests や examples では `ToleranceSettings::<T>::standard()` / `relaxed()` を使ってよいが、その運用を本体 API の既定値へ逆流させない

## 呼び出し境界ルール

1. API入力トレランスは呼び出し元が `ToleranceSettings` を選択して渡す。
2. 下位処理は受け取った `tolerance` をそのまま伝播させる。
3. `T::EPSILON` は数値安定化の局所用途に限定し、ドメイン判定の既定値にしない。
4. 新規実装で互換層や別正本となるトレランス定義を追加しない。

## 数値安定化しきい値ルール

1. カーネル根幹のゼロ判定（ゼロベクトル長、分母ゼロ近傍、特異行列回避）は、アプリケーション設定値ではなく固定しきい値を使用する。
2. 固定しきい値は `foundation/analysis/src/consts.rs` に用途別の意味付き定数として定義し、暗黙の `T::EPSILON` 直書きを避ける。
3. `ToleranceSettings::distance_tolerance` は幾何意味判定（包含、近接、一致）に限定し、数値安定化ガードの既定値に流用しない。
4. 無次元判定（内積・外積誤差）には無次元しきい値を使用し、単位付き距離トレランスを混在させない。

## 廃止ロードマップ

### フェーズ1（Issue #361 / #377）

- 本ドキュメントを整備。
- 旧互換層を廃止。

### フェーズ2（Issue #455）

- 単一正本の運用ルールと判定種別ルールを確定。
- 影響範囲の棚卸しと分割Issue化を完了。

### フェーズ3（分割Issueで実施）

- `geo_algorithms` の高頻度経路移行。
- `geo_primitives` / `geo_nurbs` の残存参照を移行。
- 残存参照排除と回帰テストで収束。

## #455 分割実行計画

1. ルール固定（文書正規化）
2. `geo_algorithms` 高頻度経路の移行
3. `geo_primitives` / `geo_nurbs` の残存参照移行
4. 収束（残存参照の排除、回帰確認）

## 運用ルール

- 新規コードでは `ToleranceSettings` を使用し、互換層APIを増やさない。
- `analysis::consts` は数値計算のための定数として扱い、幾何判定の正本にはしない。
- 既存コード移行時は、呼び出し境界でトレランス取得元を統一する。
- テスト値は `analysis::test_constants` を既定参照元とし、通常ケースで生の数値リテラルを直書きしない。
- 境界ケースや近接ケースで個別調整が必要なときは、テスト内に意味付きローカル定数を定義して使用する。
- 生の数値リテラルをテストで使用する場合は、共有定数や意味付きローカル定数へ置換できない理由をコメントで明記する。

## 関連Issue

- #455
- #548
- #547
- #611
- #541
- #361
- #377
- #318
- #320
- #360
