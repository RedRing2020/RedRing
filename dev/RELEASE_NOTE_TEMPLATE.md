# RELEASE NOTE TEMPLATE

> このテンプレートは RedRing のリリースノート作成用です。
> リリース時にコピーして、`dev/releases/` または GitHub Release 本文に転記してください。

## Release Information

- Version: `vX.Y.Z`
- Release Date: `YYYY-MM-DD`
- Target Branch: `main`
- Related Milestone: `<milestone-name>`
- Related PRs: `#...`

## Summary

今回リリースの要約を 3〜5 行で記載。

## Highlights

### Added

- 

### Changed

- 

### Fixed

- 

### Deprecated

- 

### Removed

- 

## Breaking Changes

- なし / あり（内容を明記）

## Migration Guide

既存利用者が必要な移行手順を記載。

1. 
2. 
3. 

## Validation

- [ ] `cargo build`
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] `./scripts/check_architecture_dependencies_simple.ps1`
- [ ] `mdbook build`

## Known Issues

- 

## Notes

- タグ: `vX.Y.Z`
- 次サイクルに持ち越す項目:
  - 

## Initial Release Checklist (v0.1.0)

初回リリース時は以下を必須チェックとして使用する。

- [ ] **Version**: `v0.1.0` を確定
- [ ] **Release Date**: 公開日を確定
- [ ] **Related PRs**: 主要PR（統合・CI修正・運用文書）を列挙
- [ ] **Summary**: 3〜5行で今回の到達点を要約
- [ ] **Highlights**: `Added / Changed / Fixed` を最低1項目ずつ記載
- [ ] **Breaking Changes**: なし/ありを明示（ありの場合は影響範囲を具体化）
- [ ] **Migration Guide**: 利用者の移行手順（必要な場合）
- [ ] **Validation**: テンプレートのチェック項目を全て実施して結果反映
- [ ] **Known Issues**: 既知の制約を明記（なければ「なし」）
- [ ] **Tag**: `main` マージ後に `v0.1.0` タグ作成を確認
- [ ] **CHANGELOG**: `Unreleased` から `0.1.0` への反映を確認
