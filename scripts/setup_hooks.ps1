<#
.SYNOPSIS
    Git pre-commit フックをセットアップ
.DESCRIPTION
    このスクリプトは .git/hooks/pre-commit を作成し、
    コミット時に自動的にフォーマットチェックが実行されるようにします。
    
    セットアップ後、通常の git commit で自動的にチェックが走ります。
    
.EXAMPLE
    pwsh scripts/setup_hooks.ps1
#>

$ErrorActionPreference = "Stop"

# git rev-parse --git-path hooks でフックの実パスを解決（git worktree 対応）
$hooksDir = (git rev-parse --git-path hooks 2>$null).Trim()
if (!$hooksDir -or $LASTEXITCODE -ne 0) {
    Write-Host "x エラー: git hooks ディレクトリを解決できません" -ForegroundColor Red
    Write-Host "   このスクリプトはリポジトリルートから実行してください" -ForegroundColor Red
    exit 1
}
$preCommitPath = Join-Path $hooksDir 'pre-commit'

# pre-commit.template を正本として読み込む
$templatePath = "scripts/hooks/pre-commit.template"
if (!(Test-Path $templatePath)) {
    Write-Host "x エラー: $templatePath が見つかりません" -ForegroundColor Red
    exit 1
}
$preCommitContent = Get-Content -Raw -Path $templatePath

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Git Pre-commit フック セットアップ" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# hooks ディレクトリ確認
if (!(Test-Path $hooksDir)) {
    Write-Host "x エラー: $hooksDir が見つかりません" -ForegroundColor Red
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

# BOM なし UTF-8 / LF 改行で書き出す（shebang 行が壊れないよう BOM と CRLF を避ける）
$fullPath = [System.IO.Path]::GetFullPath($preCommitPath)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$preCommitContentLf = $preCommitContent -replace "`r`n", "`n" -replace "`r", "`n"
[System.IO.File]::WriteAllText($fullPath, $preCommitContentLf, $utf8NoBom)
Write-Host "  $preCommitPath を作成しました" -ForegroundColor Green

# Unix 系では実行権限が必要
if ($IsLinux -or $IsMacOS) {
    chmod +x -- "$fullPath"
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

