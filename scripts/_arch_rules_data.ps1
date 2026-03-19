# RedRing Architecture Rules - Shared Data
# This file is dot-sourced by check_architecture_dependencies*.ps1
# DO NOT execute directly.

# Workspace crate path mapping
$ARCH_LAYER_MAPPING = @{
    analysis       = "foundation/analysis"
    geo_foundation = "model/geo_foundation"
    geo_contracts  = "model/geo_contracts"
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
    job_runtime    = "model/job_runtime"
    job_domain     = "model/job_domain"
    converter      = "viewmodel/converter"
    graphics       = "viewmodel/graphics"
    render         = "view/render"
    stage          = "view/stage"
    app            = "view/app"
}

# Layer groups
$ARCH_LAYERS = @{
    Analysis  = @("analysis")
    Model     = @("geo_foundation", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "job_domain")
    ViewModel = @("converter", "graphics")
    View      = @("render", "stage", "app")
}

# Required model crates
$ARCH_REQUIRED_MODEL_CRATES = @("geo_foundation", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity")

# Allowed dependency rules
# Last updated: 2026-03-19 (#318 geo_foundation retirement transition)
$ARCH_ALLOWED_DEPS = @{
    analysis       = @()
    geo_contracts  = @("analysis")
    geo_foundation = @("analysis", "geo_commons", "geo_contracts")  # geo_contracts: temporary compatibility bridge for #318
    geo_commons    = @("analysis")
    geo_core       = @("analysis", "geo_entity")
    geo_primitives = @("geo_foundation", "geo_contracts", "geo_commons", "geo_core", "analysis")
    geo_algorithms = @("geo_foundation", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "analysis")
    geo_nurbs      = @("geo_foundation", "geo_contracts", "geo_commons", "geo_core", "geo_primitives", "analysis")
    geo_io         = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "analysis")
    geo_entity     = @("geo_foundation", "geo_primitives")
    cam_core       = @("analysis", "geo_foundation", "geo_primitives", "geo_algorithms")
    cam_entity     = @("cam_core", "geo_entity")
    cam_sim        = @("analysis", "cam_core", "geo_algorithms", "job_runtime", "job_domain")
    job_runtime    = @("analysis")
    job_domain     = @("analysis", "job_runtime")
    converter      = @("geo_foundation", "geo_contracts", "geo_algorithms", "geo_io", "cam_core", "cam_entity", "cam_sim", "job_domain", "analysis")
    graphics       = @("geo_foundation", "geo_core", "geo_primitives", "analysis")
    render         = @("analysis")
    stage          = @("render", "analysis")
    app            = @("converter", "graphics", "render", "stage", "analysis")
}

# Forbidden dependency rules
$ARCH_FORBIDDEN_DEPS = @{
    geo_foundation = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_contracts  = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "graphics", "render", "stage", "app")
    geo_commons    = @("converter", "graphics", "render", "stage", "app", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_core       = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_primitives = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_algorithms = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_nurbs      = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_io         = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    geo_entity     = @("converter", "graphics", "render", "stage", "app", "cam_core", "cam_entity", "cam_sim", "job_runtime")
    cam_core       = @("converter", "graphics", "render", "stage", "app", "geo_entity", "cam_sim")
    cam_entity     = @("converter", "graphics", "render", "stage", "app", "cam_sim")
    cam_sim        = @("converter", "graphics", "render", "stage", "app", "geo_entity", "geo_foundation", "geo_core", "geo_primitives", "geo_nurbs", "geo_io", "cam_entity")
    job_runtime    = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics", "render", "stage", "app")
    job_domain     = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "converter", "graphics", "render", "stage", "app")
    converter      = @("render", "stage", "app")
    graphics       = @("render", "stage", "app")
    render         = @("geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics")
    stage          = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "converter", "graphics")
    app            = @("geo_foundation", "geo_core", "geo_primitives", "geo_algorithms", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim")
    analysis       = @("geo_foundation", "geo_commons", "geo_core", "geo_primitives", "geo_algorithms", "geo_nurbs", "geo_io", "geo_entity", "cam_core", "cam_entity", "cam_sim", "job_runtime", "converter", "graphics", "render", "stage", "app")
}

# Shared helper: extract workspace dependencies from Cargo.toml
function Get-CrateDependenciesShared {
    param([string]$CratePath)

    $cargoToml = Join-Path $CratePath "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        return @()
    }

    $knownCrates = $ARCH_ALLOWED_DEPS.Keys
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
            if ($knownCrates -contains $depName) {
                $dependencies += $depName
            }
        }
    }

    return $dependencies
}

# Shared helper: return workspace crate path map
function Get-WorkspaceCratesShared {
    $result = @{}
    foreach ($crateName in $ARCH_LAYER_MAPPING.Keys) {
        $cratePath = Join-Path (Get-Location) $ARCH_LAYER_MAPPING[$crateName]
        if (Test-Path $cratePath) {
            $result[$crateName] = $cratePath
        }
    }
    return $result
}
