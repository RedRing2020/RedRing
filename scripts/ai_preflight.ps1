param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ssK"
$branch = $null
try {
    $branchRaw = (& git -C $repoRoot rev-parse --abbrev-ref HEAD 2>$null)
    if ($LASTEXITCODE -eq 0 -and $branchRaw) {
        $branch = ($branchRaw | Select-Object -First 1).Trim()
    }
} catch {
    $branch = $null
}
$currentDir = (Get-Location).Path

$ruleFiles = @(
    ".github/copilot-instructions.md",
    "dev/GIT_PR_WORKFLOW_OPERATION.md",
    "dev/ISSUE_LABEL_OPERATION.md"
)

$checks = @()
$missingFiles = @()
foreach ($relativePath in $ruleFiles) {
    $absolutePath = Join-Path $repoRoot $relativePath
    $exists = Test-Path $absolutePath
    $sha256 = $null

    if ($exists) {
        $sha256 = (Get-FileHash -Algorithm SHA256 $absolutePath).Hash
    } else {
        $missingFiles += $relativePath
    }

    $checks += [ordered]@{
        path = $relativePath
        exists = $exists
        sha256 = $sha256
    }
}

$errors = @()
if ([string]::IsNullOrWhiteSpace($branch)) {
    $errors += "Failed to resolve current git branch."
}
if ($branch -eq "develop") {
    $errors += "Branch is develop. Implementation work is not allowed on develop."
}
if ($missingFiles.Count -gt 0) {
    $errors += "Missing rule files: $($missingFiles -join ', ')"
}

$status = if ($errors.Count -eq 0) { "OK" } else { "NG" }
$branchForRecord = if ([string]::IsNullOrWhiteSpace($branch)) { "(unknown)" } else { $branch }

$logDir = Join-Path $repoRoot "logs/ai_preflight"
New-Item -ItemType Directory -Path $logDir -Force | Out-Null
$logFile = Join-Path $logDir ("preflight-" + (Get-Date -Format "yyyyMMdd-HHmmss-fff") + ".json")

$record = [ordered]@{
    timestamp = $timestamp
    repoRoot = $repoRoot
    currentDirectory = $currentDir
    branch = $branchForRecord
    status = $status
    checks = $checks
    errors = $errors
}

$record | ConvertTo-Json -Depth 6 | Set-Content -Path $logFile -Encoding utf8

Write-Output ("AI_PREFLIGHT_STATUS=" + $status)
Write-Output ("AI_PREFLIGHT_BRANCH=" + $branchForRecord)
Write-Output ("AI_PREFLIGHT_LOG=" + $logFile)

if ($status -ne "OK") {
    foreach ($e in $errors) {
        Write-Output ("AI_PREFLIGHT_ERROR=" + $e)
    }
    exit 1
}

Write-Output "AI_PREFLIGHT_OK"
exit 0
