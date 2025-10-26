# RedRing Architecture Dependency Check Script
param(
    [switch]$Verbose,
    [switch]$ExitOnError
)

function Write-ColorText {
    param($Text, $Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

function Get-CrateDependencies {
    param([string]$CratePath)

    $cargoToml = Join-Path $CratePath "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        return @()
    }

    $dependencies = @()
    $content = Get-Content $cargoToml
    $inDepsSection = $false

    foreach ($line in $content) {
        if ($line -match '^\[dependencies\]') {
            $inDepsSection = $true
            continue
        }
        if ($line -match '^\[.*\]' -and $inDepsSection) {
            break
        }
        if ($inDepsSection -and $line -match '^(\w+)\s*=') {
            $depName = $matches[1]
            # Check if it's a workspace crate
            $workspaceCrates = @("analysis", "geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "converter", "graphics", "render", "stage", "app")
            if ($workspaceCrates -contains $depName) {
                $dependencies += $depName
            }
        }
    }

    return $dependencies
}

function Test-ArchitectureDependencies {
    Write-ColorText "=== RedRing Architecture Dependency Check ===" "Cyan"
    Write-ColorText "Date: $(Get-Date -Format 'yyyy/MM/dd HH:mm:ss')" "Gray"
    Write-Host ""

    $errorCount = 0
    $warningCount = 0

    # Define workspace structure
    $workspaceCrates = @{
        "analysis"       = "analysis"
        "geo_foundation" = "model\geo_foundation"
        "geo_core"       = "model\geo_core"
        "geo_primitives" = "model\geo_primitives"
        "geo_algorithms" = "model\geo_algorithms"
        "geo_io"         = "model\geo_io"
        "converter"      = "viewmodel\converter"
        "graphics"       = "viewmodel\graphics"
        "render"         = "view\render"
        "stage"          = "view\stage"
        "app"            = "view\app"
    }

    # Define allowed dependencies
    $allowedDeps = @{
        "analysis"       = @()
        "geo_foundation" = @("analysis")
        "geo_core"       = @("geo_foundation", "analysis")
        "geo_primitives" = @("geo_foundation", "geo_core", "analysis")
        "geo_algorithms" = @("geo_foundation", "geo_core", "geo_primitives", "analysis")
        "geo_io"         = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "analysis")
        "converter"      = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "analysis")
        "graphics"       = @("geo_foundation", "geo_core", "geo_primitives", "analysis")
        "render"         = @("analysis")
        "stage"          = @("render", "analysis")
        "app"            = @("converter", "graphics", "render", "stage", "analysis")
    }

    Write-ColorText "1. Checking Model layer naming rules..." "Yellow"
    $modelCrates = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io")
    foreach ($crateName in $modelCrates) {
        if (Test-Path $workspaceCrates[$crateName]) {
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
        $cratePath = $workspaceCrates[$crateName]

        if (Test-Path $cratePath) {
            $actualDeps = Get-CrateDependencies $cratePath
            $allowed = $allowedDeps[$crateName]

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
        else {
            Write-ColorText "  WARN: Crate path not found: $cratePath" "Yellow"
            $warningCount++
        }
    }
    Write-Host ""

    Write-ColorText "3. Layer summary:" "Yellow"
    $layers = @{
        "Analysis"  = @("analysis")
        "Model"     = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io")
        "ViewModel" = @("converter", "graphics")
        "View"      = @("render", "stage", "app")
    }

    foreach ($layerName in $layers.Keys) {
        Write-ColorText "  $layerName layer:" "Cyan"
        foreach ($crateName in $layers[$layerName]) {
            if (Test-Path $workspaceCrates[$crateName]) {
                $deps = Get-CrateDependencies $workspaceCrates[$crateName]
                if ($deps.Length -gt 0) {
                    $depsStr = $deps -join ", "
                    Write-Host "    $crateName -> $depsStr"
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

# Main execution
if ($args -contains "-Help" -or $args -contains "-h") {
    Write-Host "RedRing Architecture Dependency Check Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\scripts\check_architecture_dependencies.ps1 [options]"
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
