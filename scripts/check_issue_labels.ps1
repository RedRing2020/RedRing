<#
.SYNOPSIS
    開いている Issue のラベルが、dev/ISSUE_LABEL_OPERATION.md の決まりに沿っているかを確かめる

.DESCRIPTION
    種別がない・種別や優先度・リリースが 2 つ以上ある・決まりにないラベルがある・振り分ける前のまま日がたった
    Issue を一覧にし、1 件でもあれば失敗にする。-Fix を付けると、リリースのない Issue に release:triage、
    優先度のない Issue に priority: medium を付ける（中身を読まずに決まる既定だけ。ほかは付け替えない）。
    gh（認証済み）を使う。

.PARAMETER Fix
    リリース・優先度の既定を付ける

.PARAMETER Issue
    この番号の Issue だけを見る（作ったときに既定を付けるため。決まりの誤りでは失敗にせず、既定を付けられなかった
    ときだけ失敗にする）

.PARAMETER TriageDays
    作ってからこの日数より長く release:triage のままの Issue を、決まりに沿っていない Issue として出す
#>
param(
    [switch]$Fix,
    [int]$Issue = 0,
    [int]$TriageDays = 14
)

$ErrorActionPreference = 'Stop'
# gh の出力（ラベルの説明・題名の日本語）を UTF-8 で受け取る
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$types = @('bug', 'enhancement', 'design', 'refactor', 'tech-debt', 'documentation', 'investigation')
$layers = @('foundation', 'model', 'viewmodel', 'view')
$domains = @('architecture', 'cam', 'cutting-simulation', 'geometry', 'job', 'ui', 'performance', 'security', 'ci')
$priorities = @('priority: high', 'priority: medium', 'priority: low')
$releases = @('release:in', 'release:out', 'release:triage')
$others = @('on-hold')
$known = $types + $layers + $domains + $priorities + $releases + $others

$fields = 'number,title,labels,createdAt'
$limit = 1000
if ($Issue -gt 0) {
    $json = gh issue view $Issue --json $fields
} else {
    $json = gh issue list --state open --limit $limit --json $fields
}
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$issues = @($json | ConvertFrom-Json)

# 名前の一覧 $names のうち、軸 $axis のラベルの数
function Count-Axis($names, $axis) {
    @($names | Where-Object { $axis -contains $_ }).Count
}

$problems = [System.Collections.Generic.List[string]]::new()
$editFailed = $false
if ($Issue -eq 0 -and $issues.Count -ge $limit) {
    $problems.Add("- 開いている Issue が $limit 件に達したため、それより後の Issue は確かめていない")
}
foreach ($item in $issues) {
    $names = @($item.labels | ForEach-Object { $_.name })
    $reasons = @()
    $add = @()
    if ((Count-Axis $names $releases) -eq 0) { $add += 'release:triage' }
    if ((Count-Axis $names $priorities) -eq 0) { $add += 'priority: medium' }
    $triageAdded = $false
    if ($Fix -and $add.Count -gt 0) {
        $editArgs = @('issue', 'edit', "$($item.number)")
        foreach ($label in $add) { $editArgs += @('--add-label', $label) }
        gh @editArgs | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "#$($item.number): added $($add -join ', ')"
            $names += $add
            $triageAdded = $add -contains 'release:triage'
        } else {
            # 1 件で止めず、残りの Issue も確かめる
            if ($Issue -gt 0) { exit $LASTEXITCODE }
            $reasons += "既定のラベルを付けられなかった（$($add -join ', ')）"
            $editFailed = $true
        }
    }

    $typeCount = Count-Axis $names $types
    if ($typeCount -ne 1) { $reasons += "種別が $typeCount 個" }
    $priorityCount = Count-Axis $names $priorities
    if ($priorityCount -ne 1) { $reasons += "優先度が $priorityCount 個" }
    $releaseCount = Count-Axis $names $releases
    if ($releaseCount -ne 1) { $reasons += "リリースが $releaseCount 個" }
    $unknown = @($names | Where-Object { $known -notcontains $_ })
    if ($unknown.Count -gt 0) { $reasons += "決まりにないラベル: $($unknown -join ', ')" }
    # 日数は作った日から数える（この回に付けた release:triage は、振り分ける間がなかったため数えない）
    $age = ((Get-Date).ToUniversalTime() - ([datetime]$item.createdAt).ToUniversalTime()).TotalDays
    if (-not $triageAdded -and $names -contains 'release:triage' -and $age -gt $TriageDays) {
        $reasons += "作ってから $([int]$age) 日、release:triage のまま"
    }
    if ($reasons.Count -gt 0) {
        $problems.Add("- #$($item.number) $($item.title): $($reasons -join '、')")
    }
}

$report = if ($problems.Count -eq 0) {
    "開いている Issue $($issues.Count) 件のラベルは、決まりに沿っています。"
} else {
    "決まりに沿っていない Issue が $($problems.Count) 件あります（dev/ISSUE_LABEL_OPERATION.md）。`n`n" + ($problems -join "`n")
}
Write-Host $report
if ($env:GITHUB_STEP_SUMMARY) {
    "## Issue のラベル`n`n$report" | Out-File -FilePath $env:GITHUB_STEP_SUMMARY -Append -Encoding utf8
}
if ($editFailed -or ($Issue -eq 0 -and $problems.Count -gt 0)) { exit 1 }
exit 0
