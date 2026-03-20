## 概要

collision/intersection の責務モデルを Foundation パターン（Core / Extension / Transform）から切り分けて再定義する。

## 背景

- 現行の `geo_contracts` trait と `geo_primitives` 形状型の組み合わせでは、`geo_algorithms` への trait実装の物理移管は Rust の orphan rules により成立しない
- 依存方向も `geo_algorithms -> geo_primitives` であり、`geo_primitives -> geo_algorithms` を入れずに trait実装の正本位置を動かすことはできない
- 一方で、pair-base / free-function ベースの形状ペアロジックを `geo_algorithms` 側の正本とみなす方針自体は妥当である
- したがって問題は「pair-base 方針」ではなく、「collision/intersection を Foundation パターンの延長として扱う前提」にある

## 問題設定

- Core / Extension / Transform は単一形状責務の整理としては機能している
- collision/intersection は複数形状間アルゴリズムであり、単一形状中心の Foundation パターンにそのまま載せると責務境界が歪む
- その結果、以下が衝突する
  - trait定義の所在
  - 実装の所在
  - crate依存方向
  - orphan rules

## 目的

- collision/intersection を Foundation パターンの対象外または別系統責務として明文化する
- `geo_contracts` / `geo_algorithms` / `geo_primitives` の責務境界を再定義する
- #347 / #350 / #348 / #349 / #351 を破綻なく進められる実施基準を作る

## 検討ポイント

- `geo_contracts` に置くべきものは何か
- `geo_algorithms` を pair-base / free-function 正本とする場合の公開APIをどう定義するか
- `geo_primitives` 側trait実装を互換ラッパーとして残すのか、別の呼び出しモデルへ移行するのか
- 削除完了条件を「trait実装の移管」ではなく「ロジック正本の移行」に読み替えるべきか

## 非対象

- 個別の 2D / 3D / NURBS 実装差分
- 即時の crate 分割や依存逆転導入
- 既存 pair-base ロジックの撤回

## 成果物

- collision/intersection の責務モデル案
- #347 系列に適用する受け入れ条件の読み替え案
- #350 / #348 / #349 / #351 の進め方ガイド

## 関連

- #347 Phase C: collision/intersection trait実装をgeo_algorithmsへ移管
- #350 2D collision/intersection 移管
- #348 3D collision/intersection 移管
- #349 NURBS/Primitive 混在整理
- #351 旧実装削除と回帰テスト整備