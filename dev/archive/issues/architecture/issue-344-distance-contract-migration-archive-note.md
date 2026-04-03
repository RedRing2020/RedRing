# ISSUE #344 実装準備（契約移行）

対象Issue: [#344 [Distance] 契約移行: geometry::core から geometry::operations へのdistance契約整理](https://github.com/RedRing2020/RedRing/issues/344)

前提: #343 はクローズ済み（設計固定完了）

## 目的

- distance 契約の一次参照先を `geometry::operations` に統一する
- `geometry::core` 側の cross-shape distance を段階縮退する
- `geo_contracts` の公開API再エクスポートを一意化する

## 現状分類ルール

- `core` に残す:
  - 自己計量（同一型間）
  - 形状固有計量（同一形状のmeasureで閉じるもの）
  - 形状から point への距離（shape-local kernel として保持）
- `operations` に寄せる:
  - cross-shape distance（例: line-line, segment-segment, ray-ray, circle-circle など）
  - A-B / B-A の対称性が必要な距離契約

## 棚卸（初版）

### core に残す候補

- `Point2DMeasure::distance_to` / `Point3DMeasure::distance_to`
- `Vector2DMeasure::distance_to` / `Vector3DMeasure::distance_to`
- 各 shape の `distance_to_point` 系（shape-local）

### operations へ移行候補

- `Circle2DMeasure::distance_to_circle`
- `Circle3DMeasure::distance_to_circle`
- `LineSegment2DMeasure::distance_to_segment`
- `LineSegment3DMeasure::distance_to_segment`
- `InfiniteLine3DMeasure::distance_to_line`
- `Ray3DMeasure::distance_to_ray`
- `LineSegment3DMeasure::distance_to_aabb`

## #344 実装ステップ

1. [x] `operations` に distance 契約モジュールを追加
2. [x] `operations/mod.rs` で distance 契約を再エクスポート
3. [x] `lib.rs` で distance 契約の再エクスポートを統一
4. [x] `core` 側 cross-shape distance を `#[deprecated]` で段階縮退開始
5. [x] `cargo check -p geo_contracts` で契約整合を確認

## 進捗メモ（2026-03-23）

- `core` の以下メソッドへ `#[deprecated]` を追加し、移行先を
  `geometry::operations::CrossDistance` に統一:
  - `Circle2DMeasure::distance_to_circle`
  - `Circle3DMeasure::distance_to_circle`
  - `InfiniteLine3DMeasure::distance_to_line`
  - `LineSegment2DMeasure::distance_to_segment`
  - `LineSegment3DMeasure::distance_to_segment`
  - `LineSegment3DCollisionDetection::distance_to_aabb`
  - `Ray3DMeasure::distance_to_ray`

## 互換方針

- 互換は最小限: 既存呼び出しを急激に壊さないため `deprecated` 経由で段階移行
- 完全撤去は #342 の実装移行後に実施

## 着手前チェック（#344 C1-C5）

- C1: Yes（#343 完了）
- C2: Yes（本ドキュメントで分類表を作成）
- C3: Yes（`operations` 一次参照 / `core` shape-local の責務分担を定義）
- C4: Yes（最小互換 + 段階縮退）
- C5: Yes（`geo_primitives` / `geo_nurbs` / `geo_algorithms` を移行対象として確認）
