<#
.SYNOPSIS
    geo_* クレートの呼び出し元のない公開メソッドと、簡易実装の記載の検出（警告のみ）
.DESCRIPTION
    規約「基本の形状定義と最低限のメソッドから始め、呼び出し元が必要になった時点で追加する」からの
    逸脱の兆候を一覧にします。
    警告のみを表示し、失敗にはしません。

    検出対象:
    - 呼び出し元のない公開メソッド: geo_* クレートの impl ブロック直下の `pub fn` のうち、
      テスト以外のどのファイルにも `.名前(` / `::名前(` の呼び出しがないもの
      （名前だけで判定するため、別の型の同名メソッドの呼び出しがあれば検出しない）
    - 簡易実装の記載: 「簡易実装」「簡略化」「仮実装」「未実装」「TODO」「FIXME」「todo!(」「unimplemented!(」

    意図して残す公開メソッドは scripts/unused_public_api_allowlist.txt に `ファイルのパス:メソッド名` で記載します。
    簡易実装の記載の誤検出は、行末に「api-check: allow」を付けると除外できます。
.EXAMPLE
    pwsh scripts/check_unused_public_api.ps1
#>

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    $crates = @("geo_core", "geo_primitives", "geo_nurbs", "geo_algorithms", "geo_topology", "geo_commons")
    $isTestPath = { param($path) $path -match '(^|/)tests?/' -or $path -match 'test' }

    $allRust = git ls-files -- "*.rs"
    if ($LASTEXITCODE -ne 0) {
        throw "git ls-files failed"
    }
    $nonTestRust = @($allRust | Where-Object { -not (& $isTestPath $_) })
    $targetFiles = @($nonTestRust | Where-Object {
        $path = $_
        $crates | Where-Object { $path -like "model/$_/src/*" }
    })

    # テスト以外のファイルに現れる呼び出し名
    $callRegex = [regex]'(?:\.|::)(\w+)\s*(?:::<[^>]*>)?\('
    $calledNames = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($path in $nonTestRust) {
        $text = Get-Content -LiteralPath $path -Raw -Encoding UTF8
        foreach ($match in $callRegex.Matches($text)) {
            [void]$calledNames.Add($match.Groups[1].Value)
        }
    }

    $allowlistPath = Join-Path $PSScriptRoot "unused_public_api_allowlist.txt"
    $allowlist = [System.Collections.Generic.HashSet[string]]::new()
    if (Test-Path $allowlistPath) {
        foreach ($line in Get-Content -LiteralPath $allowlistPath -Encoding UTF8) {
            $entry = ($line -replace '#.*$', '').Trim()
            if ($entry) {
                [void]$allowlist.Add($entry)
            }
        }
    }

    $ignoredNames = @("new", "fmt", "default", "from")
    $methodRegex = '^    pub fn (\w+)\s*[(<]'
    $markerRegex = '簡易実装|簡略化|仮実装|未実装|TODO|FIXME|todo!\(|unimplemented!\('
    $allowMarkerRegex = 'api-check: allow\s*$'

    $unused = @()
    $markers = @()
    foreach ($path in $targetFiles) {
        $lines = @(Get-Content -LiteralPath $path -Encoding UTF8)
        for ($i = 0; $i -lt $lines.Length; $i++) {
            $line = $lines[$i]
            if ($line -match $methodRegex) {
                $name = $Matches[1]
                if ($ignoredNames -notcontains $name -and -not $calledNames.Contains($name) -and
                    -not $allowlist.Contains("${path}:$name")) {
                    $unused += [PSCustomObject]@{ Path = $path; Line = $i + 1; Name = $name }
                }
            }
            # doc コメントのテスト例（`/// # let ... = todo!();`）は対象外とする
            if ($line -match $markerRegex -and $line -notmatch '^\s*///\s*#' -and $line -notmatch $allowMarkerRegex) {
                $markers += [PSCustomObject]@{ Path = $path; Line = $i + 1; Text = $line.Trim() }
            }
        }
    }
}
finally {
    Pop-Location
}

if ($unused.Count -eq 0 -and $markers.Count -eq 0) {
    Write-Host "✓ 呼び出し元のない公開メソッド・簡易実装の記載なし（$($targetFiles.Count) ファイルを検査）" -ForegroundColor Green
    exit 0
}

if ($unused.Count -gt 0) {
    Write-Host "⚠ テスト以外から呼ばれない公開メソッド: $($unused.Count) 件" -ForegroundColor Yellow
    foreach ($item in $unused) {
        Write-Host "  $($item.Path):$($item.Line) $($item.Name)" -ForegroundColor Yellow
    }
}
if ($markers.Count -gt 0) {
    Write-Host "⚠ 簡易実装・TODO の記載: $($markers.Count) 件" -ForegroundColor Yellow
    foreach ($item in $markers) {
        Write-Host "  $($item.Path):$($item.Line) $($item.Text)" -ForegroundColor Yellow
    }
}
Write-Host "  呼び出し元が必要になるまで追加しない・別名を置かない・結果が誤った簡易実装を残さない。" -ForegroundColor Yellow
Write-Host "  意図して残す場合は scripts/unused_public_api_allowlist.txt に記載する" -ForegroundColor Yellow
exit 0
