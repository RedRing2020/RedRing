<#
.SYNOPSIS
    ソースコード中の GitHub 固有情報（Issue 番号・PR 番号・URL）の検出
.DESCRIPTION
    運用ルール「ソースコード本文・コメントへ GitHub 固有情報を記載しない」の違反を検出して失敗します。

    対象:
    - git 追跡対象の *.rs / *.ps1 / *.sh / *.wgsl / *.toml / *.yml / *.yaml
    - 除外: dev/ / docs/ / manual/ / .github/ 配下（文書・運用設定では参照を許容）

    検出パターン:
    - 番号付き参照: 「#」の直後に数字が続くもの（Rust 属性の「#[...]」は対象外）
    - 「Issue」「PR」「Pull Request」の直後の番号
    - GitHub の issues / pull の URL

    誤検出（数字のみの 16 進カラーコード等）は、行末に「github-ref-check: allow」を付けると除外できます。
.EXAMPLE
    pwsh scripts/check_github_refs_in_source.ps1
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    $extensions = @("*.rs", "*.ps1", "*.sh", "*.wgsl", "*.toml", "*.yml", "*.yaml")
    $files = git ls-files -- $extensions |
        Where-Object { $_ -notmatch '^(dev|docs|manual|\.github)/' }
    if ($LASTEXITCODE -ne 0) {
        throw "git ls-files failed"
    }

    $patterns = @(
        # 番号の後ろは \b ではなく英数字以外で判定する（日本語が続く場合も検出するため）
        @{ Rule = "numbered reference"; Regex = '(?<![\w&/])#\d{1,6}(?![0-9A-Za-z_])' },
        @{ Rule = "Issue/PR number"; Regex = '(?i)\b(?:issue|pr|pull request)\s*#?\s*\d+' },
        @{ Rule = "GitHub issue/pull URL"; Regex = 'github\.com/[^\s/]+/[^\s/]+/(?:issues|pull)/\d+' }
    )
    $allowMarker = 'github-ref-check: allow'

    $violations = @()
    foreach ($path in $files) {
        $lines = @(Get-Content -LiteralPath $path -Encoding UTF8)
        for ($i = 0; $i -lt $lines.Length; $i++) {
            $line = $lines[$i]
            if ($line.Contains($allowMarker)) {
                continue
            }
            foreach ($pattern in $patterns) {
                if ($line -match $pattern.Regex) {
                    $violations += [PSCustomObject]@{
                        Path = $path
                        Line = $i + 1
                        Rule = $pattern.Rule
                        Text = $line.Trim()
                    }
                    break
                }
            }
        }
    }
}
finally {
    Pop-Location
}

if ($violations.Count -gt 0) {
    Write-Host "GitHub 固有情報（Issue 番号・PR 番号・URL）がソースコードに含まれています:" -ForegroundColor Red
    foreach ($violation in $violations) {
        Write-Host ("  {0}:{1} [{2}] {3}" -f $violation.Path, $violation.Line, $violation.Rule, $violation.Text)
    }
    Write-Host ""
    Write-Host "Issue 番号ではなく、機能・状態・設計書の節（例: 設計: CAM_ALGORITHMS_DESIGN.md §8）で記載してください。" -ForegroundColor Yellow
    exit 1
}

Write-Host ("✓ GitHub 固有情報の記載なし（{0} ファイルを検査）" -f @($files).Count) -ForegroundColor Green
exit 0
