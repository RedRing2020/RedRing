# RedRing Architecture Dependency Check Script
# Usage: powershell -ExecutionPolicy Bypass -File .\scripts\check_architecture_dependencies.ps1

param(
    [switch]$Verbose,
    [switch]$ExitOnError
)

function Write-Success { param($Message) Write-Host "OK $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "WARN $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "ERROR $Message" -ForegroundColor Red }
function Write-Info { param($Message) Write-Host "INFO $Message" -ForegroundColor Cyan }

$ARCHITECTURE_RULES = @{
    AllowedDependencies = @{
        # Analysis
        analysis       = @()

        # Model: geo_*
        geo_foundation = @("analysis", "geo_commons")
        geo_commons    = @("geo_foundation", "analysis")
        geo_core       = @("geo_foundation", "analysis", "geo_entity") # geo_core -> geo_entity: OK
        geo_primitives = @("geo_foundation", "geo_core", "analysis")
        geo_algorithms = @("geo_foundation", "geo_core", "geo_primitives", "analysis", "geo_nurbs")
        geo_nurbs      = @("geo_foundation", "geo_core", "geo_primitives", "analysis")
        geo_io         = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "analysis")
        geo_entity     = @("geo_foundation", "geo_primitives")

        # Model: cam_*
        cam_core       = @("analysis", "geo_foundation", "geo_primitives", "geo_algorithms", "cam_entity") # cam_core -> cam_entity: OK
        cam_entity     = @("cam_core", "geo_entity")
        cam_sim        = @("analysis", "cam_core", "geo_algorithms", "job_runtime")
        job_runtime = @("analysis")

        # ViewModel
        converter      = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "analysis")
        graphics       = @("geo_foundation", "geo_core", "geo_primitives", "analysis")

        # View
        render         = @("analysis")
        stage          = @("render", "analysis")
        app            = @("converter", "graphics", "render", "stage", "analysis")
    }

    ForbiddenDependencies = @{
        # geo_* -> cam_* is forbidden
        geo_foundation = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        geo_commons    = @("converter", "graphics", "render", "stage", "app", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        geo_core       = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime") # geo_core -> cam_entity: NG
        geo_primitives = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        geo_algorithms = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        geo_nurbs      = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        geo_io         = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        geo_entity     = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")

        # cam_*
        cam_core       = @("converter", "graphics", "render", "stage", "app", "geo_entity", "cam_sim") # cam_core -> geo_entity: NG
        cam_entity     = @("converter", "graphics", "render", "stage", "app", "cam_sim")
        cam_sim        = @("converter", "graphics", "render", "stage", "app", "geo_entity", "geo_foundation", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "cam_entity")
        job_runtime = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics", "render", "stage", "app")

        # ViewModel -> View forbidden
        converter      = @("render", "stage", "app")
        graphics       = @("render", "stage", "app")

        # View -> Model forbidden (current policy)
        render         = @("geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics")
        stage          = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics")
        app            = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim")

        # analysis isolation
        analysis       = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "converter", "graphics", "render", "stage", "app")
    }

    NamingRules = @{
        ModelPrefix = "geo_"
        RequiredModelCrates = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity")
    }
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
            if ($ARCHITECTURE_RULES.AllowedDependencies.ContainsKey($depName)) {
                $dependencies += $depName
            }
        }
    }

    return $dependencies
}

function Get-WorkspaceCrates {
    $workspaceCrates = @{}

    $layerMapping = @{
        analysis       = "foundation/analysis"
        geo_foundation = "model/geo_foundation"
        geo_commons    = "model/geo_commons"
        geo_core       = "model/geo_core"
        geo_primitives = "model/geo_primitives"
        geo_algorithms = "model/geo_algorithms"
        geo_nurbs      = "model/geo_nurbs"
        geo_io         = "model/geo_io"
        geo_entity     = "model/geo_entity"
        cam_core       = "model/cam_core"
        cam_entity     = "model/cam_entity"
        cam_sim        = "model/cam_sim"
        job_runtime = "model/job_runtime"
        converter      = "viewmodel/converter"
        graphics       = "viewmodel/graphics"
        render         = "view/render"
        stage          = "view/stage"
        app            = "view/app"
    }

    foreach ($crateName in $layerMapping.Keys) {
        $cratePath = Join-Path (Get-Location) $layerMapping[$crateName]
        if (Test-Path $cratePath) {
            $workspaceCrates[$crateName] = $cratePath
        }
    }

    return $workspaceCrates
}

function Test-ArchitectureDependencies {
    Write-Info "RedRing Architecture Dependency Check Start"
    Write-Info "Date: $(Get-Date -Format 'yyyy/MM/dd HH:mm:ss')"
    Write-Host ""

    $errorCount = 0
    $warningCount = 0
    $workspaceCrates = Get-WorkspaceCrates

    Write-Info "1. Model Layer Naming Rule Check"
    foreach ($crateName in $ARCHITECTURE_RULES.NamingRules.RequiredModelCrates) {
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
        $cratePath = $workspaceCrates[$crateName]
        $actualDeps = Get-CrateDependencies $cratePath

        Write-Info "Validating '$crateName' dependencies..."

        $allowedDeps = $ARCHITECTURE_RULES.AllowedDependencies[$crateName]
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

        $forbiddenDeps = $ARCHITECTURE_RULES.ForbiddenDependencies[$crateName]
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
    $layers = @{
        Analysis  = @("analysis")
        Model     = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime")
        ViewModel = @("converter", "graphics")
        View      = @("render", "stage", "app")
    }

    foreach ($layerName in $layers.Keys) {
        Write-Info "Layer: $layerName"
        foreach ($crateName in $layers[$layerName]) {
            if ($workspaceCrates.ContainsKey($crateName)) {
                $deps = Get-CrateDependencies $workspaceCrates[$crateName]
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
