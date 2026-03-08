param(
    [switch]$SkipClippy,
    [switch]$SkipTests,
    [switch]$AllowDevelop
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "[STEP] $Message" -ForegroundColor Cyan
}

function Write-Ok {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Run-Or-Throw {
    param(
        [string]$Label,
        [string]$Command
    )

    Write-Step $Label
    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        throw "Failed: $Label"
    }
    Write-Ok $Label
}

Write-Step "Check git status"
$branch = (git branch --show-current).Trim()
$statusShort = git status --short
$statusBranch = git status --short --branch | Select-Object -First 1

if ([string]::IsNullOrWhiteSpace($branch)) {
    throw "Cannot get current branch name."
}

if ($branch -eq "develop" -and -not $AllowDevelop) {
    throw "Running on develop is not allowed. Use a feature branch or pass -AllowDevelop."
}

if (-not [string]::IsNullOrWhiteSpace($statusShort)) {
    throw "Working tree is dirty. Commit or stash changes first."
}

if ($statusBranch -match "ahead") {
    Write-Warn "Current branch is ahead of upstream: $statusBranch"
}

Write-Ok "Check git status"

Run-Or-Throw "cargo fmt check" "cargo fmt --all -- --check"

if (-not $SkipClippy) {
    Run-Or-Throw "cargo clippy" "cargo clippy --workspace --all-targets -- -D warnings"
} else {
    Write-Warn "Skip clippy because -SkipClippy is set"
}

if (-not $SkipTests) {
    Run-Or-Throw "cargo test" "cargo test --workspace"
} else {
    Write-Warn "Skip tests because -SkipTests is set"
}

Write-Ok "Pre-PR checks completed"
