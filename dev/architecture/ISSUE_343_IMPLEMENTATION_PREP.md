# ISSUE #343 設計固定（確定版）

対象Issue: [#343 [Distance] 設計固定: distance契約と実装責務の最終整理](https://github.com/RedRing2020/RedRing/issues/343)

## 背景

distance 系の定義・実装責務が `geometry::core` / `geometry::operations` / 各 shape 実装に分散しており、
cross-shape distance の一次参照先と実装正本の境界が曖昧になっている。

## 現状観測（2026-03-22）

- `geo_contracts::geometry::operations::collision` に `BasicCollision` / `PointDistance` が存在
- 一方で `geo_contracts::geometry::core` にも shapeごとの `distance_to_*` が多数残存
- `geo_algorithms::distance` には 2D/3D の free-function 正本化の土台が存在
- `geo_algorithms::collision` / `intersection` でも `distance_to_*` を直接呼ぶ実装が多数ある

## 設計選択肢

### 案A（推奨）

- cross-shape distance 契約は `geometry::operations` を一次参照先に統一
- `geometry::core` の distance は「自己計量・形状固有計量」のみに限定
- `geo_algorithms` を cross-shape distance 実装正本として段階移行
- 逆方向対称性は委譲実装で統一（A-B 正規、B-A 委譲）

利点:
- #342 の実装移行と整合
- 責務境界が最も明確

懸念:
- 既存 API の互換整理が必要

### 案B（段階互換優先）

- `geometry::core` 側の distance 定義を当面残しつつ、`operations` へ段階的再エクスポート
- 実装移行中は暫定互換レイヤーを設置

利点:
- 破壊的変更を減らせる

懸念:
- 二重参照期間が長引きやすい

## 設計固定の結論

採用方針: 案A

- cross-shape distance 契約は `geometry::operations` を一次参照先に統一する
- `geometry::core` の distance は自己計量・形状固有計量に限定する
- cross-shape distance 実装の正本は `geo_algorithms` に集約する
- 対称性は「代表方向のみ正規実装 + 逆方向は委譲」で統一する
- NURBS を含む収束演算は失敗を明示できる結果型を採用する
- 全組み合わせ即時実装は非目標とし、段階導入を前提とする

## Q1-Q8 確定回答（Issue本文対応）

- Q1: Yes
- Q2: Yes
- Q3: Yes
- Q4: Yes
- Q5: Yes
- Q6: Yes
- Q7: Yes
- Q8: Yes

## #343 で確定した判断軸

1. 契約の一次参照先: `geometry::operations`
2. `geometry::core` の責務: 自己計量・形状固有計量のみ
3. cross-shape distance 実装正本: `geo_algorithms`
4. 対称性ルール: 正規方向実装 + 逆方向委譲
5. 失敗モデル: 収束失敗を結果型で明示
6. 導入戦略: 代表ペアから段階移行（非目標: 全N×N即時実装）

## #344 向け分類ルール（簡易）

- `core` に残す: 同一形状内で閉じる計量（例: point/vector の自己距離、shape 固有の measure）
- `operations` に寄せる: 複数形状を横断する距離契約（cross-shape）

## 互換方針

- 一時互換層は「必要最小限のみ」
- 互換層は #344 の契約移行期間に限定し、#342 で段階削減する

## #344 への受け渡し条件

- Q1-Q8 のYes/Noが確定
- `core` に残すdistance / `operations` へ寄せるdistance の分類表が作成済み
- 互換方針（互換層の有無）が決定済み

## #342 への受け渡し条件

- #344 完了後、代表ペア（line/point, line/line, NURBS代表）の実装対象が確定
- 失敗モデルの型とテスト方針が確定
