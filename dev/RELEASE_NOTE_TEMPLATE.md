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
