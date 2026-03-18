# RedRing Architecture Dependency Check Script
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1

param(
    [switch]$Verbose,
    [switch]$ExitOnError
)

# 共有ルールデータ・共通関数を読み込む
. (Join-Path $PSScriptRoot "_arch_rules_data.ps1")

function Write-Success { param($Message) Write-Host "OK $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "WARN $Message" -ForegroundColor Yellow }
function Write-Error   { param($Message) Write-Host "ERROR $Message" -ForegroundColor Red }
function Write-Info    { param($Message) Write-Host "INFO $Message" -ForegroundColor Cyan }

function Test-ArchitectureDependencies {
    Write-Info "RedRing Architecture Dependency Check Start"
    Write-Info "Date: $(Get-Date -Format 'yyyy/MM/dd HH:mm:ss')"
    Write-Host ""

    $errorCount   = 0
    $warningCount = 0
    $workspaceCrates = Get-WorkspaceCratesShared

    Write-Info "1. Model Layer Naming Rule Check"
    foreach ($crateName in $ARCH_REQUIRED_MODEL_CRATES) {
        if ($workspaceCrates.ContainsKey($crateName)) {
            Write-Success "Model crate '$crateName' follows correct naming rules"
        }
        else {
            Write-Error "Required Model crate '$crateName' not found"
            $errorCount++
        }
    }
    Write-Host ""

    Write-Info "2. Dependency Rule Check"
    foreach ($crateName in $workspaceCrates.Keys) {
        $cratePath  = $workspaceCrates[$crateName]
        $actualDeps = Get-CrateDependenciesShared $cratePath

        Write-Info "Validating '$crateName' dependencies..."

        $allowedDeps   = $ARCH_ALLOWED_DEPS[$crateName]
        $forbiddenDeps = $ARCH_FORBIDDEN_DEPS[$crateName]

        foreach ($dep in $actualDeps) {
            if ($allowedDeps -contains $dep) {
                if ($Verbose) {
                    Write-Success "  OK '$crateName' -> '$dep' (allowed)"
                }
            }
            else {
                Write-Error "  ERROR '$crateName' -> '$dep' (not allowed)"
                $errorCount++
            }
        }

        foreach ($dep in $actualDeps) {
            if ($forbiddenDeps -contains $dep) {
                Write-Error "  ERROR '$crateName' -> '$dep' (explicitly forbidden)"
                $errorCount++
            }
        }

        if ($actualDeps.Length -eq 0) {
            Write-Info "  INFO '$crateName' has no workspace dependencies"
        }
    }
    Write-Host ""

    Write-Info "3. Layer Dependency Summary"
    foreach ($layerName in $ARCH_LAYERS.Keys) {
        Write-Info "Layer: $layerName"
        foreach ($crateName in $ARCH_LAYERS[$layerName]) {
            if ($workspaceCrates.ContainsKey($crateName)) {
                $deps = Get-CrateDependenciesShared $workspaceCrates[$crateName]
                if ($deps.Length -gt 0) {
                    Write-Host "    $crateName -> $($deps -join ', ')" -ForegroundColor White
                }
                else {
                    Write-Host "    $crateName -> (no deps)" -ForegroundColor Gray
                }
            }
            else {
                Write-Warning "    $crateName -> (not found)"
                $warningCount++
            }
        }
    }
    Write-Host ""

    Write-Info "4. Check Result Summary"
    if ($errorCount -eq 0 -and $warningCount -eq 0) {
        Write-Success "SUCCESS: Architecture dependency check passed"
        Write-Success "- geo_* -> cam_* forbidden rule enforced"
        Write-Success "- Requested exceptions/policies are applied"
    }
    else {
        Write-Error "FAILED: Architecture dependency check found issues"
        Write-Error "- Errors: $errorCount"
        Write-Warning "- Warnings: $warningCount"

        if ($ExitOnError -and $errorCount -gt 0) {
            Write-Error "STOP: Exiting due to detected errors"
            exit 1
        }
    }

    return @{ Errors = $errorCount; Warnings = $warningCount }
}

if ($args -contains "-Help" -or $args -contains "-h") {
    Write-Host "RedRing Architecture Dependency Check Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\scripts\check_architecture_dependencies.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Verbose      Show detailed dependency information"
    Write-Host "  -ExitOnError  Exit script on error detection (default: false)"
    Write-Host "  -Help, -h     Show this help"
    exit 0
}

$result = Test-ArchitectureDependencies

if ($env:CI -eq "true") {
    if ($result.Errors -gt 0) {
        Write-Output "::error::Architecture dependency check detected errors"
        exit 1
    }
    if ($result.Warnings -gt 0) {
        Write-Output "::warning::Architecture dependency check detected warnings"
    }
}
