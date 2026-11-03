# Issue #281 Phase1: ViewModel NURBS命名整合

- 対象Issue: #281
- 作成日: 2026-02-26
- スコープ: 命名修正のみ（挙動変更なし）

## 背景

ViewModel層に `nurbs_debug` というモジュール名が残っているが、実装責務は「NURBS評価データの生成・読込」であり、debug接頭辞方針（操作種別）との軸が混在している。

## 目的

- モジュール名を責務ベースへ変更して可読性を向上
- `debug/sample/dev` 接頭辞方針と整合

## 実施方針（Phase1）

- `viewmodel/converter/src/nurbs_debug.rs` → `nurbs_eval_loader.rs`
- エラー型名を責務名に追従
  - `NurbsDebugError` → `NurbsEvalLoaderError`
- 呼び出し側の最小追従
  - `view/app/src/app_state/debug_scene/nurbs.rs`
  - `viewmodel/converter/src/lib.rs`
- ロジック変更は行わない

## 非対象

- App層の `load_debug_nurbs*` API 改名
- NURBS評価処理ロジックの変更

## 検証

- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt`

## DoD

- 旧モジュール名/型名参照が残っていない
- 既存挙動を維持したままビルド・Lint・fmt が通る
