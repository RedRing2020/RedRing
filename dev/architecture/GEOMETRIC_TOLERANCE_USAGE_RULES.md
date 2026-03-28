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

## 呼び出し境界ルール

1. API入力トレランスは呼び出し元が `ToleranceSettings` を選択して渡す。
2. 下位処理は受け取った `tolerance` をそのまま伝播させる。
3. `T::EPSILON` は数値安定化の局所用途に限定し、ドメイン判定の既定値にしない。
4. 新規実装で互換層や別正本となるトレランス定義を追加しない。

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

## 関連Issue

- #455
- #361
- #377
- #318
- #320
- #360
