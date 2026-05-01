<#
.SYNOPSIS
    Rust フォーマッティングをチェック
.DESCRIPTION
    cargo fmt --all --check を実行してフォーマットエラーをチェック
    CI や push 前の手動チェック用
#>

$ErrorActionPreference = "Stop"

Write-Host "Checking Rust formatting..." -ForegroundColor Cyan
cargo fmt --all -- --check

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ All files are properly formatted" -ForegroundColor Green
    exit 0
} else {
    Write-Host "✗ Formatting issues found. Run 'cargo fmt --all' to fix." -ForegroundColor Red
    exit 1
}
