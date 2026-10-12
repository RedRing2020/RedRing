<#
.SYNOPSIS
    Push前の完全なコード品質チェック
.DESCRIPTION
    次のチェックを順番に実行：
    1. cargo fmt --all -- --check (フォーマットチェック)
    2. cargo clippy --all-targets --all-features --workspace -- -D warnings (リント)
    3. cargo test --workspace (テスト)
    4. scripts/check_message_language_policy.ps1 (実行時文言の言語ポリシーチェック)
    5. scripts/check_github_refs_in_source.ps1 (ソースコード中の GitHub 固有情報チェック)
    6. scripts/check_unused_public_api.ps1 (呼び出し元のない公開メソッド・trait定義のメソッド・簡易実装の記載。警告のみ)
    
    1〜5 のチェックが通らないと、このスクリプトは失敗します。6 は警告のみで失敗にしません。
    
.EXAMPLE
    pwsh scripts/check_all_before_push.ps1
#>

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Push前の完全チェック" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. フォーマットチェック
Write-Host "[1/6] フォーマットチェック..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ フォーマッティングエラーを検出" -ForegroundColor Red
    Write-Host "実行: cargo fmt --all" -ForegroundColor Yellow
    exit 1
}
Write-Host "✓ フォーマットOK" -ForegroundColor Green
Write-Host ""

# 2. Clippy チェック
Write-Host "[2/6] Clippy リントチェック..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features --workspace -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ Clippy警告を検出" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Clippy OK" -ForegroundColor Green
Write-Host ""

# 3. テスト実行
Write-Host "[3/6] テスト実行..." -ForegroundColor Yellow
cargo test --workspace
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ テスト失敗" -ForegroundColor Red
    exit 1
}
Write-Host "✓ テスト OK" -ForegroundColor Green
Write-Host ""

# 4. 実行時文言の言語ポリシーチェック
Write-Host "[4/6] 実行時文言の言語ポリシーチェック..." -ForegroundColor Yellow
$messagePolicyScript = Join-Path $PSScriptRoot "check_message_language_policy.ps1"
pwsh -NoProfile -ExecutionPolicy Bypass -File $messagePolicyScript
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ 実行時文言の言語ポリシー違反を検出" -ForegroundColor Red
    exit 1
}
Write-Host "✓ 言語ポリシー OK" -ForegroundColor Green
Write-Host ""

# 5. ソースコード中の GitHub 固有情報チェック
Write-Host "[5/6] ソースコード中の GitHub 固有情報チェック..." -ForegroundColor Yellow
$githubRefsScript = Join-Path $PSScriptRoot "check_github_refs_in_source.ps1"
pwsh -NoProfile -ExecutionPolicy Bypass -File $githubRefsScript
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "✗ ソースコード中の GitHub 固有情報を検出" -ForegroundColor Red
    exit 1
}
Write-Host ""

# 6. 呼び出し元のない公開メソッド・trait定義のメソッド・簡易実装の記載（警告のみ）
Write-Host "[6/6] 呼び出し元のない公開メソッド・trait定義のメソッド・簡易実装の記載チェック（警告のみ）..." -ForegroundColor Yellow
$unusedApiScript = Join-Path $PSScriptRoot "check_unused_public_api.ps1"
pwsh -NoProfile -ExecutionPolicy Bypass -File $unusedApiScript
Write-Host ""

Write-Host "========================================" -ForegroundColor Green
Write-Host "✓ すべてのチェックが通りました" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Push準備完了。" -ForegroundColor Cyan
exit 0
