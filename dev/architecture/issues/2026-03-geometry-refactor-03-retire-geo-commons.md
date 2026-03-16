## 概要

`geo_commons` を廃止し、責務を `analysis` と `geo_algorithms` に分解移管する。

## 背景

- `geo_commons` が中間層として残っており、責務と依存が追いにくい
- `geo_foundation` 廃止と同時に構成を単純化したい

## タスク

### Phase A: API棚卸し

- [x] `geo_commons` 公開関数24件を棚卸し
- [x] `analysis` / `geo_algorithms` への分類方針を確定
- [x] 依存元クレート（現状: `geo_foundation` のみ）を確認

### Phase B: analysis移管

- [ ] `metrics::area_volume::*` を `geo_algorithms` へ移管
- [ ] `approximations::*` を `geo_algorithms` へ移管
- [ ] `geo_foundation` 側の参照を `geo_algorithms` 基準へ切替
- [ ] `cargo check -p geo_algorithms -p geo_foundation`
- [ ] `cargo test -p geo_algorithms -p geo_foundation`

### Phase C: geo_algorithms移管

- [ ] `analysis` には形状を含む関数を残さないことを確認
- [ ] 必要なら `geo_algorithms` 側で数値基盤ヘルパーを整理
- [ ] `cargo check -p analysis -p geo_algorithms`

### Phase D: geo_commons撤去

- [ ] `geo_foundation` の `geo_commons` 依存を削除
- [ ] `model/geo_commons` クレートを削除
- [ ] ワークスペース参照/依存ルールを更新
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `powershell -NoProfile -ExecutionPolicy Bypass -File .\\scripts\\check_architecture_dependencies.ps1 -ExitOnError`

## 受け入れ条件

- [ ] ワークスペースに `geo_commons` 依存が存在しない
- [ ] `analysis` は形状を含まない純粋数値計算のみを保持
- [ ] 面積/体積/近似/距離を含む形状計算は `geo_*` 側に集約されている
- [ ] 移管後APIで既存利用箇所がビルド通過
- [ ] `cargo fmt --all -- --check` が通る
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` が通る
- [ ] `cargo check --workspace` が通る
- [ ] `cargo test --workspace` が通る

## 備考

- 分割移管の途中で中間PRを許容する
- 実施詳細は `dev/architecture/ISSUE_320_IMPLEMENTATION_PREP.md` を参照
