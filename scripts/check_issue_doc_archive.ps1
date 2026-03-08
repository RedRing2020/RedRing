param(
    [string]$Repository = "RedRing2020/RedRing",
    [string]$DevRoot = "dev",
    [switch]$ExitOnError
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()

function Write-ColorText {
    param(
        [string]$Text,
        [string]$Color = "White"
    )
    Write-Host $Text -ForegroundColor $Color
}

function Get-IssueState {
    param(
        [string]$Repo,
        [string]$IssueNumber
    )

    try {
        return (gh issue view $IssueNumber -R $Repo --json state --jq '.state' 2>$null)
    }
    catch {
        return "UNKNOWN"
    }
}

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-ColorText "ERROR: gh command not found. Install GitHub CLI first." "Red"
    exit 1
}

$workspaceRoot = (Resolve-Path ".").Path
$devPath = Join-Path $workspaceRoot $DevRoot
$archivePath = Join-Path $devPath "archive\issues"

if (-not (Test-Path $devPath)) {
    Write-ColorText "ERROR: DevRoot does not exist: $devPath" "Red"
    exit 1
}

Write-ColorText "=== Issue Document Archive Check ===" "Cyan"
Write-ColorText "Date: $(Get-Date -Format 'yyyy/MM/dd HH:mm:ss')" "Gray"
Write-ColorText "Repository: $Repository" "Gray"
Write-ColorText "DevRoot: $DevRoot" "Gray"
Write-Host ""

$activeIssueFiles = Get-ChildItem -Path $devPath -Recurse -File -Filter "ISSUE_*.md" |
    Where-Object {
        $_.FullName -notlike "*$([IO.Path]::DirectorySeparatorChar)archive$([IO.Path]::DirectorySeparatorChar)issues$([IO.Path]::DirectorySeparatorChar)*"
    }

$archivedIssueFiles = @()
if (Test-Path $archivePath) {
    $archivedIssueFiles = Get-ChildItem -Path $archivePath -Recurse -File -Filter "ISSUE_*.md"
}

$closedOutsideArchive = @()
$openInsideArchive = @()
$unknownStateFiles = @()

foreach ($file in $activeIssueFiles) {
    if ($file.Name -match '^ISSUE_(\d+)_.*\.md$') {
        $issueNumber = $Matches[1]
        $state = Get-IssueState -Repo $Repository -IssueNumber $issueNumber
        $relativePath = $file.FullName.Substring($workspaceRoot.Length + 1).Replace('\\', '/')

        switch ($state) {
            "CLOSED" { $closedOutsideArchive += "$relativePath (#$issueNumber)" }
            "OPEN" { }
            default { $unknownStateFiles += "$relativePath (#$issueNumber): $state" }
        }
    }
}

foreach ($file in $archivedIssueFiles) {
    if ($file.Name -match '^ISSUE_(\d+)_.*\.md$') {
        $issueNumber = $Matches[1]
        $state = Get-IssueState -Repo $Repository -IssueNumber $issueNumber
        $relativePath = $file.FullName.Substring($workspaceRoot.Length + 1).Replace('\\', '/')

        switch ($state) {
            "OPEN" { $openInsideArchive += "$relativePath (#$issueNumber)" }
            "CLOSED" { }
            default { $unknownStateFiles += "$relativePath (#$issueNumber): $state" }
        }
    }
}

Write-ColorText "1) Closed issue docs outside archive" "Yellow"
if ($closedOutsideArchive.Count -eq 0) {
    Write-ColorText "  OK: 0" "Green"
}
else {
    Write-ColorText "  NG: $($closedOutsideArchive.Count)" "Red"
    $closedOutsideArchive | ForEach-Object { Write-ColorText "    - $_" "Red" }
}

Write-Host ""
Write-ColorText "2) Open issue docs inside archive" "Yellow"
if ($openInsideArchive.Count -eq 0) {
    Write-ColorText "  OK: 0" "Green"
}
else {
    Write-ColorText "  NG: $($openInsideArchive.Count)" "Red"
    $openInsideArchive | ForEach-Object { Write-ColorText "    - $_" "Red" }
}

Write-Host ""
Write-ColorText "3) Unknown issue state" "Yellow"
if ($unknownStateFiles.Count -eq 0) {
    Write-ColorText "  OK: 0" "Green"
}
else {
    Write-ColorText "  WARN: $($unknownStateFiles.Count)" "Yellow"
    $unknownStateFiles | ForEach-Object { Write-ColorText "    - $_" "Yellow" }
}

$errorCount = $closedOutsideArchive.Count + $openInsideArchive.Count
Write-Host ""
if ($errorCount -eq 0) {
    Write-ColorText "SUCCESS: No archive placement violations found." "Green"
    exit 0
}

Write-ColorText "FAILED: Found $errorCount archive placement violation(s)." "Red"
if ($ExitOnError) {
    exit 1
}
