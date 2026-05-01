<#
.SYNOPSIS
    Git pre-commit フックをセットアップ
.DESCRIPTION
    このスクリプトは .git/hooks/pre-commit を作成し、
    コミット時に自動的に品質チェックが実行されるようにします。
    
    セットアップ後、通常の git commit で自動的にチェックが走ります。
    
.EXAMPLE
    pwsh scripts/setup_hooks.ps1
#>

$ErrorActionPreference = "Stop"

$hooksDir = ".git/hooks"
$preCommitPath = "$hooksDir/pre-commit"
$preCommitContent = @'
#!/bin/sh
# Auto-generated pre-commit hook
# This script is automatically created by scripts/setup_hooks.ps1

# フォーマットチェック
echo "[pre-commit] Checking formatting..."
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo ""
    echo "x Format check failed."
    echo "Run 'cargo fmt --all' to fix."
    exit 1
fi

echo "v Format check passed."
exit 0
'@

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Git Pre-commit フック セットアップ" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# .git/hooks/ ディレクトリ確認
if (!(Test-Path $hooksDir)) {
    Write-Host "x エラー: .git/hooks/ が見つかりません" -ForegroundColor Red
    Write-Host "   このスクリプトはリポジトリルートから実行してください" -ForegroundColor Red
    exit 1
}

# pre-commit フックを作成
Write-Host "Pre-commit フックを作成中..." -ForegroundColor Yellow

# 既存の pre-commit があれば バックアップ
if (Test-Path $preCommitPath) {
    Copy-Item $preCommitPath "$preCommitPath.backup" -Force
    Write-Host "  既存のフックを $preCommitPath.backup にバックアップしました" -ForegroundColor Yellow
}

# BOM なし UTF-8 で書き出す（shebang 行が壊れないよう BOM を避ける）
$fullPath = [System.IO.Path]::GetFullPath($preCommitPath)
[System.IO.File]::WriteAllText($fullPath, $preCommitContent, [System.Text.Encoding]::UTF8)
Write-Host "  $preCommitPath を作成しました" -ForegroundColor Green

# Unix 系では実行権限が必要
if ($IsLinux -or $IsMacOS) {
    chmod +x $fullPath
    Write-Host "  実行権限を付与しました (chmod +x)" -ForegroundColor Green
} else {
    Write-Host "  注意: macOS/Linux 環境では 'chmod +x .git/hooks/pre-commit' が必要です" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "v セットアップ完了" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "以下の動作が追加されました：" -ForegroundColor Cyan
Write-Host "  - git commit 時に自動的にフォーマットチェックが実行"
Write-Host "  - フォーマット エラーがある場合、コミットが拒否"
Write-Host "  - 修正後に再度 git commit を実行してください" -ForegroundColor Cyan
Write-Host ""
Write-Host "テスト:" -ForegroundColor Yellow
Write-Host "  git commit -m 'test' で フック動作確認" -ForegroundColor Gray

