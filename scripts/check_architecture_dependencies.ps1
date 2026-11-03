# RedRing Architecture Dependency Check Script
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1
#
# ワークスペースのクレートと依存を `cargo metadata` から取得し、scripts/_arch_rules_data.ps1 のルールと照合する。
# - ルールにないクレート・許可されていない依存・禁止された依存はエラー
# - レイヤーに属さないクレートはエラー

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

    $errorCount = 0
    $workspaceCrates = Get-WorkspaceCratesShared

    Write-Info "1. Model Layer Naming Rule Check"
    foreach ($crateName in $ARCH_REQUIRED_MODEL_CRATES) {
        if ($workspaceCrates.Contains($crateName)) {
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
        $actualDeps = $workspaceCrates[$crateName].Deps

        if (-not $ARCH_ALLOWED_DEPS.ContainsKey($crateName)) {
            Write-Error "  ERROR '$crateName' has no dependency rule (add it to scripts/_arch_rules_data.ps1)"
            $errorCount++
            continue
        }
        $allowedDeps   = $ARCH_ALLOWED_DEPS[$crateName]
        $forbiddenDeps = $ARCH_FORBIDDEN_DEPS[$crateName]

        foreach ($dep in $actualDeps) {
            if ($forbiddenDeps -contains $dep) {
                Write-Error "  ERROR '$crateName' -> '$dep' (explicitly forbidden)"
                $errorCount++
            }
            elseif ($allowedDeps -contains $dep) {
                if ($Verbose) {
                    Write-Success "  OK '$crateName' -> '$dep' (allowed)"
                }
            }
            else {
                Write-Error "  ERROR '$crateName' -> '$dep' (not allowed)"
                $errorCount++
            }
        }
    }
    Write-Host ""

    Write-Info "3. Layer Dependency Summary"
    $layered = @()
    foreach ($layerName in $ARCH_LAYERS.Keys) {
        Write-Info "Layer: $layerName"
        foreach ($crateName in $ARCH_LAYERS[$layerName]) {
            $layered += $crateName
            if ($workspaceCrates.Contains($crateName)) {
                $deps = $workspaceCrates[$crateName].Deps
                if ($deps.Length -gt 0) {
                    Write-Host "    $crateName -> $($deps -join ', ')" -ForegroundColor White
                }
                else {
                    Write-Host "    $crateName -> (no deps)" -ForegroundColor Gray
                }
            }
            else {
                Write-Error "    $crateName -> (not found in workspace)"
                $errorCount++
            }
        }
    }
    foreach ($crateName in $workspaceCrates.Keys) {
        if ($layered -notcontains $crateName) {
            Write-Error "    '$crateName' does not belong to any layer (add it to `$ARCH_LAYERS)"
            $errorCount++
        }
    }
    Write-Host ""

    Write-Info "4. Check Result Summary"
    if ($errorCount -eq 0) {
        Write-Success "SUCCESS: Architecture dependency check passed ($($workspaceCrates.Count) crates)"
    }
    else {
        Write-Error "FAILED: Architecture dependency check found $errorCount error(s)"
        if ($ExitOnError) {
            Write-Error "STOP: Exiting due to detected errors"
            exit 1
        }
    }

    return @{ Errors = $errorCount }
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

if ($env:CI -eq "true" -and $result.Errors -gt 0) {
    Write-Output "::error::Architecture dependency check detected errors"
    exit 1
}
