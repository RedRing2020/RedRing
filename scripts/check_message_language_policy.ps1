<#
.SYNOPSIS
    model層の実行時文言に対する言語ポリシーチェック
.DESCRIPTION
    以下の違反を検出して失敗します。
    - model/ 配下の Rust ソースで、Display 実装の write!(f, "...") に日本語が含まれる
    - model/ 配下の Rust ソースで、tracing ログマクロの文字列に日本語が含まれる

    注意:
    - 日本語コメントやテストメッセージはこのチェック対象外です。
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$modelRoot = Join-Path $repoRoot "model"

if (-not (Test-Path $modelRoot)) {
    Write-Host "model ディレクトリが見つからないためチェックをスキップします。" -ForegroundColor Yellow
    exit 0
}

$files = Get-ChildItem -Path $modelRoot -Recurse -Filter *.rs -File |
    Where-Object {
        $_.FullName -notmatch "[\\/]tests[\\/]" -and
        $_.Name -notlike "*_test.rs" -and
        $_.Name -ne "tests.rs"
    }

$violations = @()

# 日本語判定
$jpPattern = '[ぁ-んァ-ヶ一-龯々ー]'

foreach ($file in $files) {
    $relativePath = $file.FullName.Substring($repoRoot.Length + 1)

    $lines = Get-Content -Path $file.FullName
    for ($i = 0; $i -lt $lines.Length; $i++) {
        $line = $lines[$i]

        if ($line -match 'write!\s*\(\s*f\s*,') {
            $end = [Math]::Min($i + 4, $lines.Length - 1)
            $windowLines = $lines[$i..$end] | Where-Object { $_ -notmatch '^\s*//' }
            $window = ($windowLines -join "`n")
            if ($window -match $jpPattern) {
                $violations += [PSCustomObject]@{
                    Path = $relativePath
                    Line = $i + 1
                    Rule = "Display write! message must be English in model/"
                    Text = $line.Trim()
                }
            }
        }

        if ($line -match 'tracing::(?:trace|debug|info|warn|error)!\s*\(') {
            $end = [Math]::Min($i + 4, $lines.Length - 1)
            $windowLines = $lines[$i..$end] | Where-Object { $_ -notmatch '^\s*//' }
            $window = ($windowLines -join "`n")
            if ($window -match $jpPattern) {
                $violations += [PSCustomObject]@{
                    Path = $relativePath
                    Line = $i + 1
                    Rule = "tracing log message must be English in model/"
                    Text = $line.Trim()
                }
            }
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "実行時文言の言語ポリシー違反を検出しました:" -ForegroundColor Red
    foreach ($v in $violations) {
        Write-Host ("- {0}:{1} [{2}]" -f $v.Path, $v.Line, $v.Rule) -ForegroundColor Red
        Write-Host ("  {0}" -f $v.Text) -ForegroundColor DarkRed
    }
    exit 1
}

Write-Host "実行時文言の言語ポリシーチェック: OK" -ForegroundColor Green
exit 0
