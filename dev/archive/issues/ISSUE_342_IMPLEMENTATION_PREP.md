# ISSUE #342 実装準備（distance 実装移行: Phase 1）

対象Issue: [#342 [Distance] 実装移行: geo_algorithms へのdistance実装集約（対称性・収束演算対応）](https://github.com/RedRing2020/RedRing/issues/342)

前提:

- #343 クローズ済み（設計固定完了）
- #344 クローズ済み（契約移行完了）

## 今回の実装スコープ（Option 1）

- 代表ペアの最小導入に限定する
  - line/line: `InfiniteLine3D` 同士
  - NURBS代表: `NurbsCurve3D` - `Point3D`
- 対称性は正規方向 + 逆方向委譲で統一する
- NURBS代表ケースに fallible API を追加し、収束失敗モデルを明示する

## 実装方針

1. distance entrypoint の追加（geo_algorithms）
- `infinite_line3d_infinite_line3d_distance`
- `nurbscurve3d_point3d_distance`
- `point3d_nurbscurve3d_distance`（委譲）
- `nurbscurve3d_point3d_try_distance`（fallible）

2. 失敗モデルの統一
- `DistanceConvergenceError` を返す fallible API を導入
- 初期導入では以下をエラー条件とする
  - `max_samples == 0`: `InvalidInitialization`
  - `max_samples == 1`: `NotConverged`

3. テスト更新
- line/line 代表ケースの距離検証
- NURBS fallible API の成功・失敗検証
- coverage matrix の distance seed に代表ペアを追加

## 着手前チェック（#342 I1-I6）

- [x] I1: #343（設計固定）の合意チェックが完了している
- [x] I2: #344（契約移行）が完了し、distance 契約の参照先が一意になっている
- [x] I3: 対称性ルール（正規方向と委譲方向）が実装ガイドとして文書化済み
- [x] I4: NURBS 収束失敗時の結果型とエラーハンドリング方針が確定済み
- [x] I5: 初期導入する代表ペア（line/point, line/line, NURBS代表ケース）が定義済み
- [x] I6: 依存境界チェック項目（アーキテクチャチェック）に抵触しないことを事前確認済み

## 完了判定（Phase 1）

- `geo_algorithms::distance` に代表ペア entrypoint が追加されている
- line/line と NURBS代表ケースのテストが追加されている
- `DistanceConvergenceError` を用いた fallible 経路のテストがある
