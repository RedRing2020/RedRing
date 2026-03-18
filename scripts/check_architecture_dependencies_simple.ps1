# RedRing Architecture Dependency Check Script (Simple)
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies_simple.ps1

param(
    [switch]$Verbose,
    [switch]$ExitOnError
)

# 共有ルールデータ・共通関数を読み込む
. (Join-Path $PSScriptRoot "_arch_rules_data.ps1")

function Write-ColorText {
    param($Text, $Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

function Test-ArchitectureDependencies {
    Write-ColorText "=== RedRing Architecture Dependency Check ===" "Cyan"
    Write-ColorText "Date: $(Get-Date -Format 'yyyy/MM/dd HH:mm:ss')" "Gray"
    Write-Host ""

    $errorCount   = 0
    $warningCount = 0
    $workspaceCrates = Get-WorkspaceCratesShared

    Write-ColorText "1. Checking Model layer naming rules..." "Yellow"
    foreach ($crateName in $ARCH_REQUIRED_MODEL_CRATES) {
        if ($workspaceCrates.ContainsKey($crateName)) {
            Write-ColorText "  OK: $crateName follows geo_ prefix rule" "Green"
        }
        else {
            Write-ColorText "  ERROR: Required Model crate '$crateName' not found" "Red"
            $errorCount++
        }
    }
    Write-Host ""

    Write-ColorText "2. Checking dependency rules..." "Yellow"

    foreach ($crateName in $workspaceCrates.Keys) {
        $cratePath  = $workspaceCrates[$crateName]
        $actualDeps = Get-CrateDependenciesShared $cratePath
        $allowed    = $ARCH_ALLOWED_DEPS[$crateName]

        Write-ColorText "  Checking: $crateName" "Cyan"

        foreach ($dep in $actualDeps) {
            if ($allowed -contains $dep) {
                if ($Verbose) {
                    Write-ColorText "    OK: $crateName -> $dep (allowed)" "Green"
                }
            }
            else {
                Write-ColorText "    ERROR: $crateName -> $dep (not allowed)" "Red"
                $errorCount++
            }
        }

        if ($actualDeps.Length -eq 0) {
            Write-ColorText "    INFO: No workspace dependencies" "Gray"
        }
    }
    Write-Host ""

    Write-ColorText "3. Layer summary:" "Yellow"
    foreach ($layerName in $ARCH_LAYERS.Keys) {
        Write-ColorText "  $layerName layer:" "Cyan"
        foreach ($crateName in $ARCH_LAYERS[$layerName]) {
            if ($workspaceCrates.ContainsKey($crateName)) {
                $deps = Get-CrateDependenciesShared $workspaceCrates[$crateName]
                if ($deps.Length -gt 0) {
                    Write-Host "    $crateName -> $($deps -join ', ')"
                }
                else {
                    Write-Host "    $crateName -> (no deps)" -ForegroundColor Gray
                }
            }
            else {
                Write-ColorText "    $crateName -> (not found)" "Yellow"
            }
        }
    }
    Write-Host ""

    Write-ColorText "4. Results:" "Yellow"
    if ($errorCount -eq 0 -and $warningCount -eq 0) {
        Write-ColorText "  SUCCESS: All architecture dependency checks passed!" "Green"
        Write-ColorText "  - View -> ViewModel -> Model direction maintained" "Green"
        Write-ColorText "  - Model layer naming rules followed" "Green"
        Write-ColorText "  - No forbidden dependencies detected" "Green"
    }
    else {
        Write-ColorText "  FAILED: Architecture dependency issues found" "Red"
        Write-ColorText "  - Errors: $errorCount" "Red"
        Write-ColorText "  - Warnings: $warningCount" "Yellow"

        if ($ExitOnError -and $errorCount -gt 0) {
            Write-ColorText "  STOP: Exiting due to errors" "Red"
            exit 1
        }
    }

    return @{ "Errors" = $errorCount; "Warnings" = $warningCount }
}

# ヘルプ表示
if ($args -contains "-Help" -or $args -contains "-h") {
    Write-Host "RedRing Architecture Dependency Check Script (Simple)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\scripts\check_architecture_dependencies_simple.ps1 [options]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Verbose      Show detailed dependency information"
    Write-Host "  -ExitOnError  Exit script on error detection"
    Write-Host "  -Help, -h     Show this help"
    exit 0
}

$result = Test-ArchitectureDependencies

# CI/CD output
if ($env:CI -eq "true") {
    if ($result.Errors -gt 0) {
        Write-Output "::error::Architecture dependency check detected errors"
        exit 1
    }
    if ($result.Warnings -gt 0) {
        Write-Output "::warning::Architecture dependency check detected warnings"
    }
}
