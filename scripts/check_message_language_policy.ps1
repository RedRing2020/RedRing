<#
.SYNOPSIS
    ログとエラーの文の言語ポリシーチェック（英語に統一する）
.DESCRIPTION
    model/・viewmodel/・view/・foundation/ 配下の Rust ソースで、次の呼び出しの文字列に日本語が含まれると失敗します。
    - tracing のログマクロ（trace!・debug!・info!・warn!・error!。`tracing::` を付けない呼び出しも含む）
    - write!・writeln!（Display 実装のエラーの文）
    - thiserror の #[error("...")]
    - コンソールへの出力（eprintln!・eprint!・println!・print!）
    - パニックの文（panic!・unreachable!・.expect("...")）

    画面の文言は、日本語・英語の文言の表（*_messages.rs・*message_catalog.rs の表）に置きます。表はマクロの呼び出しでは
    ないため、このチェックにかかりません。

    注意:
    - コメント、テスト（tests.rs・tests/・#[cfg(test)] を付けたモジュール）、examples/・benches/ は対象外です。
    - 呼び出しの終わりは、文字列の外の括弧の対応で決めます。
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$roots = @("model", "viewmodel", "view", "foundation") |
    ForEach-Object { Join-Path $repoRoot $_ } |
    Where-Object { Test-Path $_ }

$files = $roots | ForEach-Object { Get-ChildItem -Path $_ -Recurse -Filter *.rs -File } |
    Where-Object {
        $relative = $_.FullName.Substring($repoRoot.Length + 1)
        $relative -notmatch "(^|[\\/])(tests|examples|benches|target)[\\/]" -and
        $_.Name -notlike "*_test.rs" -and
        $_.Name -notlike "*_tests.rs" -and
        $_.Name -ne "tests.rs"
    }

# 日本語判定
$jpPattern = '[ぁ-んァ-ヶ一-龯々ー]'

$rules = @(
    @{ Pattern = '(?<![\w])(?:trace|debug|info|warn|error)!\s*\('; Rule = "log message must be English" },
    @{ Pattern = '(?<![\w])write(?:ln)?!\s*\('; Rule = "write! message must be English" },
    @{ Pattern = '#\[error\('; Rule = "error message must be English" },
    @{ Pattern = '(?<![\w])e?print(?:ln)?!\s*\('; Rule = "console output must be English" },
    @{ Pattern = '(?<![\w])(?:panic|unreachable)!\s*\(|\.expect\s*\('; Rule = "panic message must be English" }
)

# 行 $start の、パターンに合った位置 $column から、括弧が閉じるまでの文字列リテラルの中身を返す
function Get-CallStrings([string[]]$lines, [int]$start, [int]$column) {
    $depth = 0
    $inString = $false
    $escape = $false
    $strings = New-Object System.Text.StringBuilder
    $last = [Math]::Min($start + 30, $lines.Length - 1)
    for ($j = $start; $j -le $last; $j++) {
        $text = $lines[$j]
        $k = if ($j -eq $start) { $column } else { 0 }
        while ($k -lt $text.Length) {
            $c = $text[$k]
            if ($inString) {
                if ($escape) { $escape = $false }
                elseif ($c -eq '\') { $escape = $true }
                elseif ($c -eq '"') { $inString = $false }
                else { [void]$strings.Append($c) }
            }
            else {
                if ($c -eq '/' -and $k + 1 -lt $text.Length -and $text[$k + 1] -eq '/') { break }
                if ($c -eq "'" -and $k + 2 -lt $text.Length -and $text[$k + 2] -eq "'") { $k += 3; continue }
                if ($c -eq '"') { $inString = $true }
                elseif ($c -eq '(') { $depth++ }
                elseif ($c -eq ')') {
                    $depth--
                    if ($depth -eq 0) { return $strings.ToString() }
                }
            }
            $k++
        }
        if ($inString) { [void]$strings.Append("`n") }
    }
    return $strings.ToString()
}

$violations = @()

foreach ($file in $files) {
    $relativePath = $file.FullName.Substring($repoRoot.Length + 1)
    $lines = @(Get-Content -Path $file.FullName)

    $skipDepth = -1      # テストのモジュールを読み飛ばしている間の、波括弧の深さ（-1 は読み飛ばしていない）
    $pendingTest = $false
    for ($i = 0; $i -lt $lines.Length; $i++) {
        $line = $lines[$i]
        $code = $line -replace '//.*$', ''

        if ($skipDepth -ge 0) {
            $skipDepth += ([regex]::Matches($code, '\{')).Count - ([regex]::Matches($code, '\}')).Count
            if ($skipDepth -le 0) { $skipDepth = -1 }
            continue
        }
        if ($code -match '^\s*#\[cfg\(test\)\]') {
            $pendingTest = $true
            continue
        }
        if ($pendingTest) {
            if ($code -match '^\s*$' -or $code -match '^\s*#\[') { continue }
            $pendingTest = $false
            $opened = ([regex]::Matches($code, '\{')).Count - ([regex]::Matches($code, '\}')).Count
            if ($code -match '\{' -and $opened -gt 0) { $skipDepth = $opened }
            continue
        }
        if ($line -match '^\s*//') {
            continue
        }

        foreach ($rule in $rules) {
            $match = [regex]::Match($code, $rule.Pattern)
            if (-not $match.Success) {
                continue
            }
            $strings = Get-CallStrings $lines $i $match.Index
            if ($strings -match $jpPattern) {
                $violations += [PSCustomObject]@{
                    Path = $relativePath
                    Line = $i + 1
                    Rule = $rule.Rule
                    Text = $line.Trim()
                }
            }
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "ログとエラーの文の言語ポリシー違反を検出しました（英語にしてください。画面の文言は文言の表へ）:" -ForegroundColor Red
    foreach ($v in $violations) {
        Write-Host ("- {0}:{1} [{2}]" -f $v.Path, $v.Line, $v.Rule) -ForegroundColor Red
        Write-Host ("  {0}" -f $v.Text) -ForegroundColor DarkRed
    }
    exit 1
}

Write-Host "ログとエラーの文の言語ポリシーチェック: OK" -ForegroundColor Green
exit 0
